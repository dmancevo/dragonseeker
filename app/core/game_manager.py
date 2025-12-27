"""Game manager singleton for coordinating multiple games."""

import secrets
from datetime import datetime, timedelta

from .constants import GAME_TTL_SECONDS
from .game_session import GameSession


class GameManager:
    """Singleton manager for all active game sessions."""

    def __init__(self):
        """Initialize the game manager."""
        self.games: dict[str, GameSession] = {}

    def create_game(self) -> GameSession:
        """Create a new game session with a unique ID.

        Returns:
            The newly created GameSession
        """
        # Generate a unique, URL-safe game ID (8 characters)
        game_id = secrets.token_urlsafe(6)

        # Ensure uniqueness (very unlikely to collide, but check anyway)
        while game_id in self.games:
            game_id = secrets.token_urlsafe(6)

        game = GameSession(game_id=game_id)
        self.games[game_id] = game
        return game

    def get_game(self, game_id: str) -> GameSession | None:
        """Retrieve a game session by ID.

        Args:
            game_id: The game's unique identifier

        Returns:
            The GameSession if found, None otherwise
        """
        return self.games.get(game_id)

    def remove_game(self, game_id: str) -> None:
        """Remove a game session.

        Args:
            game_id: The game's unique identifier
        """
        if game_id in self.games:
            del self.games[game_id]

    def cleanup_stale_games(self) -> int:
        """Remove games that are too old.

        Returns:
            Number of games cleaned up
        """
        now = datetime.now()
        cutoff_time = now - timedelta(seconds=GAME_TTL_SECONDS)

        stale_game_ids = [
            game_id for game_id, game in self.games.items() if game.created_at < cutoff_time
        ]

        for game_id in stale_game_ids:
            self.remove_game(game_id)

        return len(stale_game_ids)

    def get_stats(self) -> dict:
        """Get statistics about active games.

        Returns:
            Dictionary with game statistics
        """
        total_players = sum(len(game.players) for game in self.games.values())
        active_games = sum(1 for game in self.games.values() if game.state.value != "finished")

        return {
            "total_games": len(self.games),
            "active_games": active_games,
            "total_players": total_players,
        }


# Global singleton instance
game_manager = GameManager()
