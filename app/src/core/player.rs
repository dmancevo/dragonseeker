use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Represents a player in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Unique player identifier
    pub id: String,
    /// Player's display name
    pub nickname: String,
    /// Player's role (villager, knight, or dragon) - set when game starts
    pub role: Option<String>,
    /// Whether the player is still alive in the game
    pub is_alive: bool,
    /// Whether this player is the game host
    pub is_host: bool,
    /// False for Dragon, True for Villagers/Knights
    pub knows_word: bool,
    /// Timestamp when player joined
    pub joined_at: OffsetDateTime,
}

impl Player {
    /// Create a new player
    ///
    /// # Arguments
    ///
    /// * `nickname` - The player's display name
    /// * `is_host` - Whether this player is the game host
    ///
    /// # Returns
    ///
    /// A new Player instance with a unique ID
    pub fn new(nickname: String, is_host: bool) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            nickname,
            role: None,
            is_alive: true,
            is_host,
            knows_word: false,
            joined_at: OffsetDateTime::now_utc(),
        }
    }

    /// Convert player to dictionary for API responses
    ///
    /// # Arguments
    ///
    /// * `include_role` - Whether to include the player's role (only for game end)
    ///
    /// # Returns
    ///
    /// JSON value representation of the player
    pub fn to_dict(&self, include_role: bool) -> serde_json::Value {
        if include_role {
            serde_json::json!({
                "id": self.id,
                "nickname": self.nickname,
                "is_alive": self.is_alive,
                "is_host": self.is_host,
                "role": self.role,
            })
        } else {
            serde_json::json!({
                "id": self.id,
                "nickname": self.nickname,
                "is_alive": self.is_alive,
                "is_host": self.is_host,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_player() {
        let player = Player::new("TestPlayer".to_string(), false);

        assert_eq!(player.nickname, "TestPlayer");
        assert!(!player.is_host);
        assert!(player.is_alive);
        assert!(!player.knows_word);
        assert!(player.role.is_none());
        assert!(!player.id.is_empty());
    }

    #[test]
    fn test_new_host_player() {
        let player = Player::new("Host".to_string(), true);

        assert!(player.is_host);
    }

    #[test]
    fn test_to_dict_without_role() {
        let player = Player::new("Test".to_string(), false);
        let dict = player.to_dict(false);

        assert_eq!(dict["nickname"], "Test");
        assert_eq!(dict["is_alive"], true);
        assert_eq!(dict["is_host"], false);
        assert!(dict.get("role").is_none());
    }

    #[test]
    fn test_to_dict_with_role() {
        let mut player = Player::new("Test".to_string(), false);
        player.role = Some("villager".to_string());

        let dict = player.to_dict(true);

        assert_eq!(dict["role"], "villager");
    }
}
