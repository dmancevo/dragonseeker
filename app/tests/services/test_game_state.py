"""Tests for game state service."""

from core.game_session import GameState
from services.game_state import (
    can_start_game,
    can_start_voting,
    transition_to_finished,
    transition_to_playing,
    transition_to_voting,
)


class TestCanStartGame:
    """Tests for can_start_game function."""

    def test_can_start_with_enough_players(self, game_with_players):
        """Test that game can start with minimum required players."""
        game, _ = game_with_players
        can_start, error = can_start_game(game)
        assert can_start is True
        assert error == ""

    def test_cannot_start_with_too_few_players(self, game_session):
        """Test that game cannot start without enough players."""
        # Add only 2 players (minimum is 3)
        game_session.add_player("Player1")
        game_session.add_player("Player2")

        can_start, error = can_start_game(game_session)
        assert can_start is False
        assert "Need at least" in error

    def test_cannot_start_already_started_game(self, started_game):
        """Test that already started game cannot be started again."""
        can_start, error = can_start_game(started_game)
        assert can_start is False
        assert error == "Game has already started"


class TestCanStartVoting:
    """Tests for can_start_voting function."""

    def test_can_start_voting_from_playing_state(self, started_game):
        """Test that voting can start from playing state with enough alive players."""
        can_start, error = can_start_voting(started_game)
        assert can_start is True
        assert error == ""

    def test_cannot_start_voting_from_lobby(self, game_with_players):
        """Test that voting cannot start from lobby state."""
        game, _ = game_with_players
        can_start, error = can_start_voting(game)
        assert can_start is False
        assert error == "Can only start voting from playing state"

    def test_cannot_start_voting_with_too_few_alive_players(self, started_game):
        """Test that voting cannot start with fewer than 2 alive players."""
        # Kill all but 1 player
        player_ids = list(started_game.players.keys())
        for i, player_id in enumerate(player_ids):
            if i > 0:  # Keep first player alive
                started_game.players[player_id].is_alive = False

        can_start, error = can_start_voting(started_game)
        assert can_start is False
        assert error == "Need at least 2 alive players to vote"


class TestTransitionToVoting:
    """Tests for transition_to_voting function."""

    def test_transition_to_voting(self, started_game):
        """Test transitioning to voting phase."""
        # Add some votes first to verify they get cleared
        player_ids = list(started_game.players.keys())
        started_game.votes[player_ids[0]] = player_ids[1]

        transition_to_voting(started_game)

        assert started_game.state == GameState.VOTING
        assert len(started_game.votes) == 0

    def test_votes_cleared_on_transition(self, started_game):
        """Test that previous votes are cleared when transitioning."""
        player_ids = list(started_game.players.keys())
        started_game.votes[player_ids[0]] = player_ids[1]
        started_game.votes[player_ids[1]] = player_ids[2]

        transition_to_voting(started_game)

        assert started_game.votes == {}


class TestTransitionToPlaying:
    """Tests for transition_to_playing function."""

    def test_transition_to_playing(self, voting_game):
        """Test transitioning back to playing phase."""
        # Add some votes
        player_ids = list(voting_game.players.keys())
        voting_game.votes[player_ids[0]] = player_ids[1]

        transition_to_playing(voting_game)

        assert voting_game.state == GameState.PLAYING
        assert len(voting_game.votes) == 0

    def test_votes_cleared_on_transition_to_playing(self, voting_game):
        """Test that votes are cleared when transitioning to playing."""
        player_ids = list(voting_game.players.keys())
        voting_game.votes[player_ids[0]] = player_ids[1]
        voting_game.votes[player_ids[1]] = player_ids[2]

        transition_to_playing(voting_game)

        assert voting_game.votes == {}


class TestTransitionToFinished:
    """Tests for transition_to_finished function."""

    def test_transition_to_finished_villagers_win(self, started_game):
        """Test transitioning to finished state with villagers winning."""
        transition_to_finished(started_game, "villagers")

        assert started_game.state == GameState.FINISHED
        assert started_game.winner == "villagers"

    def test_transition_to_finished_dragon_wins(self, started_game):
        """Test transitioning to finished state with dragon winning."""
        transition_to_finished(started_game, "dragon")

        assert started_game.state == GameState.FINISHED
        assert started_game.winner == "dragon"
