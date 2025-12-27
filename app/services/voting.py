"""Voting service helpers."""

from core.game_session import GameSession, GameState


def can_vote(game: GameSession, player_id: str) -> tuple[bool, str | None]:
    """Check if a player can vote.

    Args:
        game: The game session
        player_id: ID of the player trying to vote

    Returns:
        Tuple of (can_vote, error_message)
    """
    if game.state != GameState.VOTING:
        return False, "Not in voting phase"

    player = game.players.get(player_id)
    if not player:
        return False, "Player not found"

    if not player.is_alive:
        return False, "Dead players cannot vote"

    if player_id in game.votes:
        return False, "Already voted"

    return True, None


def all_votes_submitted(game: GameSession) -> bool:
    """Check if all alive players have voted.

    Args:
        game: The game session

    Returns:
        True if all alive players have submitted votes
    """
    alive_count = sum(1 for p in game.players.values() if p.is_alive)
    return len(game.votes) >= alive_count
