"""Game session management."""
from datetime import datetime
from enum import Enum
from typing import Optional
import random
import json
import asyncio

from fastapi import WebSocket

from .player import Player
from .roles import assign_roles, Role
from .constants import WORD_PAIRS, MIN_PLAYERS


class GameState(str, Enum):
    """Game state enum."""
    LOBBY = "lobby"
    PLAYING = "playing"
    VOTING = "voting"
    DRAGON_GUESS = "dragon_guess"
    FINISHED = "finished"


class GameSession:
    """Manages a single game session."""

    def __init__(self, game_id: str):
        """Initialize a new game session.

        Args:
            game_id: Unique identifier for this game
        """
        self.game_id: str = game_id
        self.players: dict[str, Player] = {}
        self.state: GameState = GameState.LOBBY
        self.villager_word: Optional[str] = None  # Word for villagers
        self.knight_word: Optional[str] = None    # Similar word for knights
        self.created_at: datetime = datetime.now()
        self.started_at: Optional[datetime] = None
        self.votes: dict[str, str] = {}  # voter_id -> target_id
        self.connections: dict[str, WebSocket] = {}  # player_id -> WebSocket
        self.winner: Optional[str] = None  # "villagers" or "dragon"
        self.dragon_guess: Optional[str] = None
        self.eliminated_player_id: Optional[str] = None
        self.last_elimination: Optional[dict] = None  # Stores last elimination details

    def add_player(self, nickname: str) -> Player:
        """Add a new player to the game.

        Args:
            nickname: The player's display name

        Returns:
            The newly created Player object

        Raises:
            ValueError: If game is not in lobby state
        """
        if self.state != GameState.LOBBY:
            raise ValueError("Cannot join game that has already started")

        is_host = len(self.players) == 0  # First player is host
        player = Player(nickname=nickname, is_host=is_host)
        self.players[player.id] = player
        return player

    def remove_player(self, player_id: str) -> None:
        """Remove a player from the game.

        Args:
            player_id: ID of the player to remove
        """
        if player_id in self.players:
            del self.players[player_id]

        if player_id in self.connections:
            del self.connections[player_id]

        # If host left, assign new host
        if self.players and not any(p.is_host for p in self.players.values()):
            next_player = next(iter(self.players.values()))
            next_player.is_host = True

    def can_start(self) -> bool:
        """Check if the game can be started.

        Returns:
            True if minimum players requirement is met
        """
        return len(self.players) >= MIN_PLAYERS

    def start_game(self) -> None:
        """Start the game by assigning roles and selecting a word.

        Raises:
            ValueError: If game cannot be started
        """
        if not self.can_start():
            raise ValueError(f"Need at least {MIN_PLAYERS} players to start")

        if self.state != GameState.LOBBY:
            raise ValueError("Game has already started")

        # Assign roles
        players_list = list(self.players.values())
        assign_roles(players_list)

        # Select random word pair
        word_pair = random.choice(WORD_PAIRS)
        self.villager_word = word_pair[0]  # Main word for villagers
        self.knight_word = word_pair[1]    # Similar word for knights

        # Update state
        self.state = GameState.PLAYING
        self.started_at = datetime.now()

    def submit_vote(self, voter_id: str, target_id: str) -> None:
        """Submit a vote to eliminate a player.

        Args:
            voter_id: ID of the player voting
            target_id: ID of the player being voted for

        Raises:
            ValueError: If voting is not allowed
        """
        if self.state != GameState.VOTING:
            raise ValueError("Not in voting phase")

        voter = self.players.get(voter_id)
        target = self.players.get(target_id)

        if not voter or not voter.is_alive:
            raise ValueError("Voter is not alive or doesn't exist")

        if not target or not target.is_alive:
            raise ValueError("Cannot vote for dead or non-existent player")

        self.votes[voter_id] = target_id

    def tally_votes(self) -> dict:
        """Tally votes and determine eliminated player.

        Returns:
            Dictionary with vote results
        """
        from collections import Counter

        if not self.votes:
            return {"eliminated": None, "vote_counts": {}}

        # Count votes
        vote_counts = Counter(self.votes.values())
        max_votes = max(vote_counts.values())

        # Get all players with max votes (for tie-breaking)
        tied_players = [pid for pid, count in vote_counts.items()
                        if count == max_votes]

        # Random tie-breaker
        eliminated_id = random.choice(tied_players)
        eliminated_player = self.players[eliminated_id]
        eliminated_player.is_alive = False
        self.eliminated_player_id = eliminated_id

        # Store elimination details for display
        self.last_elimination = {
            "eliminated_id": eliminated_id,
            "eliminated_nickname": eliminated_player.nickname,
            "eliminated_role": eliminated_player.role,
            "vote_counts": dict(vote_counts),
            "was_tie": len(tied_players) > 1
        }

        return self.last_elimination

    def check_win_condition(self) -> Optional[str]:
        """Check if game has reached a win condition.

        Returns:
            "villagers", "dragon", or None if game continues
        """
        alive_players = [p for p in self.players.values() if p.is_alive]
        dragon = next((p for p in self.players.values() if p.role == Role.DRAGON.value), None)

        # Dragon was eliminated
        if dragon and not dragon.is_alive:
            # Give dragon a chance to guess the word
            return None  # Will transition to DRAGON_GUESS state

        # Only 2 players left and Dragon is alive
        if len(alive_players) <= 2 and dragon and dragon.is_alive:
            return "dragon"

        return None  # Game continues

    def get_state_for_player(self, player_id: str) -> dict:
        """Get game state customized for a specific player.

        Args:
            player_id: ID of the player to get state for

        Returns:
            Dictionary with game state
        """
        player = self.players.get(player_id)
        if not player:
            return {}

        alive_count = sum(1 for p in self.players.values() if p.is_alive)

        # Determine which word to show based on role
        your_word = None
        if player.knows_word:
            if player.role == Role.KNIGHT.value:
                your_word = self.knight_word
            else:  # Villager
                your_word = self.villager_word

        state_data = {
            "game_id": self.game_id,
            "state": self.state.value,
            "your_id": player_id,
            "your_role": player.role,
            "your_word": your_word,
            "is_host": player.is_host,
            "is_alive": player.is_alive,
            "players": [p.to_dict() for p in self.players.values()],
            "player_count": len(self.players),
            "alive_count": alive_count,
            "can_start": self.can_start(),
            "votes_submitted": len(self.votes),
            "has_voted": player_id in self.votes,
            "last_elimination": self.last_elimination,
        }

        if self.state == GameState.FINISHED:
            state_data["winner"] = self.winner
            state_data["villager_word"] = self.villager_word
            state_data["knight_word"] = self.knight_word
            state_data["dragon_guess"] = self.dragon_guess
            state_data["players"] = [p.to_dict(include_role=True) for p in self.players.values()]

        return state_data

    async def broadcast_state(self) -> None:
        """Broadcast current game state to all connected players."""
        print(f"📢 Broadcasting state for game {self.game_id} to {len(self.connections)} connections")
        print(f"   Game state: {self.state.value}")

        disconnected = []

        for player_id, websocket in self.connections.items():
            try:
                state = self.get_state_for_player(player_id)
                message = json.dumps({
                    "type": "state_update",
                    "data": state
                })
                await websocket.send_text(message)
                print(f"   ✅ Sent to {player_id}")
            except Exception as e:
                print(f"   ❌ Failed to send to {player_id}: {e}")
                # Mark for removal if send fails
                disconnected.append(player_id)

        # Remove disconnected players
        for player_id in disconnected:
            del self.connections[player_id]
            print(f"   🗑️ Removed disconnected player: {player_id}")

    def __repr__(self) -> str:
        return f"GameSession(id={self.game_id}, state={self.state}, players={len(self.players)})"
