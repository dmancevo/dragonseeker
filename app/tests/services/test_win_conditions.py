"""Tests for win conditions service."""

from core.roles import Role
from services.win_conditions import (
    check_dragon_eliminated,
    check_dragon_survived,
    determine_winner,
)


class TestCheckDragonEliminated:
    """Tests for check_dragon_eliminated function."""

    def test_dragon_eliminated(self, started_game):
        """Test when dragon has been eliminated."""
        # Find and kill the dragon
        dragon = next(p for p in started_game.players.values() if p.role == Role.DRAGON.value)
        dragon.is_alive = False

        assert check_dragon_eliminated(started_game) is True

    def test_dragon_alive(self, started_game):
        """Test when dragon is still alive."""
        assert check_dragon_eliminated(started_game) is False

    def test_all_players_alive(self, started_game):
        """Test when all players including dragon are alive."""
        # Verify all players are alive
        for player in started_game.players.values():
            player.is_alive = True

        assert check_dragon_eliminated(started_game) is False


class TestCheckDragonSurvived:
    """Tests for check_dragon_survived function."""

    def test_dragon_survived_with_two_players(self, started_game):
        """Test when dragon is alive and exactly 2 players remain."""
        # Find dragon and another player
        dragon = next(p for p in started_game.players.values() if p.role == Role.DRAGON.value)
        other_player = next(p for p in started_game.players.values() if p.id != dragon.id)

        # Kill all players first
        for player in started_game.players.values():
            player.is_alive = False

        # Keep only dragon and one other player alive
        dragon.is_alive = True
        other_player.is_alive = True

        assert check_dragon_survived(started_game) is True

    def test_dragon_survived_with_one_player(self, started_game):
        """Test when only dragon remains (edge case)."""
        # Kill all except dragon
        dragon = next(p for p in started_game.players.values() if p.role == Role.DRAGON.value)
        for player in started_game.players.values():
            if player.id != dragon.id:
                player.is_alive = False

        assert check_dragon_survived(started_game) is True

    def test_dragon_not_survived_with_three_players(self, started_game):
        """Test when more than 2 players are alive."""
        # Ensure at least 3 players are alive
        alive_count = 0
        for player in started_game.players.values():
            if alive_count < 3:
                player.is_alive = True
                alive_count += 1
            else:
                player.is_alive = False

        # Dragon should be alive
        dragon = next(p for p in started_game.players.values() if p.role == Role.DRAGON.value)
        dragon.is_alive = True

        assert check_dragon_survived(started_game) is False

    def test_dragon_not_survived_when_dead(self, started_game):
        """Test when dragon is dead regardless of player count."""
        # Find and kill dragon
        dragon = next(p for p in started_game.players.values() if p.role == Role.DRAGON.value)
        dragon.is_alive = False

        # Kill others to leave only 2 alive (but dragon is dead)
        alive_count = 0
        for player in started_game.players.values():
            if player.id != dragon.id and alive_count >= 2:
                player.is_alive = False
            elif player.id != dragon.id:
                alive_count += 1

        assert check_dragon_survived(started_game) is False


class TestDetermineWinner:
    """Tests for determine_winner function."""

    def test_dragon_eliminated_returns_none(self, started_game):
        """Test that eliminating dragon returns None (for guess phase)."""
        dragon = next(p for p in started_game.players.values() if p.role == Role.DRAGON.value)
        dragon.is_alive = False

        # Should return None to allow dragon to guess
        assert determine_winner(started_game) is None

    def test_dragon_wins_when_survived(self, started_game):
        """Test that dragon wins when surviving to 2 or fewer players."""
        # Find dragon and another player
        dragon = next(p for p in started_game.players.values() if p.role == Role.DRAGON.value)
        other_player = next(p for p in started_game.players.values() if p.id != dragon.id)

        # Kill all players first
        for player in started_game.players.values():
            player.is_alive = False

        # Keep only dragon and one other player alive
        dragon.is_alive = True
        other_player.is_alive = True

        assert determine_winner(started_game) == "dragon"

    def test_game_continues(self, started_game):
        """Test when game should continue (no winner yet)."""
        # Ensure all players are alive and game is in normal state
        for player in started_game.players.values():
            player.is_alive = True

        assert determine_winner(started_game) is None
