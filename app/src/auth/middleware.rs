use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::{request::Parts, StatusCode},
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;

use crate::state::AppState;

/// Authenticated player information extracted from request
#[derive(Debug, Clone)]
pub struct AuthenticatedPlayer {
    pub game_id: String,
    pub player_id: String,
    pub expiry: u64,
}

#[derive(Deserialize)]
struct PlayerIdQuery {
    player_id: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedPlayer
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract player_id from query parameters
        let Query(query) = Query::<PlayerIdQuery>::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, "Missing player_id parameter"))?;

        // Get cookie jar
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to extract cookies",
                )
            })?;

        // Get player-specific cookie
        let cookie_name = format!("player_token_{}", query.player_id);
        let token = jar
            .get(&cookie_name)
            .ok_or((StatusCode::UNAUTHORIZED, "Missing authentication token"))?
            .value();

        // Get secret key from extensions (set by middleware)
        let app_state = parts
            .extensions
            .get::<AppState>()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "App state not found"))?;

        // Verify token
        let (game_id, player_id, expiry) =
            super::token::verify_player_token(Some(token), &app_state.secret_key).ok_or((
                StatusCode::UNAUTHORIZED,
                "Invalid or expired authentication token",
            ))?;

        // Verify player_id matches query parameter
        if player_id != query.player_id {
            return Err((StatusCode::FORBIDDEN, "Token does not match player"));
        }

        Ok(AuthenticatedPlayer {
            game_id,
            player_id,
            expiry,
        })
    }
}

impl AuthenticatedPlayer {
    /// Verify that the token matches the expected game and player
    ///
    /// # Arguments
    ///
    /// * `expected_game_id` - The expected game ID
    /// * `expected_player_id` - The expected player ID
    ///
    /// # Returns
    ///
    /// Ok if match, Err with HTTP status and message otherwise
    pub fn verify_matches(
        &self,
        expected_game_id: &str,
        expected_player_id: &str,
    ) -> Result<(), (StatusCode, &'static str)> {
        if self.game_id != expected_game_id || self.player_id != expected_player_id {
            return Err((
                StatusCode::FORBIDDEN,
                "Authentication token does not match player",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_matches_success() {
        let auth = AuthenticatedPlayer {
            game_id: "game123".to_string(),
            player_id: "player456".to_string(),
            expiry: 0,
        };

        let result = auth.verify_matches("game123", "player456");
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_matches_wrong_game() {
        let auth = AuthenticatedPlayer {
            game_id: "game123".to_string(),
            player_id: "player456".to_string(),
            expiry: 0,
        };

        let result = auth.verify_matches("game999", "player456");
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_verify_matches_wrong_player() {
        let auth = AuthenticatedPlayer {
            game_id: "game123".to_string(),
            player_id: "player456".to_string(),
            expiry: 0,
        };

        let result = auth.verify_matches("game123", "player999");
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
}
