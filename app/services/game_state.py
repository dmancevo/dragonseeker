"""Game state transition helpers."""
from core.game_session import GameSession, GameState


def can_start_game(game: GameSession) -> tuple[bool, str]:
    """Check if game can be started.

    Args:
        game: The game session

    Returns:
        Tuple of (can_start, error_message)
    """
    if game.state != GameState.LOBBY:
        return False, "Game has already started"

    if not game.can_start():
        return False, f"Need at least {game.can_start()} players to start"

    return True, ""


def can_start_voting(game: GameSession) -> tuple[bool, str]:
    """Check if voting phase can be started.

    Args:
        game: The game session

    Returns:
        Tuple of (can_start_voting, error_message)
    """
    if game.state != GameState.PLAYING:
        return False, "Can only start voting from playing state"

    alive_count = sum(1 for p in game.players.values() if p.is_alive)
    if alive_count < 2:
        return False, "Need at least 2 alive players to vote"

    return True, ""


def transition_to_voting(game: GameSession) -> None:
    """Transition game to voting phase.

    Args:
        game: The game session
    """
    game.state = GameState.VOTING
    game.votes.clear()  # Clear any previous votes


def transition_to_playing(game: GameSession) -> None:
    """Transition game back to playing phase.

    Args:
        game: The game session
    """
    game.state = GameState.PLAYING
    game.votes.clear()


def transition_to_finished(game: GameSession, winner: str) -> None:
    """Transition game to finished state.

    Args:
        game: The game session
        winner: "dragon" or "villagers"
    """
    game.state = GameState.FINISHED
    game.winner = winner
