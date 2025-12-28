use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Player information for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerResponse {
    /// Player's unique ID
    pub id: String,
    /// Player's display name
    pub nickname: String,
    /// Whether player is alive
    pub is_alive: bool,
    /// Whether player is the host
    pub is_host: bool,
    /// Player's role (only included in game-over state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

impl PlayerResponse {
    /// Create a PlayerResponse from a Player
    ///
    /// # Arguments
    ///
    /// * `player` - The player to convert
    /// * `include_role` - Whether to include the role field
    pub fn from_player(player: &crate::core::Player, include_role: bool) -> Self {
        Self {
            id: player.id.clone(),
            nickname: player.nickname.clone(),
            is_alive: player.is_alive,
            is_host: player.is_host,
            role: if include_role {
                player.role.clone()
            } else {
                None
            },
        }
    }
}

/// Game state response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateResponse {
    /// Game's unique ID
    pub game_id: String,
    /// Current game state
    pub state: String,
    /// List of players
    pub players: Vec<PlayerResponse>,
    /// Total number of players
    pub player_count: usize,
    /// Number of alive players
    pub alive_count: usize,
    /// Whether the game can be started
    pub can_start: bool,
}

/// Vote tallying result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResultResponse {
    /// ID of eliminated player
    pub eliminated_id: String,
    /// Nickname of eliminated player
    pub eliminated_nickname: String,
    /// Role of eliminated player
    pub eliminated_role: String,
    /// Vote counts per player (player_id -> count)
    pub vote_counts: HashMap<String, usize>,
    /// Whether the elimination was a tie
    pub was_tie: bool,
}

/// Final game result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResultResponse {
    /// Winner: "dragon" or "villagers"
    pub winner: String,
    /// The secret word
    pub word: String,
    /// Dragon's guess (if dragon was eliminated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dragon_guess: Option<String>,
    /// All players with roles revealed
    pub players: Vec<PlayerResponse>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Player;

    #[test]
    fn test_player_response_from_player_without_role() {
        let player = Player::new("Alice".to_string(), true);
        let response = PlayerResponse::from_player(&player, false);

        assert_eq!(response.id, player.id);
        assert_eq!(response.nickname, "Alice");
        assert!(response.is_alive);
        assert!(response.is_host);
        assert_eq!(response.role, None);
    }

    #[test]
    fn test_player_response_from_player_with_role() {
        let mut player = Player::new("Bob".to_string(), false);
        player.role = Some("dragon".to_string());

        let response = PlayerResponse::from_player(&player, true);

        assert_eq!(response.id, player.id);
        assert_eq!(response.nickname, "Bob");
        assert!(response.is_alive);
        assert!(!response.is_host);
        assert_eq!(response.role, Some("dragon".to_string()));
    }

    #[test]
    fn test_game_state_response_creation() {
        let response = GameStateResponse {
            game_id: "game123".to_string(),
            state: "lobby".to_string(),
            players: vec![],
            player_count: 0,
            alive_count: 0,
            can_start: false,
        };

        assert_eq!(response.game_id, "game123");
        assert_eq!(response.state, "lobby");
        assert_eq!(response.player_count, 0);
    }

    #[test]
    fn test_vote_result_response_creation() {
        let mut vote_counts = HashMap::new();
        vote_counts.insert("player1".to_string(), 2);
        vote_counts.insert("player2".to_string(), 1);

        let response = VoteResultResponse {
            eliminated_id: "player1".to_string(),
            eliminated_nickname: "Alice".to_string(),
            eliminated_role: "villager".to_string(),
            vote_counts,
            was_tie: false,
        };

        assert_eq!(response.eliminated_id, "player1");
        assert_eq!(response.eliminated_nickname, "Alice");
        assert!(!response.was_tie);
        assert_eq!(response.vote_counts.get("player1"), Some(&2));
    }

    #[test]
    fn test_game_result_response_creation() {
        let response = GameResultResponse {
            winner: "dragon".to_string(),
            word: "elephant".to_string(),
            dragon_guess: Some("elephant".to_string()),
            players: vec![],
        };

        assert_eq!(response.winner, "dragon");
        assert_eq!(response.word, "elephant");
        assert_eq!(response.dragon_guess, Some("elephant".to_string()));
    }

    #[test]
    fn test_serialization_player_response() {
        let response = PlayerResponse {
            id: "player123".to_string(),
            nickname: "Alice".to_string(),
            is_alive: true,
            is_host: true,
            role: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"id\":\"player123\""));
        assert!(json.contains("\"nickname\":\"Alice\""));
        assert!(!json.contains("\"role\"")); // Should be omitted when None
    }

    #[test]
    fn test_serialization_player_response_with_role() {
        let response = PlayerResponse {
            id: "player123".to_string(),
            nickname: "Bob".to_string(),
            is_alive: true,
            is_host: false,
            role: Some("dragon".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"role\":\"dragon\"")); // Should be included
    }
}
