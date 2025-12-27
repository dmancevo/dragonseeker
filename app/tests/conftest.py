"""Pytest configuration and fixtures."""

import pytest

from core.game_session import GameSession
from core.player import Player


@pytest.fixture
def game_session():
    """Create a basic game session for testing."""
    return GameSession(game_id="test-game-123")


@pytest.fixture
def game_with_players():
    """Create a game session with 5 players (enough to start)."""
    game = GameSession(game_id="test-game-456")
    players = []
    for i in range(5):
        player = game.add_player(f"Player{i + 1}")
        players.append(player)
    return game, players


@pytest.fixture
def started_game():
    """Create a game session that has been started with roles assigned."""
    game = GameSession(game_id="test-game-789")
    for i in range(5):
        game.add_player(f"Player{i + 1}")
    game.start_game()
    return game


@pytest.fixture
def voting_game():
    """Create a game in voting state."""
    game = GameSession(game_id="test-game-voting")
    for i in range(5):
        game.add_player(f"Player{i + 1}")
    game.start_game()
    # Transition to voting manually
    from core.game_session import GameState

    game.state = GameState.VOTING
    return game


@pytest.fixture
def sample_player():
    """Create a single player for testing."""
    return Player(nickname="TestPlayer", is_host=True)
