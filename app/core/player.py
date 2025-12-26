"""Player model for the game."""
from datetime import datetime
from typing import Optional
import uuid


class Player:
    """Represents a player in the game."""

    def __init__(self, nickname: str, is_host: bool = False):
        """Initialize a new player.

        Args:
            nickname: The player's display name
            is_host: Whether this player is the game host
        """
        self.id: str = str(uuid.uuid4())
        self.nickname: str = nickname
        self.role: Optional[str] = None  # Will be set when game starts
        self.is_alive: bool = True
        self.is_host: bool = is_host
        self.knows_word: bool = False  # False for Dragon, True for others
        self.joined_at: datetime = datetime.now()

    def to_dict(self, include_role: bool = False) -> dict:
        """Convert player to dictionary for API responses.

        Args:
            include_role: Whether to include the player's role (only for game end)

        Returns:
            Dictionary representation of the player
        """
        data = {
            "id": self.id,
            "nickname": self.nickname,
            "is_alive": self.is_alive,
            "is_host": self.is_host,
        }
        if include_role:
            data["role"] = self.role
        return data

    def __repr__(self) -> str:
        return f"Player(id={self.id}, nickname={self.nickname}, role={self.role})"
