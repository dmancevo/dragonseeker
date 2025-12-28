use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::Player;

/// Player roles in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Villager,
    Knight,
    Dragon,
}

impl Role {
    /// Get the string representation of the role
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Villager => "villager",
            Role::Knight => "knight",
            Role::Dragon => "dragon",
        }
    }
}

/// Calculate role distribution based on player count
///
/// Always 1 Dragon, rest split between Villagers and Knights.
///
/// Distribution:
/// - 3-4 players: 1 Dragon, 2-3 Villagers, 0 Knights
/// - 5-6 players: 1 Dragon, 3-4 Villagers, 1 Knight
/// - 7-8 players: 1 Dragon, 4-5 Villagers, 2 Knights
/// - 9-10 players: 1 Dragon, 5-6 Villagers, 3 Knights
/// - 11-12 players: 1 Dragon, 6-7 Villagers, 4 Knights
///
/// # Arguments
///
/// * `player_count` - Number of players in the game
///
/// # Returns
///
/// HashMap mapping Role to count
///
/// # Errors
///
/// Returns an error if player count is out of valid range (3-12)
pub fn calculate_role_distribution(player_count: usize) -> Result<HashMap<Role, usize>, String> {
    if player_count < 3 {
        return Err("Minimum 3 players required".to_string());
    }
    if player_count > 12 {
        return Err("Maximum 12 players allowed".to_string());
    }

    let dragons = 1;
    let knights = if player_count >= 3 {
        (player_count - 3) / 2
    } else {
        0
    };
    let villagers = player_count - dragons - knights;

    tracing::debug!(
        "Role distribution for {} players: {} dragon, {} knights, {} villagers",
        player_count, dragons, knights, villagers
    );

    let mut distribution = HashMap::new();
    distribution.insert(Role::Dragon, dragons);
    distribution.insert(Role::Knight, knights);
    distribution.insert(Role::Villager, villagers);

    Ok(distribution)
}

/// Randomly assign roles to players based on distribution
///
/// Modifies players in-place to set their role and knows_word attributes.
///
/// # Arguments
///
/// * `players` - Mutable slice of Player objects to assign roles to
///
/// # Errors
///
/// Returns an error if player count is out of valid range
pub fn assign_roles(players: &mut [Player]) -> Result<(), String> {
    let player_count = players.len();

    // Validate player count
    if player_count < 3 {
        return Err("Minimum 3 players required".to_string());
    }
    if player_count > 12 {
        return Err("Maximum 12 players allowed".to_string());
    }

    // Calculate role counts
    let dragons = 1;
    let knights = (player_count - 3) / 2;
    let villagers = player_count - dragons - knights;

    tracing::debug!(
        "Role distribution for {} players: {} dragon, {} knights, {} villagers",
        player_count, dragons, knights, villagers
    );

    // Build role pool array directly with correct proportions
    let mut role_pool = Vec::with_capacity(player_count);

    // Add 1 dragon
    role_pool.push(Role::Dragon);

    // Add knights
    for _ in 0..knights {
        role_pool.push(Role::Knight);
    }

    // Add villagers
    for _ in 0..villagers {
        role_pool.push(Role::Villager);
    }

    // Shuffle the role pool
    let mut rng = thread_rng();
    role_pool.shuffle(&mut rng);

    // Sanity check
    if role_pool.len() != player_count {
        return Err(format!(
            "Role pool size {} does not match player count {}",
            role_pool.len(),
            player_count
        ));
    }

    // Assign roles to players
    for (player, role) in players.iter_mut().zip(role_pool.iter()) {
        player.role = Some(role.as_str().to_string());
        player.knows_word = *role != Role::Dragon;
        tracing::debug!(
            "Assigned role {} to player {}",
            role.as_str(),
            player.nickname
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_distribution_3_players() {
        let dist = calculate_role_distribution(3).unwrap();
        assert_eq!(dist[&Role::Dragon], 1);
        assert_eq!(dist[&Role::Knight], 0);
        assert_eq!(dist[&Role::Villager], 2);
    }

    #[test]
    fn test_role_distribution_4_players() {
        let dist = calculate_role_distribution(4).unwrap();
        assert_eq!(dist[&Role::Dragon], 1);
        assert_eq!(dist[&Role::Knight], 0);
        assert_eq!(dist[&Role::Villager], 3);
    }

    #[test]
    fn test_role_distribution_5_players() {
        let dist = calculate_role_distribution(5).unwrap();
        assert_eq!(dist[&Role::Dragon], 1);
        assert_eq!(dist[&Role::Knight], 1);
        assert_eq!(dist[&Role::Villager], 3);
    }

    #[test]
    fn test_role_distribution_6_players() {
        let dist = calculate_role_distribution(6).unwrap();
        assert_eq!(dist[&Role::Dragon], 1);
        assert_eq!(dist[&Role::Knight], 1);
        assert_eq!(dist[&Role::Villager], 4);
    }

    #[test]
    fn test_role_distribution_7_players() {
        let dist = calculate_role_distribution(7).unwrap();
        assert_eq!(dist[&Role::Dragon], 1);
        assert_eq!(dist[&Role::Knight], 2);
        assert_eq!(dist[&Role::Villager], 4);
    }

    #[test]
    fn test_role_distribution_8_players() {
        let dist = calculate_role_distribution(8).unwrap();
        assert_eq!(dist[&Role::Dragon], 1);
        assert_eq!(dist[&Role::Knight], 2);
        assert_eq!(dist[&Role::Villager], 5);
    }

    #[test]
    fn test_role_distribution_9_players() {
        let dist = calculate_role_distribution(9).unwrap();
        assert_eq!(dist[&Role::Dragon], 1);
        assert_eq!(dist[&Role::Knight], 3);
        assert_eq!(dist[&Role::Villager], 5);
    }

    #[test]
    fn test_role_distribution_10_players() {
        let dist = calculate_role_distribution(10).unwrap();
        assert_eq!(dist[&Role::Dragon], 1);
        assert_eq!(dist[&Role::Knight], 3);
        assert_eq!(dist[&Role::Villager], 6);
    }

    #[test]
    fn test_role_distribution_11_players() {
        let dist = calculate_role_distribution(11).unwrap();
        assert_eq!(dist[&Role::Dragon], 1);
        assert_eq!(dist[&Role::Knight], 4);
        assert_eq!(dist[&Role::Villager], 6);
    }

    #[test]
    fn test_role_distribution_12_players() {
        let dist = calculate_role_distribution(12).unwrap();
        assert_eq!(dist[&Role::Dragon], 1);
        assert_eq!(dist[&Role::Knight], 4);
        assert_eq!(dist[&Role::Villager], 7);
    }

    #[test]
    fn test_role_distribution_too_few_players() {
        let result = calculate_role_distribution(2);
        assert!(result.is_err());
    }

    #[test]
    fn test_role_distribution_too_many_players() {
        let result = calculate_role_distribution(13);
        assert!(result.is_err());
    }

    #[test]
    fn test_assign_roles() {
        let mut players = vec![
            Player::new("Player1".to_string(), true),
            Player::new("Player2".to_string(), false),
            Player::new("Player3".to_string(), false),
            Player::new("Player4".to_string(), false),
            Player::new("Player5".to_string(), false),
        ];

        let result = assign_roles(&mut players);
        assert!(result.is_ok());

        // Check that roles are assigned
        for player in &players {
            assert!(player.role.is_some());
        }

        // Count roles
        let mut dragon_count = 0;
        let mut knight_count = 0;
        let mut villager_count = 0;

        for player in &players {
            match player.role.as_ref().unwrap().as_str() {
                "dragon" => {
                    dragon_count += 1;
                    assert!(!player.knows_word);
                }
                "knight" => {
                    knight_count += 1;
                    assert!(player.knows_word);
                }
                "villager" => {
                    villager_count += 1;
                    assert!(player.knows_word);
                }
                _ => panic!("Unknown role"),
            }
        }

        assert_eq!(dragon_count, 1);
        assert_eq!(knight_count, 1);
        assert_eq!(villager_count, 3);
    }

    #[test]
    fn test_role_as_str() {
        assert_eq!(Role::Villager.as_str(), "villager");
        assert_eq!(Role::Knight.as_str(), "knight");
        assert_eq!(Role::Dragon.as_str(), "dragon");
    }
}
