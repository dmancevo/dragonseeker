use crate::core::game_session::GameSession;

/// Check if the dragon has been eliminated
///
/// # Arguments
///
/// * `game` - The game session
///
/// # Returns
///
/// True if dragon is not alive
pub fn check_dragon_eliminated(game: &GameSession) -> bool {
    game.players
        .values()
        .find(|p| p.role.as_deref() == Some("dragon"))
        .is_some_and(|dragon| !dragon.is_alive)
}

/// Check if dragon has survived to win condition
///
/// # Arguments
///
/// * `game` - The game session
///
/// # Returns
///
/// True if dragon is alive and only 2 or fewer players remain
pub fn check_dragon_survived(game: &GameSession) -> bool {
    let alive_players: Vec<_> = game.players.values().filter(|p| p.is_alive).collect();

    let dragon = game
        .players
        .values()
        .find(|p| p.role.as_deref() == Some("dragon"));

    dragon.is_some_and(|d| d.is_alive && alive_players.len() <= 2)
}

/// Determine the winner of the game
///
/// # Arguments
///
/// * `game` - The game session
///
/// # Returns
///
/// "dragon", "villagers", or None if game should continue
pub fn determine_winner(game: &GameSession) -> Option<String> {
    if check_dragon_eliminated(game) {
        // Dragon has a chance to guess the word
        return None; // Will transition to DRAGON_GUESS state
    }

    if check_dragon_survived(game) {
        return Some("dragon".to_string());
    }

    None // Game continues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::game_session::GameSession;

    fn create_test_game_with_roles() -> GameSession {
        let mut game = GameSession::new("test_game".to_string());

        // Add 5 players and start game to assign roles
        game.add_player("Alice".to_string()).unwrap();
        game.add_player("Bob".to_string()).unwrap();
        game.add_player("Charlie".to_string()).unwrap();
        game.add_player("Diana".to_string()).unwrap();
        game.add_player("Eve".to_string()).unwrap();

        game.start_game().unwrap();

        game
    }

    #[test]
    fn test_check_dragon_eliminated_false() {
        let game = create_test_game_with_roles();

        // Dragon should be alive initially
        assert!(!check_dragon_eliminated(&game));
    }

    #[test]
    fn test_check_dragon_eliminated_true() {
        let mut game = create_test_game_with_roles();

        // Find and kill the dragon
        let dragon_id = game
            .players
            .iter()
            .find(|(_, p)| p.role.as_deref() == Some("dragon"))
            .map(|(id, _)| id.clone())
            .unwrap();

        if let Some(dragon) = game.players.get_mut(&dragon_id) {
            dragon.is_alive = false;
        }

        assert!(check_dragon_eliminated(&game));
    }

    #[test]
    fn test_check_dragon_survived_false_too_many_players() {
        let game = create_test_game_with_roles();

        // 5 players alive, should be false
        assert!(!check_dragon_survived(&game));
    }

    #[test]
    fn test_check_dragon_survived_false_dragon_dead() {
        let mut game = create_test_game_with_roles();

        // Kill the dragon
        let dragon_id = game
            .players
            .iter()
            .find(|(_, p)| p.role.as_deref() == Some("dragon"))
            .map(|(id, _)| id.clone())
            .unwrap();

        if let Some(dragon) = game.players.get_mut(&dragon_id) {
            dragon.is_alive = false;
        }

        // Kill other players until only 2 remain
        let mut non_dragon_players: Vec<String> = game
            .players
            .iter()
            .filter(|(_, p)| p.role.as_deref() != Some("dragon"))
            .map(|(id, _)| id.clone())
            .collect();

        // Kill 3 non-dragon players (leaving 2 total: 1 dead dragon + 1 alive other)
        for _ in 0..3 {
            if let Some(player_id) = non_dragon_players.pop() {
                if let Some(player) = game.players.get_mut(&player_id) {
                    player.is_alive = false;
                }
            }
        }

        assert!(!check_dragon_survived(&game));
    }

    #[test]
    fn test_check_dragon_survived_true() {
        let mut game = create_test_game_with_roles();

        // Find non-dragon players and kill enough to get to 2 alive
        let non_dragon_players: Vec<String> = game
            .players
            .iter()
            .filter(|(_, p)| p.role.as_deref() != Some("dragon"))
            .map(|(id, _)| id.clone())
            .collect();

        // Kill 3 players to get down to 2 alive (dragon + 1 other)
        for player_id in non_dragon_players.iter().take(3) {
            if let Some(player) = game.players.get_mut(player_id) {
                player.is_alive = false;
            }
        }

        assert!(check_dragon_survived(&game));
    }

    #[test]
    fn test_determine_winner_none_game_continues() {
        let game = create_test_game_with_roles();

        // Game just started, should continue
        assert_eq!(determine_winner(&game), None);
    }

    #[test]
    fn test_determine_winner_dragon_eliminated() {
        let mut game = create_test_game_with_roles();

        // Find and kill the dragon
        let dragon_id = game
            .players
            .iter()
            .find(|(_, p)| p.role.as_deref() == Some("dragon"))
            .map(|(id, _)| id.clone())
            .unwrap();

        if let Some(dragon) = game.players.get_mut(&dragon_id) {
            dragon.is_alive = false;
        }

        // Should return None to allow dragon guess
        assert_eq!(determine_winner(&game), None);
    }

    #[test]
    fn test_determine_winner_dragon_survived() {
        let mut game = create_test_game_with_roles();

        // Find non-dragon players and kill enough to get to 2 alive
        let non_dragon_players: Vec<String> = game
            .players
            .iter()
            .filter(|(_, p)| p.role.as_deref() != Some("dragon"))
            .map(|(id, _)| id.clone())
            .collect();

        // Kill 3 players to get down to 2 alive
        for player_id in non_dragon_players.iter().take(3) {
            if let Some(player) = game.players.get_mut(player_id) {
                player.is_alive = false;
            }
        }

        assert_eq!(determine_winner(&game), Some("dragon".to_string()));
    }

    #[test]
    fn test_determine_winner_exactly_2_players_dragon_alive() {
        let mut game = create_test_game_with_roles();

        // Kill all but 2 players (one being the dragon)
        let non_dragon_players: Vec<String> = game
            .players
            .iter()
            .filter(|(_, p)| p.role.as_deref() != Some("dragon"))
            .map(|(id, _)| id.clone())
            .collect();

        // Kill all but 1 non-dragon player
        for player_id in non_dragon_players.iter().take(non_dragon_players.len() - 1) {
            if let Some(player) = game.players.get_mut(player_id) {
                player.is_alive = false;
            }
        }

        assert_eq!(determine_winner(&game), Some("dragon".to_string()));
    }
}
