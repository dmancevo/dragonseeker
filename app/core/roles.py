"""Role definitions and assignment logic."""

import random
from enum import Enum


class Role(str, Enum):
    """Player roles in the game."""

    VILLAGER = "villager"
    KNIGHT = "knight"
    DRAGON = "dragon"


def calculate_role_distribution(player_count: int) -> dict[Role, int]:
    """Calculate role distribution based on player count.

    Always 1 Dragon, rest split between Villagers and Knights.

    Distribution:
    - 3-4 players: 1 Dragon, 2-3 Villagers, 0 Knights
    - 5-6 players: 1 Dragon, 3-4 Villagers, 1 Knight
    - 7-8 players: 1 Dragon, 4-5 Villagers, 2 Knights
    - 9-10 players: 1 Dragon, 5-6 Villagers, 3 Knights
    - 11-12 players: 1 Dragon, 6-7 Villagers, 4 Knights

    Args:
        player_count: Number of players in the game

    Returns:
        Dictionary mapping Role to count

    Raises:
        ValueError: If player count is out of valid range
    """
    if player_count < 3:
        raise ValueError("Minimum 3 players required")
    if player_count > 12:
        raise ValueError("Maximum 12 players allowed")

    dragons = 1
    knights = max(0, (player_count - 3) // 2)
    villagers = player_count - dragons - knights

    return {Role.DRAGON: dragons, Role.KNIGHT: knights, Role.VILLAGER: villagers}


def assign_roles(players: list) -> None:
    """Randomly assign roles to players based on distribution.

    Modifies players in-place to set their role and knows_word attributes.

    Args:
        players: List of Player objects to assign roles to
    """

    distribution = calculate_role_distribution(len(players))

    # Create role pool
    role_pool = []
    for role, count in distribution.items():
        role_pool.extend([role] * count)

    # Shuffle and assign
    random.shuffle(role_pool)
    for player, role in zip(players, role_pool, strict=True):
        player.role = role.value
        player.knows_word = role != Role.DRAGON
