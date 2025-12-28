use crate::core::{
    constants::MIN_PLAYERS,
    game_session::{GameSession, GameState},
};
use time::OffsetDateTime;

/// Check if game can be started
///
/// # Arguments
///
/// * `game` - The game session
///
/// # Returns
///
/// Tuple of (can_start, error_message)
pub fn can_start_game(game: &GameSession) -> (bool, String) {
    if game.state != GameState::Lobby {
        return (false, "Game has already started".to_string());
    }

    if !game.can_start() {
        return (
            false,
            format!("Need at least {} players to start", MIN_PLAYERS),
        );
    }

    (true, String::new())
}

/// Check if voting phase can be started
///
/// # Arguments
///
/// * `game` - The game session
///
/// # Returns
///
/// Tuple of (can_start_voting, error_message)
pub fn can_start_voting(game: &GameSession) -> (bool, String) {
    if game.state != GameState::Playing {
        return (
            false,
            "Can only start voting from playing state".to_string(),
        );
    }

    let alive_count = game.players.values().filter(|p| p.is_alive).count();
    if alive_count < 2 {
        return (false, "Need at least 2 alive players to vote".to_string());
    }

    (true, String::new())
}

/// Transition game to voting phase
///
/// # Arguments
///
/// * `game` - The game session
pub fn transition_to_voting(game: &mut GameSession) {
    game.state = GameState::Voting;
    game.votes.clear(); // Clear any previous votes
}

/// Transition game back to playing phase
///
/// # Arguments
///
/// * `game` - The game session
pub fn transition_to_playing(game: &mut GameSession) {
    game.state = GameState::Playing;
    game.votes.clear();
}

/// Transition game to finished state
///
/// # Arguments
///
/// * `game` - The game session
/// * `winner` - "dragon" or "villagers"
pub fn transition_to_finished(game: &mut GameSession, winner: String) {
    game.state = GameState::Finished;
    game.winner = Some(winner);
    game.finished_at = Some(OffsetDateTime::now_utc());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::game_session::GameSession;

    fn create_test_game() -> GameSession {
        GameSession::new("test_game".to_string())
    }

    #[test]
    fn test_can_start_game_in_lobby_not_enough_players() {
        let game = create_test_game();

        let (can, err) = can_start_game(&game);

        assert!(!can);
        assert!(err.contains("Need at least"));
    }

    #[test]
    fn test_can_start_game_in_lobby_enough_players() {
        let mut game = create_test_game();

        // Add minimum players
        game.add_player("Alice".to_string()).unwrap();
        game.add_player("Bob".to_string()).unwrap();
        game.add_player("Charlie".to_string()).unwrap();

        let (can, err) = can_start_game(&game);

        assert!(can);
        assert_eq!(err, "");
    }

    #[test]
    fn test_can_start_game_already_started() {
        let mut game = create_test_game();

        game.add_player("Alice".to_string()).unwrap();
        game.add_player("Bob".to_string()).unwrap();
        game.add_player("Charlie".to_string()).unwrap();

        game.start_game().unwrap();

        let (can, err) = can_start_game(&game);

        assert!(!can);
        assert_eq!(err, "Game has already started");
    }

    #[test]
    fn test_can_start_voting_not_in_playing() {
        let game = create_test_game();

        let (can, err) = can_start_voting(&game);

        assert!(!can);
        assert_eq!(err, "Can only start voting from playing state");
    }

    #[test]
    fn test_can_start_voting_not_enough_alive() {
        let mut game = create_test_game();

        game.add_player("Alice".to_string()).unwrap();
        game.add_player("Bob".to_string()).unwrap();
        game.add_player("Charlie".to_string()).unwrap();

        game.start_game().unwrap();

        // Kill all but one player
        let player_ids: Vec<String> = game.players.keys().cloned().collect();
        for player_id in player_ids.iter().take(player_ids.len() - 1) {
            if let Some(player) = game.players.get_mut(player_id) {
                player.is_alive = false;
            }
        }

        let (can, err) = can_start_voting(&game);

        assert!(!can);
        assert_eq!(err, "Need at least 2 alive players to vote");
    }

    #[test]
    fn test_can_start_voting_success() {
        let mut game = create_test_game();

        game.add_player("Alice".to_string()).unwrap();
        game.add_player("Bob".to_string()).unwrap();
        game.add_player("Charlie".to_string()).unwrap();

        game.start_game().unwrap();

        let (can, err) = can_start_voting(&game);

        assert!(can);
        assert_eq!(err, "");
    }

    #[test]
    fn test_transition_to_voting() {
        let mut game = create_test_game();

        game.add_player("Alice".to_string()).unwrap();
        game.add_player("Bob".to_string()).unwrap();
        game.add_player("Charlie".to_string()).unwrap();

        game.start_game().unwrap();

        // Add some votes first
        let player_ids: Vec<String> = game.players.keys().cloned().collect();
        game.votes
            .insert(player_ids[0].clone(), player_ids[1].clone());

        transition_to_voting(&mut game);

        assert_eq!(game.state, GameState::Voting);
        assert!(game.votes.is_empty()); // Votes should be cleared
    }

    #[test]
    fn test_transition_to_playing() {
        let mut game = create_test_game();

        game.state = GameState::Voting;

        // Add some votes
        game.votes
            .insert("player1".to_string(), "player2".to_string());

        transition_to_playing(&mut game);

        assert_eq!(game.state, GameState::Playing);
        assert!(game.votes.is_empty());
    }

    #[test]
    fn test_transition_to_finished() {
        let mut game = create_test_game();

        game.state = GameState::Playing;

        transition_to_finished(&mut game, "dragon".to_string());

        assert_eq!(game.state, GameState::Finished);
        assert_eq!(game.winner, Some("dragon".to_string()));
        assert!(game.finished_at.is_some());
    }

    #[test]
    fn test_transition_to_finished_villagers() {
        let mut game = create_test_game();

        transition_to_finished(&mut game, "villagers".to_string());

        assert_eq!(game.state, GameState::Finished);
        assert_eq!(game.winner, Some("villagers".to_string()));
        assert!(game.finished_at.is_some());
    }
}
