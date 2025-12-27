"""Request models for API endpoints."""

from pydantic import BaseModel, Field, field_validator


class JoinGameRequest(BaseModel):
    """Request to join a game."""

    nickname: str = Field(..., min_length=1, max_length=20, description="Player's display name")

    @field_validator("nickname")
    @classmethod
    def nickname_must_be_clean(cls, v: str) -> str:
        """Validate and clean nickname."""
        v = v.strip()
        if not v:
            raise ValueError("Nickname cannot be empty")
        # Allow alphanumeric and spaces
        if not all(c.isalnum() or c.isspace() for c in v):
            raise ValueError("Nickname must contain only letters, numbers, and spaces")
        return v


class VoteRequest(BaseModel):
    """Request to vote for a player."""

    target_id: str = Field(..., description="ID of player to vote for")


class GuessWordRequest(BaseModel):
    """Dragon's word guess request."""

    guess: str = Field(..., min_length=1, max_length=50, description="The guessed word")

    @field_validator("guess")
    @classmethod
    def clean_guess(cls, v: str) -> str:
        """Clean the guess."""
        return v.strip().lower()
