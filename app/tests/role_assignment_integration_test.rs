use dragonseeker::core::game_manager::GameManager;

#[cfg(test)]
mod role_assignment_integration_tests {
    use super::*;

    /// Test that verifies role assignment in a real game scenario
    /// This catches issues that unit tests miss
    #[test]
    fn test_five_player_game_has_correct_roles() {
        // Create a game manager
        let mut manager = GameManager::new();

        // Create a game
        let game_id = manager.create_game();
        let game = manager.get_game_mut(&game_id).unwrap();

        // Add 5 players (first player is automatically host)
        for i in 1..=5 {
            let player_name = format!("Player{}", i);
            game.add_player(player_name).unwrap();
        }

        assert_eq!(game.players.len(), 5, "Should have 5 players");

        // Start the game
        game.start_game().unwrap();

        // Count roles by iterating through players
        let mut dragon_count = 0;
        let mut knight_count = 0;
        let mut villager_count = 0;
        let mut knight_player_id: Option<String> = None;
        let mut villager_player_id: Option<String> = None;

        for (player_id, player) in &game.players {
            match player.role.as_deref() {
                Some("dragon") => {
                    dragon_count += 1;
                    assert!(!player.knows_word, "Dragon should not know word");
                }
                Some("knight") => {
                    knight_count += 1;
                    assert!(player.knows_word, "Knight should know word");
                    knight_player_id = Some(player_id.clone());
                }
                Some("villager") => {
                    villager_count += 1;
                    assert!(player.knows_word, "Villager should know word");
                    villager_player_id = Some(player_id.clone());
                }
                _ => panic!("Unknown or missing role for player {}", player_id),
            }
        }

        // Assert correct distribution for 5 players
        assert_eq!(
            dragon_count, 1,
            "Should have exactly 1 dragon, found {}",
            dragon_count
        );
        assert_eq!(
            knight_count, 1,
            "Should have exactly 1 knight, found {}",
            knight_count
        );
        assert_eq!(
            villager_count, 3,
            "Should have exactly 3 villagers, found {}",
            villager_count
        );

        // Verify that words are assigned
        assert!(
            game.villager_word.is_some(),
            "Villager word should be set"
        );
        assert!(game.knight_word.is_some(), "Knight word should be set");

        // Verify that knight word is different from villager word
        assert_ne!(
            game.villager_word, game.knight_word,
            "Knight word should be different from villager word"
        );

        // Verify knight and villager get different words via get_state_for_player
        if let (Some(knight_id), Some(villager_id)) = (knight_player_id, villager_player_id) {
            let knight_state = game.get_state_for_player(&knight_id);
            let villager_state = game.get_state_for_player(&villager_id);

            let knight_word = knight_state["your_word"].as_str();
            let villager_word = villager_state["your_word"].as_str();

            assert!(knight_word.is_some(), "Knight should have a word");
            assert!(villager_word.is_some(), "Villager should have a word");
            assert_ne!(
                knight_word, villager_word,
                "Knight and villager should see different words"
            );

            println!(
                "   Knight sees: '{}', Villager sees: '{}'",
                knight_word.unwrap(),
                villager_word.unwrap()
            );
        } else {
            panic!("Could not find knight or villager player IDs");
        }

        println!("✅ 5-player game role assignment verified:");
        println!("   - 1 Dragon");
        println!("   - 1 Knight");
        println!("   - 3 Villagers");
        println!("   - Knight sees different word than villagers");
    }

    /// Test role assignment for different player counts
    #[test]
    fn test_role_distribution_for_all_player_counts() {
        for player_count in 3..=12 {
            let mut manager = GameManager::new();
            let game_id = manager.create_game();
            let game = manager.get_game_mut(&game_id).unwrap();

            // Add players
            for i in 1..=player_count {
                let player_name = format!("Player{}", i);
                game.add_player(player_name).unwrap();
            }

            // Start the game
            game.start_game().unwrap();

            // Count roles
            let mut dragon_count = 0;
            let mut knight_count = 0;
            let mut villager_count = 0;

            for player in game.players.values() {
                match player.role.as_deref() {
                    Some("dragon") => dragon_count += 1,
                    Some("knight") => knight_count += 1,
                    Some("villager") => villager_count += 1,
                    _ => panic!("Unknown role in game with {} players", player_count),
                }
            }

            // Expected distribution
            let expected_dragons = 1;
            let expected_knights = (player_count - 3) / 2;
            let expected_villagers = player_count - expected_dragons - expected_knights;

            assert_eq!(
                dragon_count, expected_dragons,
                "Wrong dragon count for {} players",
                player_count
            );
            assert_eq!(
                knight_count, expected_knights,
                "Wrong knight count for {} players",
                player_count
            );
            assert_eq!(
                villager_count, expected_villagers,
                "Wrong villager count for {} players",
                player_count
            );

            println!(
                "✅ {}: {} players -> {} dragon, {} knights, {} villagers",
                player_count, player_count, dragon_count, knight_count, villager_count
            );
        }
    }

    /// Test that role assignment is random but distribution is consistent
    #[test]
    fn test_role_assignment_is_random_but_counts_consistent() {
        let mut role_positions = std::collections::HashMap::new();

        // Run 20 games and track which position gets which role
        for _ in 0..20 {
            let mut manager = GameManager::new();
            let game_id = manager.create_game();
            let game = manager.get_game_mut(&game_id).unwrap();

            // Add 5 players
            for i in 1..=5 {
                game.add_player(format!("Player{}", i)).unwrap();
            }

            // Start game
            game.start_game().unwrap();

            // Track which position (index) got dragon
            let mut player_list: Vec<_> = game.players.values().collect();
            player_list.sort_by(|a, b| a.nickname.cmp(&b.nickname));

            for (idx, player) in player_list.iter().enumerate() {
                if player.role.as_deref() == Some("dragon") {
                    *role_positions.entry(idx).or_insert(0) += 1;
                }
            }

            // Verify counts
            let knight_count = player_list
                .iter()
                .filter(|p| p.role.as_deref() == Some("knight"))
                .count();
            assert_eq!(knight_count, 1, "Should always have 1 knight");
        }

        // Dragon position should vary (not always the same player)
        // With 20 runs and 5 positions, if it's truly random, dragon
        // shouldn't appear in the same position more than ~10 times
        let max_dragon_in_one_position = role_positions.values().max().unwrap_or(&0);
        assert!(
            *max_dragon_in_one_position < 15,
            "Dragon appears too often in one position ({}), roles may not be shuffled properly",
            max_dragon_in_one_position
        );

        println!("✅ Role assignment is random across 20 games:");
        for (pos, count) in &role_positions {
            println!("   Position {} was dragon {} times", pos, count);
        }
    }
}
