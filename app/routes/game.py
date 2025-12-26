"""Routes for game creation and joining."""
from fastapi import APIRouter, HTTPException, Request
from fastapi.responses import RedirectResponse
from fastapi.templating import Jinja2Templates

from core.game_manager import game_manager
from models.requests import JoinGameRequest

router = APIRouter()
templates = Jinja2Templates(directory="templates")


@router.post("/api/games/create")
async def create_game():
    """Create a new game session.

    Returns:
        Redirect to the join page for the new game
    """
    game = game_manager.create_game()

    return RedirectResponse(
        url=f"/game/{game.game_id}/join",
        status_code=303
    )


@router.get("/game/{game_id}/join")
async def show_join_page(request: Request, game_id: str):
    """Show the join page where players enter their nickname.

    Args:
        request: The FastAPI request object
        game_id: The game session ID

    Returns:
        Rendered join page template

    Raises:
        HTTPException: If game not found
    """
    game = game_manager.get_game(game_id)

    if not game:
        raise HTTPException(status_code=404, detail="Game not found")

    if game.state.value != "lobby":
        raise HTTPException(status_code=400, detail="Game has already started")

    return templates.TemplateResponse("join.html", {
        "request": request,
        "game_id": game_id
    })


@router.post("/api/games/{game_id}/join")
async def join_game(game_id: str, join_request: JoinGameRequest):
    """Add a player to the game session.

    Args:
        game_id: The game session ID
        join_request: Player's nickname

    Returns:
        Redirect to lobby page with player_id

    Raises:
        HTTPException: If game not found or cannot join
    """
    game = game_manager.get_game(game_id)

    if not game:
        raise HTTPException(status_code=404, detail="Game not found")

    if game.state.value != "lobby":
        raise HTTPException(status_code=400, detail="Game has already started")

    # Check for duplicate nicknames
    if any(p.nickname.lower() == join_request.nickname.lower()
           for p in game.players.values()):
        raise HTTPException(status_code=400, detail="Nickname already taken")

    # Add player
    player = game.add_player(join_request.nickname)

    # Broadcast update to all connected players
    await game.broadcast_state()

    return RedirectResponse(
        url=f"/game/{game_id}/lobby?player_id={player.id}",
        status_code=303
    )
