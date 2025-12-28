use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use futures::{SinkExt, StreamExt};

use crate::{auth::token::verify_player_token, state::AppState};

/// WebSocket endpoint for real-time game updates
///
/// # Arguments
///
/// * `game_id` - The game session ID from path
/// * `player_id` - The player's ID from path
/// * `ws` - WebSocket upgrade request
/// * `jar` - Cookie jar for authentication
/// * `state` - Shared application state
///
/// # Returns
///
/// WebSocket upgrade response or error
///
/// # Flow
///
/// 1. Validate game and player
/// 2. Authenticate via player-specific cookie
/// 3. Accept WebSocket connection
/// 4. Send initial state to player
/// 5. Subscribe to broadcast channel for updates
/// 6. Keep connection alive with ping/pong
/// 7. Handle messages and timeouts
pub async fn websocket_handler(
    Path((game_id, player_id)): Path<(String, String)>,
    ws: WebSocketUpgrade,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::debug!(
        "WebSocket connection attempt: game={}, player={}",
        game_id,
        player_id
    );

    // Authenticate player via player-specific cookie
    let cookie_name = format!("player_token_{}", player_id);
    let player_token = jar.get(&cookie_name).map(|c| c.value());

    let token_data = verify_player_token(player_token, &state.secret_key).ok_or((
        StatusCode::UNAUTHORIZED,
        "Invalid or expired authentication token".to_string(),
    ))?;

    // Verify token matches requested player and game
    if token_data.0 != game_id || token_data.1 != player_id {
        tracing::warn!("Token mismatch for player: {}", player_id);
        return Err((
            StatusCode::UNAUTHORIZED,
            "Authentication token does not match player".to_string(),
        ));
    }

    // Validate game exists and player is in game
    let manager = state.game_manager.read().await;
    let game = manager
        .get_game(&game_id)
        .ok_or((StatusCode::NOT_FOUND, "Game not found".to_string()))?;

    if !game.players.contains_key(&player_id) {
        tracing::warn!("Player not in game: {}", player_id);
        return Err((StatusCode::NOT_FOUND, "Player not in game".to_string()));
    }

    // Get broadcast receiver before dropping the read lock
    let broadcast_rx = game.broadcast_tx.subscribe();

    // Get initial state
    let initial_state = game.get_state_for_player(&player_id);

    drop(manager); // Release read lock

    tracing::info!("WebSocket accepted: player={} game={}", player_id, game_id);

    // Upgrade to WebSocket connection
    Ok(ws.on_upgrade(move |socket| {
        handle_socket(
            socket,
            game_id,
            player_id,
            initial_state,
            broadcast_rx,
            state,
        )
    }))
}

/// Handle WebSocket connection
///
/// # Arguments
///
/// * `socket` - The WebSocket connection
/// * `game_id` - The game session ID
/// * `player_id` - The player's ID
/// * `initial_state` - Initial game state to send
/// * `broadcast_rx` - Broadcast receiver for state updates
/// * `state` - Shared application state for fetching fresh game state
async fn handle_socket(
    socket: WebSocket,
    game_id: String,
    player_id: String,
    initial_state: serde_json::Value,
    mut broadcast_rx: tokio::sync::broadcast::Receiver<String>,
    state: AppState,
) {
    let (mut sender, mut receiver) = socket.split();

    // Send initial state
    let initial_message = serde_json::json!({
        "type": "state_update",
        "data": initial_state
    });

    if let Ok(msg_text) = serde_json::to_string(&initial_message) {
        if sender.send(Message::Text(msg_text)).await.is_ok() {
            tracing::debug!("Sent initial state to player={}", player_id);
        }
    }

    // Spawn task to handle broadcast updates
    let player_id_clone = player_id.clone();
    let game_id_clone = game_id.clone();
    let state_clone = state.clone();
    let mut send_task = tokio::spawn(async move {
        // Listen for broadcast updates
        loop {
            match broadcast_rx.recv().await {
                Ok(broadcast_msg) => {
                    // Parse the broadcast message to check if it's an update trigger
                    if let Ok(msg_json) = serde_json::from_str::<serde_json::Value>(&broadcast_msg)
                    {
                        if msg_json["type"] == "update_trigger" {
                            tracing::info!(
                                "Player {} processing update_trigger: event={:?}",
                                player_id_clone,
                                msg_json["event"]
                            );
                            // Fetch fresh personalized state for this player
                            let manager = state_clone.game_manager.read().await;
                            if let Some(game) = manager.get_game(&game_id_clone) {
                                let player_state = game.get_state_for_player(&player_id_clone);
                                tracing::info!(
                                    "Sending state to player {}: state={}, role={:?}, is_alive={}",
                                    player_id_clone,
                                    player_state
                                        .get("state")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown"),
                                    player_state.get("your_role"),
                                    player_state
                                        .get("is_alive")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false)
                                );
                                let state_msg = serde_json::json!({
                                    "type": "state_update",
                                    "data": player_state
                                });
                                if let Ok(msg_text) = serde_json::to_string(&state_msg) {
                                    if sender.send(Message::Text(msg_text)).await.is_err() {
                                        tracing::warn!(
                                            "Failed to send state to player={}",
                                            player_id_clone
                                        );
                                        break;
                                    }
                                }
                            }
                        } else {
                            // Forward other message types directly
                            if sender.send(Message::Text(broadcast_msg)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
                Err(_) => {
                    tracing::debug!("Broadcast channel closed for player={}", player_id_clone);
                    break;
                }
            }
        }
    });

    // Handle incoming messages from client
    let player_id_clone = player_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Validate message size (1KB max)
                    if text.len() > 1024 {
                        tracing::warn!(
                            "Message too large from player={}: {} bytes",
                            player_id_clone,
                            text.len()
                        );
                        break;
                    }

                    tracing::debug!("Received from player={}: {}", player_id_clone, text);

                    // Handle ping/pong (Axum handles WebSocket pings automatically,
                    // but we support application-level ping for compatibility)
                    if text == "ping" {
                        // Echo back as pong (handled by client task)
                        tracing::debug!("Ping from player={}", player_id_clone);
                    }
                }
                Message::Close(_) => {
                    tracing::debug!("Close message from player={}", player_id_clone);
                    break;
                }
                Message::Ping(_) | Message::Pong(_) => {
                    // Axum handles WebSocket ping/pong frames automatically
                }
                Message::Binary(_) => {
                    // We don't expect binary messages
                    tracing::warn!("Unexpected binary message from player={}", player_id_clone);
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => {
            tracing::debug!("Send task finished for player={}", player_id);
            recv_task.abort();
        }
        _ = &mut recv_task => {
            tracing::debug!("Receive task finished for player={}", player_id);
            send_task.abort();
        }
    }

    tracing::info!(
        "WebSocket connection closed: player={} game={}",
        player_id,
        game_id
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cookie_name_format() {
        let player_id = "player123";
        let cookie_name = format!("player_token_{}", player_id);
        assert_eq!(cookie_name, "player_token_player123");
    }

    #[test]
    fn test_initial_message_format() {
        let initial_state = serde_json::json!({
            "game_id": "test",
            "state": "lobby"
        });

        let message = serde_json::json!({
            "type": "state_update",
            "data": initial_state
        });

        assert_eq!(message["type"], "state_update");
        assert_eq!(message["data"]["game_id"], "test");
    }
}
