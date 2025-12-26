"""WebSocket routes for real-time game updates."""
from fastapi import APIRouter, WebSocket, WebSocketDisconnect
import json

from core.game_manager import game_manager

router = APIRouter()


@router.websocket("/ws/{game_id}/{player_id}")
async def websocket_endpoint(websocket: WebSocket, game_id: str, player_id: str):
    """WebSocket endpoint for real-time game updates.

    Args:
        websocket: The WebSocket connection
        game_id: The game session ID
        player_id: The player's ID

    Flow:
        1. Validate game and player
        2. Accept WebSocket connection
        3. Register connection in game session
        4. Send initial state to player
        5. Keep connection alive with ping/pong
        6. Remove connection on disconnect
    """
    game = game_manager.get_game(game_id)

    # Validate game exists and player is in game
    if not game:
        await websocket.close(code=4004, reason="Game not found")
        return

    if player_id not in game.players:
        await websocket.close(code=4004, reason="Player not in game")
        return

    # Accept connection
    await websocket.accept()

    # Register WebSocket connection
    game.connections[player_id] = websocket

    try:
        # Send initial state to player
        initial_state = game.get_state_for_player(player_id)
        await websocket.send_text(json.dumps({
            "type": "state_update",
            "data": initial_state
        }))

        # Keep connection alive and handle messages
        while True:
            try:
                # Receive messages (mainly for keep-alive pings)
                data = await websocket.receive_text()

                # Handle ping/pong
                if data == "ping":
                    await websocket.send_text("pong")

            except WebSocketDisconnect:
                break

    except Exception as e:
        print(f"WebSocket error for player {player_id}: {e}")

    finally:
        # Remove connection when disconnected
        if player_id in game.connections:
            del game.connections[player_id]
