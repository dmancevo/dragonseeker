"""Tests for voting service."""

from services.voting import all_votes_submitted, can_vote


class TestCanVote:
    """Tests for can_vote function."""

    def test_can_vote_in_voting_phase(self, voting_game):
        """Test that alive players can vote during voting phase."""
        player_id = list(voting_game.players.keys())[0]
        can, error = can_vote(voting_game, player_id)
        assert can is True
        assert error is None

    def test_cannot_vote_when_not_in_voting_phase(self, started_game):
        """Test that players cannot vote when not in voting phase."""
        player_id = list(started_game.players.keys())[0]
        can, error = can_vote(started_game, player_id)
        assert can is False
        assert error == "Not in voting phase"

    def test_cannot_vote_when_player_not_found(self, voting_game):
        """Test that non-existent players cannot vote."""
        can, error = can_vote(voting_game, "non-existent-id")
        assert can is False
        assert error == "Player not found"

    def test_cannot_vote_when_player_dead(self, voting_game):
        """Test that dead players cannot vote."""
        player_id = list(voting_game.players.keys())[0]
        voting_game.players[player_id].is_alive = False
        can, error = can_vote(voting_game, player_id)
        assert can is False
        assert error == "Dead players cannot vote"

    def test_cannot_vote_twice(self, voting_game):
        """Test that players cannot vote twice."""
        player_id = list(voting_game.players.keys())[0]
        target_id = list(voting_game.players.keys())[1]

        # First vote should succeed
        voting_game.votes[player_id] = target_id

        # Second vote should fail
        can, error = can_vote(voting_game, player_id)
        assert can is False
        assert error == "Already voted"


class TestAllVotesSubmitted:
    """Tests for all_votes_submitted function."""

    def test_no_votes_submitted(self, voting_game):
        """Test when no votes have been submitted."""
        assert all_votes_submitted(voting_game) is False

    def test_partial_votes_submitted(self, voting_game):
        """Test when only some players have voted."""
        player_ids = list(voting_game.players.keys())
        voting_game.votes[player_ids[0]] = player_ids[1]
        voting_game.votes[player_ids[1]] = player_ids[0]
        # 2 out of 5 votes
        assert all_votes_submitted(voting_game) is False

    def test_all_votes_submitted(self, voting_game):
        """Test when all alive players have voted."""
        player_ids = list(voting_game.players.keys())
        # All 5 players vote
        for i, voter_id in enumerate(player_ids):
            target_id = player_ids[(i + 1) % len(player_ids)]
            voting_game.votes[voter_id] = target_id

        assert all_votes_submitted(voting_game) is True

    def test_all_votes_with_dead_players(self, voting_game):
        """Test that dead players are not counted for vote completion."""
        player_ids = list(voting_game.players.keys())

        # Kill one player
        voting_game.players[player_ids[0]].is_alive = False

        # Remaining 4 players vote
        for i in range(1, 5):
            target_id = player_ids[(i + 1) % 5]
            if target_id != player_ids[0]:  # Don't vote for dead player
                voting_game.votes[player_ids[i]] = target_id
            else:
                voting_game.votes[player_ids[i]] = player_ids[1]

        assert all_votes_submitted(voting_game) is True
