use crate::core::game_session::{GameSession, GameState};

/// Check if a player can vote
///
/// # Arguments
///
/// * `game` - The game session
/// * `player_id` - ID of the player trying to vote
///
/// # Returns
///
/// Tuple of (can_vote, error_message)
pub fn can_vote(game: &GameSession, player_id: &str) -> (bool, Option<String>) {
    if game.state != GameState::Voting {
        return (false, Some("Not in voting phase".to_string()));
    }

    let player = match game.players.get(player_id) {
        Some(p) => p,
        None => return (false, Some("Player not found".to_string())),
    };

    if !player.is_alive {
        return (false, Some("Dead players cannot vote".to_string()));
    }

    if game.votes.contains_key(player_id) {
        return (false, Some("Already voted".to_string()));
    }

    (true, None)
}

/// Check if all alive players have voted
///
/// # Arguments
///
/// * `game` - The game session
///
/// # Returns
///
/// True if all alive players have submitted votes
pub fn all_votes_submitted(game: &GameSession) -> bool {
    let alive_count = game.players.values().filter(|p| p.is_alive).count();
    game.votes.len() >= alive_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::game_session::GameSession;

    fn create_test_game() -> GameSession {
        GameSession::new("test_game".to_string())
    }

    fn add_test_player(game: &mut GameSession, nickname: &str) -> String {
        let player = game.add_player(nickname.to_string()).unwrap();
        player.id.clone()
    }

    #[test]
    fn test_can_vote_not_in_voting_phase() {
        let game = create_test_game();
        let (can, err) = can_vote(&game, "player1");

        assert!(!can);
        assert_eq!(err, Some("Not in voting phase".to_string()));
    }

    #[test]
    fn test_can_vote_player_not_found() {
        let mut game = create_test_game();
        game.state = GameState::Voting;

        let (can, err) = can_vote(&game, "nonexistent");

        assert!(!can);
        assert_eq!(err, Some("Player not found".to_string()));
    }

    #[test]
    fn test_can_vote_dead_player() {
        let mut game = create_test_game();
        let player_id = add_test_player(&mut game, "Alice");

        // Mark player as dead
        if let Some(player) = game.players.get_mut(&player_id) {
            player.is_alive = false;
        }

        game.state = GameState::Voting;

        let (can, err) = can_vote(&game, &player_id);

        assert!(!can);
        assert_eq!(err, Some("Dead players cannot vote".to_string()));
    }

    #[test]
    fn test_can_vote_already_voted() {
        let mut game = create_test_game();
        let player_id = add_test_player(&mut game, "Alice");
        let target_id = add_test_player(&mut game, "Bob");

        game.state = GameState::Voting;
        game.votes.insert(player_id.clone(), target_id);

        let (can, err) = can_vote(&game, &player_id);

        assert!(!can);
        assert_eq!(err, Some("Already voted".to_string()));
    }

    #[test]
    fn test_can_vote_success() {
        let mut game = create_test_game();
        let player_id = add_test_player(&mut game, "Alice");

        game.state = GameState::Voting;

        let (can, err) = can_vote(&game, &player_id);

        assert!(can);
        assert_eq!(err, None);
    }

    #[test]
    fn test_all_votes_submitted_no_votes() {
        let mut game = create_test_game();
        add_test_player(&mut game, "Alice");
        add_test_player(&mut game, "Bob");

        assert!(!all_votes_submitted(&game));
    }

    #[test]
    fn test_all_votes_submitted_partial() {
        let mut game = create_test_game();
        let player1 = add_test_player(&mut game, "Alice");
        let player2 = add_test_player(&mut game, "Bob");
        add_test_player(&mut game, "Charlie");

        game.votes.insert(player1, player2);

        assert!(!all_votes_submitted(&game));
    }

    #[test]
    fn test_all_votes_submitted_complete() {
        let mut game = create_test_game();
        let player1 = add_test_player(&mut game, "Alice");
        let player2 = add_test_player(&mut game, "Bob");
        let player3 = add_test_player(&mut game, "Charlie");

        game.votes.insert(player1.clone(), player2.clone());
        game.votes.insert(player2.clone(), player3.clone());
        game.votes.insert(player3.clone(), player1.clone());

        assert!(all_votes_submitted(&game));
    }

    #[test]
    fn test_all_votes_submitted_with_dead_players() {
        let mut game = create_test_game();
        let player1 = add_test_player(&mut game, "Alice");
        let player2 = add_test_player(&mut game, "Bob");
        let player3 = add_test_player(&mut game, "Charlie");

        // Mark player3 as dead
        if let Some(player) = game.players.get_mut(&player3) {
            player.is_alive = false;
        }

        // Only 2 alive players, need 2 votes
        game.votes.insert(player1.clone(), player2.clone());
        game.votes.insert(player2.clone(), player1.clone());

        assert!(all_votes_submitted(&game));
    }
}
