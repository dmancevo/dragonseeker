use askama::Template;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;

use crate::{
    auth::token::generate_player_token, core::game_session::GameState, models::JoinGameRequest,
    state::AppState,
};

/// Template for the join page
#[derive(Template)]
#[template(path = "join.html")]
struct JoinTemplate {
    game_id: String,
}

/// Template for the index/create page
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

/// Show the game creation page
///
/// # Returns
///
/// Rendered index template
pub async fn show_index() -> impl IntoResponse {
    IndexTemplate {}
}

/// Create a new game session
///
/// # Arguments
///
/// * `state` - Shared application state
///
/// # Returns
///
/// JSON response with HX-Redirect header to join page
pub async fn create_game(State(state): State<AppState>) -> impl IntoResponse {
    let mut manager = state.game_manager.write().await;
    let game_id = manager.create_game();

    // Create response with HX-Redirect header for HTMX
    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        format!("/game/{}/join", game_id).parse().unwrap(),
    );

    (
        headers,
        axum::Json(serde_json::json!({
            "status": "created",
            "game_id": game_id
        })),
    )
}

/// Show the join page where players enter their nickname
///
/// # Arguments
///
/// * `game_id` - The game session ID from path
/// * `state` - Shared application state
///
/// # Returns
///
/// Rendered join page template or error
pub async fn show_join_page(
    Path(game_id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let manager = state.game_manager.read().await;
    let game = manager
        .get_game(&game_id)
        .ok_or((StatusCode::NOT_FOUND, "Game not found".to_string()))?;

    if game.state != GameState::Lobby {
        return Err((
            StatusCode::BAD_REQUEST,
            "Game has already started".to_string(),
        ));
    }

    Ok(JoinTemplate { game_id })
}

/// Form data for joining a game
#[derive(Deserialize)]
pub struct JoinForm {
    nickname: String,
}

/// Add a player to the game session
///
/// # Arguments
///
/// * `game_id` - The game session ID from path
/// * `form` - Form data with nickname
/// * `state` - Shared application state
/// * `jar` - Cookie jar for setting auth cookie
///
/// # Returns
///
/// JSON response with HX-Redirect header and authentication cookie
pub async fn join_game(
    Path(game_id): Path<String>,
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<JoinForm>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Validate nickname
    let validated_request =
        JoinGameRequest::new(form.nickname).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let mut manager = state.game_manager.write().await;
    let game = manager
        .get_game_mut(&game_id)
        .ok_or((StatusCode::NOT_FOUND, "Game not found".to_string()))?;

    if game.state != GameState::Lobby {
        return Err((
            StatusCode::BAD_REQUEST,
            "Game has already started".to_string(),
        ));
    }

    // Check for duplicate nicknames
    if game
        .players
        .values()
        .any(|p| p.nickname.to_lowercase() == validated_request.nickname.to_lowercase())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Nickname already taken".to_string(),
        ));
    }

    // Add player
    let player = game
        .add_player(validated_request.nickname)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let player_id = player.id.clone();

    // Generate authentication token
    let token = generate_player_token(&game_id, &player_id, &state.secret_key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Determine if we're in development mode (allow HTTP cookies)
    let is_development = std::env::var("ENVIRONMENT")
        .unwrap_or_default()
        .eq_ignore_ascii_case("development");

    // Create authentication cookie (player-specific to allow multiple players in same browser)
    let cookie = Cookie::build((format!("player_token_{}", player_id), token))
        .path("/")
        .http_only(true)
        .secure(!is_development) // HTTPS in production, HTTP in dev
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .max_age(time::Duration::hours(24))
        .build();

    let jar = jar.add(cookie);

    // Broadcast state update trigger to all connected players
    // Each WebSocket will need to send personalized state to its client
    // We use a simple trigger message that tells WebSockets to send updates
    let broadcast_msg = serde_json::json!({
        "type": "update_trigger",
        "event": "player_joined"
    });
    if let Ok(msg_text) = serde_json::to_string(&broadcast_msg) {
        let _ = game.broadcast_tx.send(msg_text);
    }

    // Create response with HX-Redirect header
    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        format!("/game/{}/lobby?player_id={}", game_id, player_id)
            .parse()
            .unwrap(),
    );

    Ok((
        jar,
        headers,
        axum::Json(serde_json::json!({
            "status": "joined",
            "player_id": player_id,
            "is_host": player.is_host
        })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_template_creation() {
        let template = JoinTemplate {
            game_id: "test123".to_string(),
        };
        assert_eq!(template.game_id, "test123");
    }

    #[test]
    fn test_index_template_creation() {
        let _template = IndexTemplate {};
        // Just verify it compiles
    }
}
