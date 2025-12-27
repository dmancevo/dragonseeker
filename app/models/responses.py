"""Response models for API endpoints."""

from pydantic import BaseModel


class PlayerResponse(BaseModel):
    """Player information for API responses."""

    id: str
    nickname: str
    is_alive: bool
    is_host: bool
    role: str | None = None  # Only included in game-over state


class GameStateResponse(BaseModel):
    """Game state response."""

    game_id: str
    state: str
    players: list[PlayerResponse]
    player_count: int
    alive_count: int
    can_start: bool


class VoteResultResponse(BaseModel):
    """Vote tallying result."""

    eliminated_id: str
    eliminated_nickname: str
    eliminated_role: str
    vote_counts: dict[str, int]
    was_tie: bool


class GameResultResponse(BaseModel):
    """Final game result."""

    winner: str  # "dragon" or "villagers"
    word: str
    dragon_guess: str | None = None
    players: list[PlayerResponse]
