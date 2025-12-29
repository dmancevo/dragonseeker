use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashMap;
use time::{Duration, OffsetDateTime};

use super::{GameSession, GameState, FINISHED_GAME_TTL_SECONDS, GAME_TTL_SECONDS};

/// Manager for all active game sessions
#[derive(Debug)]
pub struct GameManager {
    /// Map of game_id to GameSession
    games: HashMap<String, GameSession>,
}

impl GameManager {
    /// Create a new game manager
    pub fn new() -> Self {
        Self {
            games: HashMap::new(),
        }
    }

    /// Create a new game session with a unique ID
    ///
    /// # Returns
    ///
    /// The game_id of the newly created game
    pub fn create_game(&mut self) -> String {
        // Generate a unique, URL-safe game ID (8 characters)
        // Using 6 bytes of random data encoded as base64url gives ~8 chars
        let mut game_id = Self::generate_game_id();

        // Ensure uniqueness (very unlikely to collide, but check anyway)
        while self.games.contains_key(&game_id) {
            game_id = Self::generate_game_id();
        }

        let game = GameSession::new(game_id.clone());
        self.games.insert(game_id.clone(), game);
        game_id
    }

    /// Generate a cryptographically secure URL-safe random game ID
    ///
    /// Uses OsRng (OS-provided secure random) for cryptographic security
    /// 12 characters = ~71 bits of entropy (62^12 possibilities)
    fn generate_game_id() -> String {
        use rand::rngs::OsRng;

        OsRng
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect()
    }

    /// Retrieve a game session by ID
    ///
    /// # Arguments
    ///
    /// * `game_id` - The game's unique identifier
    ///
    /// # Returns
    ///
    /// Reference to the GameSession if found, None otherwise
    pub fn get_game(&self, game_id: &str) -> Option<&GameSession> {
        self.games.get(game_id)
    }

    /// Retrieve a mutable game session by ID
    ///
    /// # Arguments
    ///
    /// * `game_id` - The game's unique identifier
    ///
    /// # Returns
    ///
    /// Mutable reference to the GameSession if found, None otherwise
    pub fn get_game_mut(&mut self, game_id: &str) -> Option<&mut GameSession> {
        self.games.get_mut(game_id)
    }

    /// Remove a game session
    ///
    /// # Arguments
    ///
    /// * `game_id` - The game's unique identifier
    pub fn remove_game(&mut self, game_id: &str) {
        self.games.remove(game_id);
    }

    /// Remove games that are too old or finished
    ///
    /// # Returns
    ///
    /// Number of games cleaned up
    pub fn cleanup_stale_games(&mut self) -> usize {
        let now = OffsetDateTime::now_utc();
        let cutoff_time = now - Duration::seconds(GAME_TTL_SECONDS as i64);
        let finished_cutoff = now - Duration::seconds(FINISHED_GAME_TTL_SECONDS as i64);

        let stale_game_ids: Vec<String> = self
            .games
            .iter()
            .filter_map(|(game_id, game)| {
                // Remove old unfinished games (after 1 hour)
                if game.created_at < cutoff_time {
                    return Some(game_id.clone());
                }

                // Remove finished games (after 30 minutes)
                if game.state == GameState::Finished {
                    if let Some(finished_at) = game.finished_at {
                        if finished_at < finished_cutoff {
                            return Some(game_id.clone());
                        }
                    }
                }

                None
            })
            .collect();

        for game_id in &stale_game_ids {
            self.remove_game(game_id);
        }

        stale_game_ids.len()
    }

    /// Get statistics about active games
    ///
    /// # Returns
    ///
    /// JSON value with game statistics
    pub fn get_stats(&self) -> serde_json::Value {
        let total_players: usize = self.games.values().map(|game| game.players.len()).sum();
        let active_games = self
            .games
            .values()
            .filter(|game| game.state != GameState::Finished)
            .count();

        serde_json::json!({
            "total_games": self.games.len(),
            "active_games": active_games,
            "total_players": total_players,
        })
    }
}

impl Default for GameManager {
    fn default() -> Self {
        Self::new()
    }
}

// Note: In Rust, we don't need a module-level singleton
// The GameManager will be wrapped in Arc<RwLock<GameManager>> in AppState

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_game() {
        let mut manager = GameManager::new();

        let game_id = manager.create_game();
        assert!(!game_id.is_empty());

        let game = manager.get_game(&game_id).unwrap();
        assert_eq!(game.state, GameState::Lobby);
    }

    #[test]
    fn test_get_game() {
        let mut manager = GameManager::new();

        let game_id = manager.create_game();

        let retrieved = manager.get_game(&game_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().game_id, game_id);
    }

    #[test]
    fn test_get_nonexistent_game() {
        let manager = GameManager::new();

        let result = manager.get_game("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_remove_game() {
        let mut manager = GameManager::new();

        let game_id = manager.create_game();

        assert!(manager.get_game(&game_id).is_some());

        manager.remove_game(&game_id);

        assert!(manager.get_game(&game_id).is_none());
    }

    #[test]
    fn test_unique_game_ids() {
        let mut manager = GameManager::new();

        let game_id1 = manager.create_game();
        let game_id2 = manager.create_game();
        let game_id3 = manager.create_game();

        assert_ne!(game_id1, game_id2);
        assert_ne!(game_id2, game_id3);
        assert_ne!(game_id1, game_id3);
    }

    #[test]
    fn test_game_id_format() {
        let mut manager = GameManager::new();

        let game_id = manager.create_game();

        // Game ID should be exactly 12 characters
        assert_eq!(game_id.len(), 12);

        // Game ID should only contain alphanumeric characters
        assert!(game_id.chars().all(|c| c.is_alphanumeric()));

        // Verify it's URL-safe (no special characters)
        assert!(!game_id.contains('/'));
        assert!(!game_id.contains('+'));
        assert!(!game_id.contains('='));
    }

    #[test]
    fn test_get_stats_empty() {
        let manager = GameManager::new();

        let stats = manager.get_stats();
        assert_eq!(stats["total_games"], 0);
        assert_eq!(stats["active_games"], 0);
        assert_eq!(stats["total_players"], 0);
    }

    #[test]
    fn test_get_stats_with_games() {
        let mut manager = GameManager::new();

        let game_id1 = manager.create_game();
        let game_id2 = manager.create_game();

        // Add players to games
        if let Some(game) = manager.get_game_mut(&game_id1) {
            game.add_player("Alice".to_string()).unwrap();
            game.add_player("Bob".to_string()).unwrap();
        }

        if let Some(game) = manager.get_game_mut(&game_id2) {
            game.add_player("Charlie".to_string()).unwrap();
        }

        let stats = manager.get_stats();
        assert_eq!(stats["total_games"], 2);
        assert_eq!(stats["active_games"], 2); // Both in lobby
        assert_eq!(stats["total_players"], 3);
    }

    #[test]
    fn test_cleanup_stale_games() {
        let mut manager = GameManager::new();

        // Create a game
        let game_id = manager.create_game();

        // Manually set created_at to be old (more than 1 hour ago)
        if let Some(game) = manager.get_game_mut(&game_id) {
            game.created_at = OffsetDateTime::now_utc() - Duration::hours(2);
        }

        // Cleanup should remove it
        let cleaned = manager.cleanup_stale_games();
        assert_eq!(cleaned, 1);
        assert!(manager.get_game(&game_id).is_none());
    }

    #[test]
    fn test_cleanup_finished_games() {
        let mut manager = GameManager::new();

        // Create a finished game
        let game_id = manager.create_game();

        // Set as finished and old
        if let Some(game) = manager.get_game_mut(&game_id) {
            game.state = GameState::Finished;
            game.finished_at = Some(OffsetDateTime::now_utc() - Duration::minutes(35));
            // Older than 30 min
        }

        // Cleanup should remove it
        let cleaned = manager.cleanup_stale_games();
        assert_eq!(cleaned, 1);
        assert!(manager.get_game(&game_id).is_none());
    }

    #[test]
    fn test_cleanup_keeps_recent_games() {
        let mut manager = GameManager::new();

        // Create a recent game
        let game_id = manager.create_game();

        // Cleanup should not remove it
        let cleaned = manager.cleanup_stale_games();
        assert_eq!(cleaned, 0);
        assert!(manager.get_game(&game_id).is_some());
    }
}
