use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use tokio::sync::broadcast;

use super::{assign_roles, Player, MIN_PLAYERS, WORD_PAIRS};

/// Game state enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameState {
    Lobby,
    Playing,
    Voting,
    DragonGuess,
    Finished,
}

/// Manages a single game session
#[derive(Debug)]
pub struct GameSession {
    /// Unique identifier for this game
    pub game_id: String,
    /// Map of player ID to Player
    pub players: HashMap<String, Player>,
    /// Current game state
    pub state: GameState,
    /// Word for villagers
    pub villager_word: Option<String>,
    /// Similar word for knights
    pub knight_word: Option<String>,
    /// When the game was created
    pub created_at: OffsetDateTime,
    /// When the game started
    pub started_at: Option<OffsetDateTime>,
    /// When game finished
    pub finished_at: Option<OffsetDateTime>,
    /// Map of voter_id -> target_id
    pub votes: HashMap<String, String>,
    /// Broadcast channel for WebSocket updates
    pub broadcast_tx: broadcast::Sender<String>,
    /// "villagers" or "dragon"
    pub winner: Option<String>,
    /// Dragon's word guess if eliminated
    pub dragon_guess: Option<String>,
    /// ID of most recently eliminated player
    pub eliminated_player_id: Option<String>,
    /// Last elimination details
    pub last_elimination: Option<serde_json::Value>,
    /// Shuffled order of player IDs for turn order
    pub player_order: Vec<String>,
    /// Timer duration (30-180), None = disabled
    pub voting_timer_seconds: Option<u32>,
    /// When voting started (for time calculation)
    pub voting_started_at: Option<OffsetDateTime>,
}

impl GameSession {
    /// Create a new game session
    ///
    /// # Arguments
    ///
    /// * `game_id` - Unique identifier for this game
    pub fn new(game_id: String) -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);

        Self {
            game_id,
            players: HashMap::new(),
            state: GameState::Lobby,
            villager_word: None,
            knight_word: None,
            created_at: OffsetDateTime::now_utc(),
            started_at: None,
            finished_at: None,
            votes: HashMap::new(),
            broadcast_tx,
            winner: None,
            dragon_guess: None,
            eliminated_player_id: None,
            last_elimination: None,
            player_order: Vec::new(),
            voting_timer_seconds: None,
            voting_started_at: None,
        }
    }

    /// Add a new player to the game
    ///
    /// # Arguments
    ///
    /// * `nickname` - The player's display name
    ///
    /// # Returns
    ///
    /// The newly created Player object
    ///
    /// # Errors
    ///
    /// Returns an error if game is not in lobby state
    pub fn add_player(&mut self, nickname: String) -> Result<Player, String> {
        if self.state != GameState::Lobby {
            return Err("Cannot join game that has already started".to_string());
        }

        let is_host = self.players.is_empty(); // First player is host
        let player = Player::new(nickname, is_host);
        self.players.insert(player.id.clone(), player.clone());
        Ok(player)
    }

    /// Remove a player from the game
    ///
    /// # Arguments
    ///
    /// * `player_id` - ID of the player to remove
    pub fn remove_player(&mut self, player_id: &str) {
        self.players.remove(player_id);

        // If host left, assign new host
        if !self.players.is_empty() && !self.players.values().any(|p| p.is_host) {
            if let Some(next_player) = self.players.values_mut().next() {
                next_player.is_host = true;
            }
        }
    }

    /// Check if the game can be started
    ///
    /// # Returns
    ///
    /// True if minimum players requirement is met
    pub fn can_start(&self) -> bool {
        self.players.len() >= MIN_PLAYERS
    }

    /// Set voting timer duration for all rounds
    ///
    /// # Arguments
    ///
    /// * `seconds` - Timer duration (30-180) or None to disable
    ///
    /// # Errors
    ///
    /// Returns an error if game not in lobby or invalid timer value
    pub fn set_voting_timer(&mut self, seconds: Option<u32>) -> Result<(), String> {
        if self.state != GameState::Lobby {
            return Err("Can only set timer in lobby".to_string());
        }

        if let Some(secs) = seconds {
            if !(30..=180).contains(&secs) {
                return Err("Timer must be between 30 and 180 seconds".to_string());
            }
        }

        self.voting_timer_seconds = seconds;
        Ok(())
    }

    /// Calculate remaining voting time in seconds
    ///
    /// # Returns
    ///
    /// Seconds remaining, or None if no timer active
    pub fn get_voting_time_remaining(&self) -> Option<i64> {
        if self.voting_timer_seconds.is_none() || self.voting_started_at.is_none() {
            return None;
        }

        let timer_seconds = self.voting_timer_seconds?;
        let started_at = self.voting_started_at?;

        let elapsed = (OffsetDateTime::now_utc() - started_at).whole_seconds();
        let remaining = (timer_seconds as i64) - elapsed;

        Some(remaining.max(0)) // Don't return negative values
    }

    /// Start the game by assigning roles and selecting a word
    ///
    /// # Errors
    ///
    /// Returns an error if game cannot be started
    pub fn start_game(&mut self) -> Result<(), String> {
        if !self.can_start() {
            return Err(format!("Need at least {} players to start", MIN_PLAYERS));
        }

        if self.state != GameState::Lobby {
            return Err("Game has already started".to_string());
        }

        // Assign roles
        let mut players_list: Vec<Player> = self.players.values().cloned().collect();
        assign_roles(&mut players_list)?;

        // Update players with assigned roles
        for player in players_list {
            self.players.insert(player.id.clone(), player);
        }

        // Select random word pair
        let mut rng = thread_rng();
        let word_pair = WORD_PAIRS
            .choose(&mut rng)
            .ok_or("No word pairs available")?;
        self.villager_word = Some(word_pair.0.to_string());
        self.knight_word = Some(word_pair.1.to_string());

        // Shuffle and store player order for turn-based word saying
        let mut player_ids: Vec<String> = self.players.keys().cloned().collect();
        player_ids.shuffle(&mut rng);
        self.player_order = player_ids;

        // Update state
        self.state = GameState::Playing;
        self.started_at = Some(OffsetDateTime::now_utc());

        Ok(())
    }

    /// Submit a vote to eliminate a player
    ///
    /// # Arguments
    ///
    /// * `voter_id` - ID of the player voting
    /// * `target_id` - ID of the player being voted for
    ///
    /// # Errors
    ///
    /// Returns an error if voting is not allowed
    pub fn submit_vote(&mut self, voter_id: &str, target_id: &str) -> Result<(), String> {
        if self.state != GameState::Voting {
            return Err("Not in voting phase".to_string());
        }

        let voter = self.players.get(voter_id).ok_or("Voter doesn't exist")?;
        let target = self.players.get(target_id).ok_or("Target doesn't exist")?;

        if !voter.is_alive {
            return Err("Voter is not alive".to_string());
        }

        if !target.is_alive {
            return Err("Cannot vote for dead player".to_string());
        }

        self.votes
            .insert(voter_id.to_string(), target_id.to_string());
        Ok(())
    }

    /// Tally votes and determine eliminated player
    ///
    /// # Returns
    ///
    /// JSON value with vote results
    pub fn tally_votes(&mut self) -> serde_json::Value {
        if self.votes.is_empty() {
            return serde_json::json!({
                "eliminated": null,
                "vote_counts": {}
            });
        }

        // Count votes
        let mut vote_counts: HashMap<String, usize> = HashMap::new();
        for target_id in self.votes.values() {
            *vote_counts.entry(target_id.clone()).or_insert(0) += 1;
        }

        // Find max votes
        let max_votes = *vote_counts.values().max().unwrap_or(&0);

        // Get all players with max votes (for tie-breaking)
        let tied_players: Vec<String> = vote_counts
            .iter()
            .filter(|(_, &count)| count == max_votes)
            .map(|(pid, _)| pid.clone())
            .collect();

        // Random tie-breaker
        let mut rng = thread_rng();
        let eliminated_id = tied_players.choose(&mut rng).unwrap().clone();

        if let Some(eliminated_player) = self.players.get_mut(&eliminated_id) {
            eliminated_player.is_alive = false;
            self.eliminated_player_id = Some(eliminated_id.clone());

            // Store elimination details for display
            let elimination_details = serde_json::json!({
                "eliminated_id": eliminated_id,
                "eliminated_nickname": eliminated_player.nickname,
                "eliminated_role": eliminated_player.role,
                "vote_counts": vote_counts,
                "was_tie": tied_players.len() > 1,
            });

            self.last_elimination = Some(elimination_details.clone());
            elimination_details
        } else {
            serde_json::json!({
                "eliminated": null,
                "vote_counts": vote_counts
            })
        }
    }

    /// Check if game has reached a win condition
    ///
    /// # Returns
    ///
    /// "villagers", "dragon", or None if game continues
    pub fn check_win_condition(&self) -> Option<String> {
        let alive_players: Vec<&Player> = self.players.values().filter(|p| p.is_alive).collect();
        let dragon = self
            .players
            .values()
            .find(|p| p.role.as_deref() == Some("dragon"));

        // Dragon was eliminated
        if let Some(d) = dragon {
            if !d.is_alive {
                // Give dragon a chance to guess the word
                return None; // Will transition to DRAGON_GUESS state
            }
        }

        // Only 2 players left and Dragon is alive
        if alive_players.len() <= 2 {
            if let Some(d) = dragon {
                if d.is_alive {
                    return Some("dragon".to_string());
                }
            }
        }

        None // Game continues
    }

    /// Get game state customized for a specific player
    ///
    /// # Arguments
    ///
    /// * `player_id` - ID of the player to get state for
    ///
    /// # Returns
    ///
    /// JSON value with game state
    pub fn get_state_for_player(&self, player_id: &str) -> serde_json::Value {
        let player = match self.players.get(player_id) {
            Some(p) => p,
            None => return serde_json::json!({}),
        };

        let alive_count = self.players.values().filter(|p| p.is_alive).count();

        // Determine which word to show based on role
        let your_word = if player.knows_word {
            match player.role.as_deref() {
                Some("knight") => self.knight_word.clone(),
                _ => self.villager_word.clone(), // Villager
            }
        } else {
            None
        };

        let mut state_data = serde_json::json!({
            "game_id": self.game_id,
            "state": self.state,
            "your_id": player_id,
            "your_role": player.role,
            "your_word": your_word,
            "is_host": player.is_host,
            "is_alive": player.is_alive,
            "players": self.players.values().map(|p| p.to_dict(false)).collect::<Vec<_>>(),
            "player_count": self.players.len(),
            "alive_count": alive_count,
            "can_start": self.can_start(),
            "votes_submitted": self.votes.len(),
            "has_voted": self.votes.contains_key(player_id),
            "last_elimination": self.last_elimination,
            "player_order": self.player_order,
            "voting_timer_seconds": self.voting_timer_seconds,
            "voting_started_at": self.voting_started_at.map(|t| t.unix_timestamp()),
        });

        if self.state == GameState::Finished {
            state_data["winner"] = serde_json::json!(self.winner);
            state_data["villager_word"] = serde_json::json!(self.villager_word);
            state_data["knight_word"] = serde_json::json!(self.knight_word);
            state_data["dragon_guess"] = serde_json::json!(self.dragon_guess);
            state_data["players"] = serde_json::json!(self
                .players
                .values()
                .map(|p| p.to_dict(true))
                .collect::<Vec<_>>());
        }

        state_data
    }

    /// Broadcast current game state to all connected players
    ///
    /// Note: In Rust, we use a broadcast channel instead of storing WebSocket connections
    /// WebSocket handlers subscribe to the broadcast channel
    pub fn broadcast_state(&self) {
        tracing::info!(
            "📢 Broadcasting state for game {} (state: {:?})",
            self.game_id,
            self.state
        );

        for player_id in self.players.keys() {
            let state = self.get_state_for_player(player_id);
            let message = serde_json::json!({
                "type": "state_update",
                "data": state
            });

            if let Ok(msg_str) = serde_json::to_string(&message) {
                // Send to broadcast channel (ignore errors if no receivers)
                let _ = self.broadcast_tx.send(msg_str);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_session() {
        let game = GameSession::new("test-123".to_string());

        assert_eq!(game.game_id, "test-123");
        assert_eq!(game.state, GameState::Lobby);
        assert!(game.players.is_empty());
        assert!(game.villager_word.is_none());
        assert!(game.knight_word.is_none());
    }

    #[test]
    fn test_add_player() {
        let mut game = GameSession::new("test-123".to_string());

        let player = game.add_player("Alice".to_string()).unwrap();
        assert_eq!(player.nickname, "Alice");
        assert!(player.is_host); // First player is host

        let player2 = game.add_player("Bob".to_string()).unwrap();
        assert_eq!(player2.nickname, "Bob");
        assert!(!player2.is_host);

        assert_eq!(game.players.len(), 2);
    }

    #[test]
    fn test_cannot_join_started_game() {
        let mut game = GameSession::new("test-123".to_string());
        game.state = GameState::Playing;

        let result = game.add_player("Alice".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_can_start() {
        let mut game = GameSession::new("test-123".to_string());

        assert!(!game.can_start()); // 0 players

        game.add_player("Alice".to_string()).unwrap();
        assert!(!game.can_start()); // 1 player

        game.add_player("Bob".to_string()).unwrap();
        assert!(!game.can_start()); // 2 players

        game.add_player("Charlie".to_string()).unwrap();
        assert!(game.can_start()); // 3 players - minimum
    }

    #[test]
    fn test_set_voting_timer() {
        let mut game = GameSession::new("test-123".to_string());

        // Valid timer
        assert!(game.set_voting_timer(Some(60)).is_ok());
        assert_eq!(game.voting_timer_seconds, Some(60));

        // Disable timer
        assert!(game.set_voting_timer(None).is_ok());
        assert!(game.voting_timer_seconds.is_none());

        // Invalid timer (too low)
        assert!(game.set_voting_timer(Some(20)).is_err());

        // Invalid timer (too high)
        assert!(game.set_voting_timer(Some(200)).is_err());

        // Cannot set after game starts
        game.state = GameState::Playing;
        assert!(game.set_voting_timer(Some(60)).is_err());
    }

    #[test]
    fn test_start_game() {
        let mut game = GameSession::new("test-123".to_string());

        game.add_player("Alice".to_string()).unwrap();
        game.add_player("Bob".to_string()).unwrap();
        game.add_player("Charlie".to_string()).unwrap();

        let result = game.start_game();
        assert!(result.is_ok());

        assert_eq!(game.state, GameState::Playing);
        assert!(game.villager_word.is_some());
        assert!(game.knight_word.is_some());
        assert!(game.started_at.is_some());
        assert_eq!(game.player_order.len(), 3);

        // All players should have roles assigned
        for player in game.players.values() {
            assert!(player.role.is_some());
        }
    }

    #[test]
    fn test_submit_vote() {
        let mut game = GameSession::new("test-123".to_string());

        let p1 = game.add_player("Alice".to_string()).unwrap();
        let p2 = game.add_player("Bob".to_string()).unwrap();

        // Cannot vote in lobby
        assert!(game.submit_vote(&p1.id, &p2.id).is_err());

        game.state = GameState::Voting;

        // Can vote in voting phase
        assert!(game.submit_vote(&p1.id, &p2.id).is_ok());
        assert_eq!(game.votes.get(&p1.id), Some(&p2.id));
    }

    #[test]
    fn test_remove_player_reassigns_host() {
        let mut game = GameSession::new("test-123".to_string());

        let host = game.add_player("Host".to_string()).unwrap();
        let player2 = game.add_player("Player2".to_string()).unwrap();

        assert!(game.players.get(&host.id).unwrap().is_host);
        assert!(!game.players.get(&player2.id).unwrap().is_host);

        // Remove host
        game.remove_player(&host.id);

        // Player2 should become host
        assert!(game.players.get(&player2.id).unwrap().is_host);
    }
}
