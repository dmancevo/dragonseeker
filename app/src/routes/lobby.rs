use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    Json,
};
use serde::Deserialize;

use crate::{
    auth::middleware::AuthenticatedPlayer, core::constants::MIN_PLAYERS,
    services::game_state::can_start_game, state::AppState,
};

/// Player data for lobby template
#[derive(Clone)]
pub struct LobbyPlayer {
    pub nickname: String,
    pub is_host: bool,
    pub first_letter: String,
}

/// Template for the lobby page
#[derive(Template)]
#[template(path = "lobby.html")]
pub struct LobbyTemplate {
    pub share_url: String,
    pub game_id: String,
    pub players: Vec<LobbyPlayer>,
    pub min_players: usize,
    pub is_host: bool,
    pub player_id: String,
}

/// Query parameters for lobby/game endpoints
#[derive(Deserialize)]
pub struct PlayerQuery {
    player_id: String,
}

/// Show the lobby page with player list and start button
///
/// # Arguments
///
/// * `game_id` - The game session ID from path
/// * `query` - Query params with player_id
/// * `state` - Shared application state
/// * `auth` - Authenticated player (from middleware)
///
/// # Returns
///
/// Rendered lobby page template or redirect
pub async fn show_lobby(
    Path(game_id): Path<String>,
    Query(query): Query<PlayerQuery>,
    State(state): State<AppState>,
    auth: AuthenticatedPlayer,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Verify token matches the requested player
    auth.verify_matches(&game_id, &query.player_id)
        .map_err(|(status, msg)| (status, msg.to_string()))?;

    let manager = state.game_manager.read().await;
    let game = manager
        .get_game(&game_id)
        .ok_or((StatusCode::NOT_FOUND, "Game not found".to_string()))?;

    let player = game.players.get(&query.player_id);

    if player.is_none() {
        // Player not in game, redirect to join page
        return Ok(Redirect::to(&format!("/game/{}/join", game_id)).into_response());
    }

    let player = player.unwrap();

    // Build share URL (use a placeholder base URL for now)
    // In production, this would use the request's base URL
    let share_url = format!("https://dragonseeker.win/game/{}/join", game_id);

    // Build player list for template
    let lobby_players: Vec<LobbyPlayer> = game
        .players
        .values()
        .map(|p| LobbyPlayer {
            nickname: p.nickname.clone(),
            is_host: p.is_host,
            first_letter: p
                .nickname
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string(),
        })
        .collect();

    // Build template
    let template = LobbyTemplate {
        share_url,
        game_id: game_id.clone(),
        players: lobby_players,
        min_players: MIN_PLAYERS,
        is_host: player.is_host,
        player_id: query.player_id.clone(),
    };

    Ok(template.into_response())
}

/// Request body for set-timer endpoint
#[derive(Deserialize)]
pub struct SetTimerRequest {
    timer_seconds: Option<u32>,
}

/// Start the game (assign roles and transition to playing)
///
/// Only callable by the host.
///
/// # Arguments
///
/// * `game_id` - The game session ID from path
/// * `query` - Query params with player_id
/// * `state` - Shared application state
/// * `auth` - Authenticated player (from middleware)
///
/// # Returns
///
/// JSON response with HX-Redirect header to game page
pub async fn start_game(
    Path(game_id): Path<String>,
    Query(query): Query<PlayerQuery>,
    State(state): State<AppState>,
    auth: AuthenticatedPlayer,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Verify token matches the requested player
    auth.verify_matches(&game_id, &query.player_id)
        .map_err(|(status, msg)| (status, msg.to_string()))?;

    let mut manager = state.game_manager.write().await;
    let game = manager
        .get_game_mut(&game_id)
        .ok_or((StatusCode::NOT_FOUND, "Game not found".to_string()))?;

    let player = game
        .players
        .get(&query.player_id)
        .ok_or((StatusCode::NOT_FOUND, "Player not found".to_string()))?;

    if !player.is_host {
        return Err((
            StatusCode::FORBIDDEN,
            "Only host can start the game".to_string(),
        ));
    }

    // Validate game can start
    let (can_start, error_msg) = can_start_game(game);
    if !can_start {
        return Err((StatusCode::BAD_REQUEST, error_msg));
    }

    // Start the game
    game.start_game()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Broadcast state update trigger to all connected players
    // Each WebSocket will fetch personalized state for its player
    let broadcast_msg = serde_json::json!({
        "type": "update_trigger",
        "event": "game_started"
    });
    if let Ok(msg_text) = serde_json::to_string(&broadcast_msg) {
        let _ = game.broadcast_tx.send(msg_text);
    }

    // Create response with HX-Redirect header
    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        format!("/game/{}/play?player_id={}", game_id, query.player_id)
            .parse()
            .unwrap(),
    );

    Ok((
        headers,
        Json(serde_json::json!({
            "status": "started",
            "game_id": game_id
        })),
    ))
}

/// Set voting timer for all rounds (host only)
///
/// # Arguments
///
/// * `game_id` - The game session ID from path
/// * `query` - Query params with player_id
/// * `request` - JSON body with timer_seconds
/// * `state` - Shared application state
/// * `auth` - Authenticated player (from middleware)
///
/// # Returns
///
/// Success message
pub async fn set_timer(
    Path(game_id): Path<String>,
    Query(query): Query<PlayerQuery>,
    State(state): State<AppState>,
    auth: AuthenticatedPlayer,
    Json(request): Json<SetTimerRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Verify token matches the requested player
    auth.verify_matches(&game_id, &query.player_id)
        .map_err(|(status, msg)| (status, msg.to_string()))?;

    let mut manager = state.game_manager.write().await;
    let game = manager
        .get_game_mut(&game_id)
        .ok_or((StatusCode::NOT_FOUND, "Game not found".to_string()))?;

    let player = game
        .players
        .get(&query.player_id)
        .ok_or((StatusCode::NOT_FOUND, "Player not found".to_string()))?;

    if !player.is_host {
        return Err((StatusCode::FORBIDDEN, "Only host can set timer".to_string()));
    }

    tracing::debug!(
        "Setting timer for game={}: {:?}s",
        game_id, request.timer_seconds
    );

    // Set the timer
    game.set_voting_timer(request.timer_seconds).map_err(|e| {
        tracing::warn!("Timer validation error: {}", e);
        (StatusCode::BAD_REQUEST, e)
    })?;

    tracing::debug!(
        "Timer set successfully: game.voting_timer_seconds = {:?}",
        game.voting_timer_seconds
    );

    Ok(Json(serde_json::json!({
        "status": "timer_set",
        "timer_seconds": request.timer_seconds
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Re-enable when Askama templates are implemented
    // #[test]
    // fn test_lobby_template_creation() {
    //     let template = LobbyTemplate {
    //         game_id: "test123".to_string(),
    //         player_id: "player456".to_string(),
    //         is_host: true,
    //         share_url: "http://localhost:8000/game/test123/join".to_string(),
    //         min_players: 3,
    //     };
    //
    //     assert_eq!(template.game_id, "test123");
    //     assert_eq!(template.player_id, "player456");
    //     assert!(template.is_host);
    //     assert_eq!(template.min_players, 3);
    // }

    #[test]
    fn test_player_query_deserialization() {
        let json = r#"{"player_id": "player123"}"#;
        let query: PlayerQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.player_id, "player123");
    }

    #[test]
    fn test_set_timer_request_deserialization() {
        let json = r#"{"timer_seconds": 60}"#;
        let request: SetTimerRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.timer_seconds, Some(60));

        let json_null = r#"{"timer_seconds": null}"#;
        let request_null: SetTimerRequest = serde_json::from_str(json_null).unwrap();
        assert_eq!(request_null.timer_seconds, None);
    }
}
