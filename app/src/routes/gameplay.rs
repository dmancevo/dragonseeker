use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    Form, Json,
};
use serde::Deserialize;
use time::OffsetDateTime;

use crate::{
    auth::middleware::AuthenticatedPlayer,
    core::game_session::GameState,
    services::{
        game_state::{
            can_start_voting, transition_to_finished, transition_to_playing, transition_to_voting,
        },
        voting::{all_votes_submitted, can_vote},
        win_conditions::{check_dragon_eliminated, determine_winner},
    },
    state::AppState,
};

/// Query parameters for gameplay endpoints
#[derive(Deserialize)]
pub struct PlayerQuery {
    player_id: String,
}

/// Form data for voting
#[derive(Deserialize)]
pub struct VoteForm {
    target_id: String,
}

/// Form data for word guessing
#[derive(Deserialize)]
pub struct GuessForm {
    guess: String,
}

/// Player data for game template
#[derive(Clone)]
pub struct GamePlayer {
    pub nickname: String,
    pub is_alive: bool,
    pub first_letter: String,
}

/// Current player data for game template
pub struct CurrentPlayer {
    pub role: Option<String>,
    pub is_host: bool,
    pub is_alive: bool,
    pub is_dragon: bool,
    pub is_knight: bool,
}

/// Game state for template
pub struct GameStateInfo {
    pub value: String,
    pub is_playing: bool,
    pub is_voting: bool,
    pub is_dragon_guess: bool,
}

/// Template for the game page
#[derive(Template)]
#[template(path = "game.html")]
pub struct GameTemplate {
    pub game_id: String,
    pub player_id: String,
    pub word: Option<String>,
    pub state: GameStateInfo,
    pub players: Vec<GamePlayer>,
    pub player: CurrentPlayer,
    pub alive_count: usize,
    pub can_show_host_controls: bool,
    pub can_show_voting_area: bool,
    pub can_show_spectator_message: bool,
    pub can_show_guess_area: bool,
    pub can_show_dragon_guessing_message: bool,
}

/// Show the active game interface
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
/// Rendered game page template or redirect
pub async fn show_game(
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

    let player = game
        .players
        .get(&query.player_id)
        .ok_or((StatusCode::FORBIDDEN, "Not in this game".to_string()))?;

    // Redirect to lobby if game hasn't started
    if game.state == GameState::Lobby {
        return Ok(Redirect::to(&format!(
            "/game/{}/lobby?player_id={}",
            game_id, query.player_id
        ))
        .into_response());
    }

    // Redirect to results if game is finished
    if game.state == GameState::Finished {
        return Ok(Redirect::to(&format!(
            "/game/{}/results?player_id={}",
            game_id, query.player_id
        ))
        .into_response());
    }

    // Determine which word to show based on player's role
    let word = if player.knows_word {
        match player.role.as_deref() {
            Some("knight") => game.knight_word.clone(),
            _ => game.villager_word.clone(), // Villager
        }
    } else {
        None
    };

    // Build player list for template
    let game_players: Vec<GamePlayer> = game
        .players
        .values()
        .map(|p| GamePlayer {
            nickname: p.nickname.clone(),
            is_alive: p.is_alive,
            first_letter: p
                .nickname
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string(),
        })
        .collect();

    // Calculate alive count
    let alive_count = game.players.values().filter(|p| p.is_alive).count();

    // Determine player role flags
    let is_dragon = player.role.as_deref() == Some("dragon");
    let is_knight = player.role.as_deref() == Some("knight");

    // Determine current state
    let is_playing = game.state == GameState::Playing;
    let is_voting = game.state == GameState::Voting;
    let is_dragon_guess = game.state == GameState::DragonGuess;

    // Determine which UI sections to show
    let can_show_host_controls = player.is_host && is_playing;
    let can_show_voting_area = is_voting && player.is_alive;
    let can_show_spectator_message = !player.is_alive;
    let can_show_guess_area = is_dragon && is_dragon_guess;
    let can_show_dragon_guessing_message = !is_dragon && is_dragon_guess;

    // Build template
    let template = GameTemplate {
        game_id: game_id.clone(),
        player_id: query.player_id.clone(),
        word,
        state: GameStateInfo {
            value: format!("{:?}", game.state).to_lowercase(),
            is_playing,
            is_voting,
            is_dragon_guess,
        },
        players: game_players,
        player: CurrentPlayer {
            role: player.role.clone(),
            is_host: player.is_host,
            is_alive: player.is_alive,
            is_dragon,
            is_knight,
        },
        alive_count,
        can_show_host_controls,
        can_show_voting_area,
        can_show_spectator_message,
        can_show_guess_area,
        can_show_dragon_guessing_message,
    };

    Ok(template.into_response())
}

/// Transition game to voting phase
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
/// Success message with timer info
pub async fn start_voting(
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
            "Only host can start voting".to_string(),
        ));
    }

    let (can_vote_now, error_msg) = can_start_voting(game);
    if !can_vote_now {
        return Err((StatusCode::BAD_REQUEST, error_msg));
    }

    // Transition to voting
    transition_to_voting(game);

    // Set voting start timestamp if timer configured
    if game.voting_timer_seconds.is_some() {
        game.voting_started_at = Some(OffsetDateTime::now_utc());
        tracing::debug!(
            "Starting timer: {:?}s at {:?}",
            game.voting_timer_seconds,
            game.voting_started_at
        );
    } else {
        tracing::debug!("No timer configured (voting_timer_seconds is None)");
    }

    // Broadcast state update trigger to all connected players
    // Each WebSocket will fetch personalized state for its player
    let broadcast_msg = serde_json::json!({
        "type": "update_trigger",
        "event": "voting_started"
    });
    if let Ok(msg_text) = serde_json::to_string(&broadcast_msg) {
        tracing::debug!("Broadcasting voting_started trigger: {}", msg_text);
        match game.broadcast_tx.send(msg_text) {
            Ok(receiver_count) => tracing::debug!("Broadcast sent to {} receivers", receiver_count),
            Err(e) => tracing::warn!("Broadcast failed: {:?}", e),
        }
    }

    Ok(Json(serde_json::json!({
        "status": "voting_started",
        "timer_seconds": game.voting_timer_seconds
    })))
}

/// Get voting timer HTML (polled by HTMX every second)
///
/// Note: No authentication required for performance (high-frequency polling).
/// Rate limited to 20 req/s and validates player exists/is alive.
///
/// # Arguments
///
/// * `game_id` - The game session ID from path
/// * `query` - Query params with player_id
/// * `state` - Shared application state
///
/// # Returns
///
/// Rendered timer HTML snippet (currently JSON)
pub async fn get_timer(
    Path(game_id): Path<String>,
    Query(query): Query<PlayerQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let manager = state.game_manager.read().await;
    let game = match manager.get_game(&game_id) {
        Some(g) => g,
        None => {
            return "--:--".into_response();
        }
    };

    let player = game.players.get(&query.player_id);
    if player.is_none() || !player.unwrap().is_alive {
        return "--:--".into_response();
    }

    // Check if in voting state with timer
    if game.state != GameState::Voting || game.voting_timer_seconds.is_none() {
        return "--:--".into_response();
    }

    // Calculate time remaining
    let time_remaining = match game.get_voting_time_remaining() {
        Some(t) => t,
        None => {
            return "--:--".into_response();
        }
    };

    // Check if timer expired
    if time_remaining == 0 {
        drop(manager);

        let mut manager = state.game_manager.write().await;
        if let Some(game) = manager.get_game_mut(&game_id) {
            // Only transition to playing if still in voting state
            // (voting might have completed and transitioned to DragonGuess or Finished)
            if game.state == GameState::Voting {
                tracing::info!("Voting timer expired for game={}", game_id);
                transition_to_playing(game);

                let broadcast_msg = serde_json::json!({
                    "type": "update_trigger",
                    "event": "timer_expired"
                });
                if let Ok(msg_text) = serde_json::to_string(&broadcast_msg) {
                    let _ = game.broadcast_tx.send(msg_text);
                }
            }
        }

        return "0:00".into_response();
    }

    // Return formatted time as plain HTML
    let minutes = time_remaining / 60;
    let seconds = time_remaining % 60;
    format!("{}:{:02}", minutes, seconds).into_response()
}

/// Submit a vote for player elimination
///
/// # Arguments
///
/// * `game_id` - The game session ID from path
/// * `query` - Query params with player_id
/// * `form` - Form data with target_id
/// * `state` - Shared application state
/// * `auth` - Authenticated player (from middleware)
///
/// # Returns
///
/// Vote status or game result
pub async fn submit_vote(
    Path(game_id): Path<String>,
    Query(query): Query<PlayerQuery>,
    State(state): State<AppState>,
    auth: AuthenticatedPlayer,
    Form(form): Form<VoteForm>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Verify token matches the requested player
    auth.verify_matches(&game_id, &query.player_id)
        .map_err(|(status, msg)| (status, msg.to_string()))?;

    let mut manager = state.game_manager.write().await;
    let game = manager
        .get_game_mut(&game_id)
        .ok_or((StatusCode::NOT_FOUND, "Game not found".to_string()))?;

    // Check if player can vote
    let (can_submit, error_msg) = can_vote(game, &query.player_id);
    if !can_submit {
        return Err((
            StatusCode::BAD_REQUEST,
            error_msg.unwrap_or_else(|| "Cannot vote".to_string()),
        ));
    }

    // Submit the vote
    game.submit_vote(&query.player_id, &form.target_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Check if all votes are in
    if all_votes_submitted(game) {
        // Tally votes
        let result = game.tally_votes();

        // Check win condition
        let winner = determine_winner(game);

        let mut headers = HeaderMap::new();

        let dragon_eliminated = check_dragon_eliminated(game);
        tracing::info!(
            "Voting complete: dragon_eliminated={}, winner={:?}, state_before={:?}",
            dragon_eliminated,
            winner,
            game.state
        );

        if dragon_eliminated {
            // Transition to dragon guess state
            game.state = GameState::DragonGuess;
            tracing::info!("→ Transitioning to DragonGuess state");
        } else if let Some(ref winner_name) = winner {
            transition_to_finished(game, winner_name.clone());
            tracing::info!("→ Game finished, winner: {}", winner_name);
            // Redirect to results page when game is finished
            headers.insert(
                "HX-Redirect",
                format!("/game/{}/results?player_id={}", game_id, query.player_id)
                    .parse()
                    .unwrap(),
            );
        } else {
            // Continue playing
            transition_to_playing(game);
            tracing::info!("→ Continuing to Playing state");
        }

        // Broadcast update trigger to all connected players
        let broadcast_msg = serde_json::json!({
            "type": "update_trigger",
            "event": "voting_complete"
        });
        if let Ok(msg_text) = serde_json::to_string(&broadcast_msg) {
            let send_result = game.broadcast_tx.send(msg_text);
            tracing::info!(
                "Broadcast voting_complete: receivers={}, state={:?}",
                send_result.unwrap_or(0),
                game.state
            );
        }

        return Ok((
            headers,
            Json(serde_json::json!({
                "status": "vote_complete",
                "result": result,
                "winner": winner,
                "game_state": format!("{:?}", game.state)
            })),
        ));
    }

    // Vote submitted, waiting for others - broadcast update trigger
    let broadcast_msg = serde_json::json!({
        "type": "update_trigger",
        "event": "vote_submitted"
    });
    if let Ok(msg_text) = serde_json::to_string(&broadcast_msg) {
        let _ = game.broadcast_tx.send(msg_text);
    }

    let alive_count = game.players.values().filter(|p| p.is_alive).count();
    Ok((
        HeaderMap::new(),
        Json(serde_json::json!({
            "status": "vote_submitted",
            "votes_submitted": game.votes.len(),
            "total_players": alive_count
        })),
    ))
}

/// Dragon attempts to guess the secret word after elimination
///
/// # Arguments
///
/// * `game_id` - The game session ID from path
/// * `query` - Query params with player_id
/// * `form` - Form data with guess
/// * `state` - Shared application state
/// * `auth` - Authenticated player (from middleware)
///
/// # Returns
///
/// Guess result and winner
pub async fn guess_word(
    Path(game_id): Path<String>,
    Query(query): Query<PlayerQuery>,
    State(state): State<AppState>,
    auth: AuthenticatedPlayer,
    Form(form): Form<GuessForm>,
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

    if player.role.as_deref() != Some("dragon") {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Dragon can guess the word".to_string(),
        ));
    }

    if game.state != GameState::DragonGuess {
        return Err((
            StatusCode::BAD_REQUEST,
            "Not in dragon guess phase".to_string(),
        ));
    }

    let villager_word = game.villager_word.as_ref().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Game state error: word not set".to_string(),
    ))?;

    // Clean and validate guess
    let guess = form.guess.trim().to_lowercase();

    // Validate guess length (word pairs are typically < 20 chars)
    if guess.len() > 50 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Guess too long (max 50 characters)".to_string(),
        ));
    }

    if guess.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Guess cannot be empty".to_string()));
    }

    // Check if guess is correct (check against villager word)
    let correct = guess == villager_word.to_lowercase();

    // Set winner
    let winner = if correct { "dragon" } else { "villagers" };
    game.dragon_guess = Some(guess.clone());

    transition_to_finished(game, winner.to_string());

    // Broadcast update trigger for game finished
    let broadcast_msg = serde_json::json!({
        "type": "update_trigger",
        "event": "dragon_guessed"
    });
    if let Ok(msg_text) = serde_json::to_string(&broadcast_msg) {
        let _ = game.broadcast_tx.send(msg_text);
    }

    // Redirect to results page
    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        format!("/game/{}/results?player_id={}", game_id, query.player_id)
            .parse()
            .unwrap(),
    );

    Ok((
        headers,
        Json(serde_json::json!({
            "correct": correct,
            "winner": winner
        })),
    ))
}

/// Template for the results page
#[derive(Template)]
#[template(path = "results.html")]
pub struct ResultsTemplate {
    pub winner: Option<String>,
    pub villager_word: Option<String>,
    pub knight_word: Option<String>,
    pub dragon_guess: Option<String>,
    pub players: Vec<crate::core::player::Player>,
}

/// Show the game results page
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
/// Rendered results page template
pub async fn show_results(
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

    let _player = game
        .players
        .get(&query.player_id)
        .ok_or((StatusCode::FORBIDDEN, "Not in this game".to_string()))?;

    // Collect players into a vector
    let players: Vec<crate::core::player::Player> = game.players.values().cloned().collect();

    // Build template
    let template = ResultsTemplate {
        winner: game.winner.clone(),
        villager_word: game.villager_word.clone(),
        knight_word: game.knight_word.clone(),
        dragon_guess: game.dragon_guess.clone(),
        players,
    };

    Ok(template.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_query_deserialization() {
        let json = r#"{"player_id": "player123"}"#;
        let query: PlayerQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.player_id, "player123");
    }

    #[test]
    fn test_vote_form_deserialization() {
        let json = r#"{"target_id": "player456"}"#;
        let form: VoteForm = serde_json::from_str(json).unwrap();
        assert_eq!(form.target_id, "player456");
    }

    #[test]
    fn test_guess_form_deserialization() {
        let json = r#"{"guess": "elephant"}"#;
        let form: GuessForm = serde_json::from_str(json).unwrap();
        assert_eq!(form.guess, "elephant");
    }
}
