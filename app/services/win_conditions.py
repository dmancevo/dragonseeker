"""Win condition checking service."""

from core.game_session import GameSession
from core.roles import Role


def check_dragon_eliminated(game: GameSession) -> bool:
    """Check if the dragon has been eliminated.

    Args:
        game: The game session

    Returns:
        True if dragon is not alive
    """
    dragon = next((p for p in game.players.values() if p.role == Role.DRAGON.value), None)
    return dragon is not None and not dragon.is_alive


def check_dragon_survived(game: GameSession) -> bool:
    """Check if dragon has survived to win condition.

    Args:
        game: The game session

    Returns:
        True if dragon is alive and only 2 or fewer players remain
    """
    alive_players = [p for p in game.players.values() if p.is_alive]
    dragon = next((p for p in game.players.values() if p.role == Role.DRAGON.value), None)

    return dragon is not None and dragon.is_alive and len(alive_players) <= 2


def determine_winner(game: GameSession) -> str | None:
    """Determine the winner of the game.

    Args:
        game: The game session

    Returns:
        "dragon", "villagers", or None if game should continue
    """
    if check_dragon_eliminated(game):
        # Dragon has a chance to guess the word
        return None  # Will transition to DRAGON_GUESS state

    if check_dragon_survived(game):
        return "dragon"

    return None  # Game continues
