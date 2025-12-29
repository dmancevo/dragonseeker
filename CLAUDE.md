# Claude Developer Guide

This document provides context for AI assistants working on the Dragonseeker codebase.

## Project Overview

**Dragonseeker** is a real-time multiplayer social deduction game built with Rust, Axum, HTMX, Tailwind CSS, and DaisyUI.

### Tech Stack

- **Backend**: Axum 0.7 + Tokio 1.40
- **Frontend**: HTMX 2.0.4 + Tailwind CSS + DaisyUI 4.12
- **Templates**: Askama 0.12 (compile-time Jinja2-like templates)
- **WebSockets**: tokio-tungstenite 0.24
- **Package Manager**: Cargo
- **Testing**: cargo test
- **Linting**: cargo clippy
- **Formatting**: cargo fmt

## Project Structure

```
app/
├── Cargo.toml                 # Project manifest and dependencies
├── Cargo.lock                 # Dependency lock file
├── src/
│   ├── main.rs               # Application entry point
│   ├── lib.rs                # Library exports for testing
│   ├── state.rs              # Shared AppState with Arc<RwLock<GameManager>>
│   │
│   ├── core/                 # Core game logic
│   │   ├── mod.rs
│   │   ├── constants.rs     # Game settings, MIN_PLAYERS=3, MAX_PLAYERS=12, 500 WORD_PAIRS
│   │   ├── player.rs        # Player model with id, nickname, role, is_alive, is_host
│   │   ├── roles.rs         # Role assignment logic (Dragon, Knight, Villager)
│   │   ├── game_session.rs  # Game state machine (Lobby → Playing → Voting → DragonGuess → Finished)
│   │   └── game_manager.rs  # Multi-game coordinator
│   │
│   ├── auth/                 # Authentication
│   │   ├── mod.rs
│   │   ├── token.rs         # HMAC-SHA256 token generation/verification
│   │   └── middleware.rs    # Axum extractor for authenticated routes
│   │
│   ├── middleware/           # Web middleware
│   │   ├── mod.rs
│   │   ├── rate_limiter.rs  # Per-IP rate limiting with sliding window
│   │   └── security_headers.rs # CSP, X-Frame-Options, HSTS headers
│   │
│   ├── routes/               # HTTP handlers
│   │   ├── mod.rs
│   │   ├── game.rs          # Create/join game endpoints
│   │   ├── lobby.rs         # Lobby management and game start
│   │   ├── gameplay.rs      # Voting, word guessing, game logic
│   │   └── websocket.rs     # WebSocket handler for real-time updates
│   │
│   ├── models/               # DTOs (Data Transfer Objects)
│   │   ├── mod.rs
│   │   ├── requests.rs      # Request body structures
│   │   └── responses.rs     # Response body structures
│   │
│   └── services/             # Business logic helpers
│       ├── mod.rs
│       ├── voting.rs        # Vote validation and tallying
│       ├── win_conditions.rs # Win detection logic
│       └── game_state.rs    # State transition helpers
│
├── static/                   # Frontend assets
│   ├── css/custom.css       # Custom styles
│   ├── js/websocket-client.js # WebSocket manager
│   └── js/htmx-config.js    # HTMX configuration
│
└── templates/                # Askama HTML templates
    ├── base.html            # Base template
    ├── index.html           # Landing page
    ├── join.html            # Join page
    ├── lobby.html           # Game lobby
    ├── game.html            # Active game
    └── results.html         # Game over
```

## Key Concepts

### Game Roles

1. **Villagers**: Know the secret word, must identify the Dragon
2. **Knights**: Know a similar (but different) word, think they are villagers, must identify the Dragon
3. **Dragon**: Doesn't know any word, must blend in to survive or guess the word if eliminated

### Role Distribution

| Players | Dragon | Knights | Villagers |
|---------|--------|---------|-----------|
| 3-4     | 1      | 0       | 2-3       |
| 5-6     | 1      | 1       | 3-4       |
| 7-8     | 1      | 2       | 4-5       |
| 9-10    | 1      | 3       | 5-6       |
| 11-12   | 1      | 4       | 6-7       |

### Game State Machine

```
Lobby → (host starts) → Playing → (host initiates voting) → Voting
  ↓                                                            ↓
  ↓ (players vote)                                             ↓
  ↓                                                            ↓
  → DragonGuess (if dragon eliminated) → Finished
  → Finished (if dragon survives with ≤2 players)
  → Playing (if game continues)
```

### Win Conditions

- **Villagers/Knights win**: Dragon is eliminated AND fails to guess the word
- **Dragon wins**:
  - Survives until ≤2 players remain, OR
  - Gets eliminated but correctly guesses the villager word

## Development Workflow

### Setup

```bash
cd app
cargo build
```

#### Environment Variables

Create a `.env` file in the project root for local development:

```bash
# Generate a cryptographically secure secret key
openssl rand -hex 32

# Create .env file with the generated secret
cat > .env << EOF
# Environment configuration for local development
# DO NOT COMMIT THIS FILE - It contains sensitive secrets

# Secret key for HMAC token signing (64 hex characters)
# Generated with: openssl rand -hex 32
SECRET=<paste-your-generated-secret-here>

# Environment mode
ENVIRONMENT=development
EOF
```

**Note**: The `.env` file is already in `.gitignore` and will not be committed.

**For Production (Digital Ocean)**: Set the `SECRET` environment variable in your app settings with the same format.

### Running the Server

```bash
cd app
cargo run              # Development mode
cargo run --release    # Production mode (optimized)
```

### Testing

```bash
cd app
cargo test                      # Run all tests
cargo test -- --nocapture       # Show output
cargo test -- --test-threads=1  # Sequential execution
```

### Code Quality

```bash
cd app
cargo fmt                   # Format code
cargo fmt --check           # Check if formatted
cargo clippy                # Check for issues
cargo clippy -- -D warnings # Treat warnings as errors
cargo fix                   # Auto-fix issues
```

### Pre-commit Checklist

Before committing, always run:
```bash
cd app
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

All commands should pass with no errors.

## Important Patterns

### 1. Shared State Pattern

All routes use shared `AppState` with `Arc<RwLock<GameManager>>`:

```rust
#[derive(Clone)]
pub struct AppState {
    pub game_manager: Arc<RwLock<GameManager>>,
    pub secret_key: String,
}

// In routes:
async fn handler(State(state): State<AppState>) {
    let manager = state.game_manager.read().await;
    let game = manager.get_game(&game_id)?;
    // Use read lock for queries
    drop(manager);

    let mut manager = state.game_manager.write().await;
    let game = manager.get_game_mut(&game_id)?;
    // Use write lock for mutations
}
```

### 2. WebSocket Broadcasting

Game state updates are broadcast via `tokio::sync::broadcast` channel:

```rust
// In GameSession:
pub broadcast_tx: broadcast::Sender<String>

// In routes (after state change):
let state_json = serde_json::to_string(&game.get_state_for_player(&player_id))?;
let _ = game.broadcast_tx.send(state_json);

// In WebSocket handler:
let mut rx = game.broadcast_tx.subscribe();
while let Ok(msg) = rx.recv().await {
    sender.send(Message::Text(msg)).await?;
}
```

### 3. Authentication Pattern

Player-specific cookies with HMAC-SHA256 tokens:

```rust
use crate::auth::middleware::AuthenticatedPlayer;

async fn protected_route(
    State(state): State<AppState>,
    auth: AuthenticatedPlayer,  // Automatically extracts and verifies token
) {
    // Token is verified, auth.game_id and auth.player_id are available
    auth.verify_matches(&game_id, &player_id)?;
}
```

### 4. HTMX Redirect Headers

Routes return `HX-Redirect` headers for client-side navigation:

```rust
let mut headers = HeaderMap::new();
headers.insert(
    "HX-Redirect",
    format!("/game/{}/lobby?player_id={}", game_id, player_id)
        .parse()
        .unwrap(),
);

Ok((headers, Json(response)))
```

### 5. Error Handling

Use `Result<impl IntoResponse, (StatusCode, String)>` for route handlers:

```rust
async fn handler(...) -> Result<impl IntoResponse, (StatusCode, String)> {
    let game = manager.get_game(&game_id)
        .ok_or((StatusCode::NOT_FOUND, "Game not found".to_string()))?;

    game.start_game()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(response))
}
```

## Common Tasks

### Adding a New Word Pair

Edit `app/src/core/constants.rs`:

```rust
pub const WORD_PAIRS: &[(&str, &str)] = &[
    ("elephant", "mammoth"),  // (villager_word, knight_word)
    // Add new pairs here
];
```

### Adding a New Test

1. Add test in appropriate module with `#[cfg(test)]`
2. Use `#[tokio::test]` for async tests
3. Follow Arrange-Act-Assert pattern

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_works() {
        // Arrange
        let game = GameSession::new("test123".to_string());

        // Act
        let result = game.some_operation();

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_feature() {
        // Async test
    }
}
```

### Adding a New Route

1. Create handler function in appropriate router file (`routes/`)
2. Use proper type annotations with `Result<impl IntoResponse, (StatusCode, String)>`
3. Add authentication if needed with `AuthenticatedPlayer` extractor
4. Handle errors with proper status codes
5. Broadcast state updates if game state changes
6. Register route in `src/main.rs`:

```rust
let app = Router::new()
    .route("/api/games/:game_id/new-endpoint", post(routes::handler))
    .with_state(state);
```

## Configuration Files

### Cargo.toml

Contains:
- Project metadata
- Dependencies (axum, tokio, askama, etc.)
- Dev dependencies (tokio-test, axum-test)
- Build configuration

Key dependencies:
- `tokio = { version = "1.40", features = ["full"] }` - Async runtime
- `axum = { version = "0.7", features = ["ws", "macros"] }` - Web framework
- `askama = { version = "0.12", features = ["with-axum"] }` - Templates
- `time = { version = "0.3", features = ["serde", "macros"] }` - Time handling (NOT chrono)
- `hmac = "0.12"` and `sha2 = "0.10"` - Authentication
- `serde = { version = "1.0", features = ["derive"] }` - Serialization

## Testing Strategy

### Test Organization

- Unit tests in `#[cfg(test)]` modules within source files
- Tests cover 119 test cases across all modules
- All async code uses `#[tokio::test]` attribute

### Test Coverage Areas

- **Core logic**: Player, Roles, GameSession, GameManager (37 tests)
- **Authentication**: Token generation, verification, middleware (5 tests)
- **Middleware**: Rate limiting, security headers (12 tests)
- **Services**: Voting, win conditions, game state (27 tests)
- **Models**: Request/response DTOs (15 tests)
- **Routes**: Handler function tests (23 tests)

## Code Conventions

1. **Documentation**: All public functions have doc comments with `///`
2. **Type Annotations**: Explicit types on function signatures
3. **Error Handling**: Use `Result` with proper error types
4. **Naming**: snake_case for functions/variables, PascalCase for types
5. **Formatting**: 4 spaces indentation, enforced by `cargo fmt`
6. **Imports**: Organized by std, external crates, local modules

## Common Pitfalls

1. **Don't forget to broadcast state**: After game state changes, always broadcast updates
2. **Check game state**: Validate game is in correct state before operations
3. **Validate player permissions**: Check if player is host, alive, etc.
4. **Null checks**: Always check `Option` values before unwrapping
5. **Lock management**: Don't hold read/write locks longer than necessary
6. **WebSocket storage**: Never store WebSocket connections in GameSession - use broadcast channel
7. **Time crate**: Use `time` crate, NOT `chrono` (security vulnerability in chrono)

## Deployment

### Docker

Multi-stage build for optimized production image:

```bash
docker build -t dragonseeker .
docker run -p 8000:8000 dragonseeker
```

The Dockerfile:
- Stage 1: Builds Rust binary with rust:1.83-alpine
- Stage 2: Copies binary to minimal alpine:3.21 runtime image
- Final image size: ~20MB (vs ~50-100MB Python)

### Digital Ocean App Platform

- Auto-deploys on push to `main` branch
- Configuration in `.do/app.yaml`
- Uses Dockerfile for build

## Useful Commands

```bash
# Run server in development mode
cargo run

# Run server in release mode (optimized)
cargo run --release

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Check code without building
cargo check

# Build release binary
cargo build --release

# Format code
cargo fmt

# Check if code is formatted
cargo fmt --check

# Lint code
cargo clippy

# Auto-fix lint issues
cargo fix

# Run security audit
cargo audit

# Update dependencies
cargo update

# Generate documentation
cargo doc --open
```

## Resources

- [Axum Documentation](https://docs.rs/axum/)
- [Tokio Documentation](https://docs.rs/tokio/)
- [Askama Documentation](https://docs.rs/askama/)
- [HTMX Documentation](https://htmx.org/)
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/)

## Notes for AI Assistants

- Always run tests after making changes
- Use existing test patterns from `#[cfg(test)]` modules
- Follow Rust naming conventions (snake_case, not camelCase)
- Check that all quality tools pass before considering work complete
- When adding features, update tests to maintain coverage
- Preserve HTMX patterns for frontend interactivity
- Remember that WebSocket updates are crucial for real-time gameplay
- Use `Arc<RwLock<T>>` for shared mutable state, not global statics
- Prefer `?` operator over `.unwrap()` in route handlers
- Always use `time` crate instead of `chrono` for date/time operations
