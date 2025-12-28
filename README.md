# 🎮 Dragonseeker

A social deduction party game built with Rust, Axum, HTMX, Tailwind CSS, and DaisyUI. Players must work together to identify the Dragon among them!

**🎮 [Play Now at dragonseeker.win](https://dragonseeker.win/)**

## 🎯 Game Overview

**Dragonseeker** is a real-time multiplayer social deduction game where players are assigned secret roles:
- **Villagers**: Know the secret word and must identify the Dragon
- **Knights**: Know a similar (but different) word and must identify the Dragon. Knights don't know they are knights - they think they are villagers!
- **Dragon**: Doesn't know the word and must blend in to survive

### How to Play

1. **Create Game**: Host creates a new game and receives a unique shareable link
2. **Invite Players**: Share the link with 2-11 friends (3-12 players total)
3. **Join Lobby**: Players enter their nicknames to join the game
4. **Start Game**: Host starts when everyone has joined
5. **Roles Assigned**: Each player sees their role and word (Dragon sees "???")
6. **Say a Word**: Each player takes turns saying ONE word that is similar to their secret word (but NOT the actual word itself). The Dragon must try to blend in by guessing what the word might be!
7. **Discussion Phase**: Players discuss and analyze the words that were said to figure out who the Dragon is
8. **Voting Phase**: Host initiates voting, everyone votes to eliminate a player
9. **Win Conditions**:
   - **Villagers/Knights win** if they eliminate the Dragon
   - **Dragon wins** if they survive until ≤2 players remain OR correctly guess the word after elimination

## ✨ Features

- 🔗 **Private Game Links** - Each game gets a unique shareable URL
- ⚡ **Real-time Updates** - WebSocket-powered live game state
- ⚖️ **Auto-balanced Roles** - Fair distribution for 3-12 players
- 🗳️ **Voting System** - Democratic elimination with tie-breaker
- 🐉 **Dragon Guess Mechanic** - Last chance redemption
- 📱 **Mobile Responsive** - Works great on all devices
- 🎨 **Modern UI** - Beautiful interface with Tailwind + DaisyUI

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- Cargo (comes with Rust)

### Run Locally

```bash
cd app
cargo run
```

Then open your browser to: **http://localhost:8000**

### Production Mode

```bash
cd app
cargo build --release
./target/release/dragonseeker
```

## 🏗️ Architecture

### Tech Stack

- **Backend**: Rust + Axum 0.7 + Tokio 1.40
- **Frontend**: HTMX 2.0.4 + Tailwind CSS + DaisyUI 4.12
- **Templates**: Askama 0.12 (compile-time Jinja2-like templates)
- **WebSockets**: tokio-tungstenite 0.24
- **Deployment**: Docker + Digital Ocean App Platform

### Project Structure

```
app/
├── src/
│   ├── main.rs                # Application entry point
│   ├── lib.rs                 # Library exports
│   ├── state.rs               # AppState with Arc<RwLock<GameManager>>
│   │
│   ├── core/                  # Game logic
│   │   ├── mod.rs
│   │   ├── constants.rs      # Game settings & 500 word pairs
│   │   ├── player.rs         # Player model
│   │   ├── roles.rs          # Role assignment
│   │   ├── game_session.rs   # Game state machine
│   │   └── game_manager.rs   # Multi-game coordinator
│   │
│   ├── auth/                  # Authentication
│   │   ├── mod.rs
│   │   ├── token.rs          # HMAC-SHA256 tokens
│   │   └── middleware.rs     # Auth extractor
│   │
│   ├── middleware/            # Web middleware
│   │   ├── mod.rs
│   │   ├── rate_limiter.rs   # Per-IP rate limiting
│   │   └── security_headers.rs
│   │
│   ├── routes/                # HTTP handlers
│   │   ├── mod.rs
│   │   ├── game.rs           # Create/join game
│   │   ├── lobby.rs          # Lobby management
│   │   ├── gameplay.rs       # Voting & gameplay
│   │   └── websocket.rs      # WebSocket handler
│   │
│   ├── services/              # Business logic
│   │   ├── mod.rs
│   │   ├── voting.rs         # Vote tallying
│   │   ├── win_conditions.rs # Win detection
│   │   └── game_state.rs     # State transitions
│   │
│   └── models/                # DTOs
│       ├── mod.rs
│       ├── requests.rs       # Request DTOs
│       └── responses.rs      # Response DTOs
│
├── static/                    # Frontend assets
│   ├── css/custom.css        # Custom styles
│   ├── js/websocket-client.js # WebSocket manager
│   └── js/htmx-config.js     # HTMX setup
│
├── templates/                 # HTML templates
│   ├── base.html             # Base template
│   ├── index.html            # Landing page
│   ├── join.html             # Join page
│   ├── lobby.html            # Game lobby
│   ├── game.html             # Active game
│   └── results.html          # Game over
│
├── Cargo.toml                 # Project manifest
└── Cargo.lock                 # Dependency lock
```

### Game State Machine

```
LOBBY → (host starts) → PLAYING → (host votes) → VOTING
  → (all voted) → DRAGON_GUESS or FINISHED
  → (check win) → FINISHED or PLAYING
```

### Role Distribution

| Players | Dragon | Knights | Villagers |
|---------|--------|---------|-----------|
| 3-4     | 1      | 0       | 2-3       |
| 5-6     | 1      | 1       | 3-4       |
| 7-8     | 1      | 2       | 4-5       |
| 9-10    | 1      | 3       | 5-6       |
| 11-12   | 1      | 4       | 6-7       |

## 🎮 API Endpoints

### Game Management
- `GET /` - Landing page
- `POST /api/games/create` - Create new game
- `GET /game/{game_id}/join` - Join page
- `POST /api/games/{game_id}/join` - Join game

### Lobby
- `GET /game/{game_id}/lobby` - Lobby page
- `POST /api/games/{game_id}/start` - Start game (host only)

### Gameplay
- `GET /game/{game_id}/play` - Game interface
- `POST /api/games/{game_id}/start-voting` - Start voting phase
- `POST /api/games/{game_id}/vote` - Submit vote
- `POST /api/games/{game_id}/guess-word` - Dragon word guess
- `GET /game/{game_id}/results` - Results page

### WebSocket
- `WS /ws/{game_id}/{player_id}` - Real-time game updates

### Health
- `GET /health` - Health check endpoint

## 🚢 Deployment

### Docker

The project includes a Dockerfile for containerized deployment:

```bash
docker build -t dragonseeker .
docker run -p 8000:8000 dragonseeker
```

### Digital Ocean App Platform

The app is configured for automatic deployment:
1. Push to your GitHub repository
2. Digital Ocean auto-deploys on push to `main` branch
3. Configuration in `.do/app.yaml`

## 🧪 Development & Testing

This project uses Rust's built-in development tools for testing, linting, and type checking.

### Development Dependencies

All development dependencies are managed through Cargo and are automatically installed when you build the project.

### Running Tests

Run all tests with cargo:

```bash
cd app
cargo test              # Run all tests
cargo test -- --nocapture  # Show output
cargo test -- --test-threads=1  # Sequential execution
```

The test suite includes 119 tests covering:
- Core game logic (player, roles, game session, game manager)
- Authentication and middleware (HMAC tokens, rate limiting, security headers)
- Business logic (voting, win conditions, game state transitions)
- Route handlers and models

### Code Formatting

Format code with cargo fmt:

```bash
cd app
cargo fmt              # Format all Rust files
cargo fmt --check      # Check if formatted
```

### Linting

Check code quality with clippy:

```bash
cd app
cargo clippy                    # Check for issues
cargo clippy -- -D warnings     # Treat warnings as errors
cargo fix                       # Auto-fix issues
```

Clippy checks for:
- Common bugs and anti-patterns
- Performance issues
- Idiomatic Rust patterns
- Unsafe code usage
- Documentation quality

### Type Checking

Rust has compile-time type checking built-in. Simply run:

```bash
cd app
cargo check      # Check for type errors
cargo build      # Full compilation check
```

### Pre-commit Workflow

Before committing code, run:

```bash
cd app
cargo fmt --check         # Verify formatting
cargo clippy -- -D warnings  # Check for issues
cargo test                # Run all tests
cargo build --release     # Verify release build
```

### Logging

The server uses structured logging with different verbosity levels:

**Production (default)**: Only warnings and errors are logged
```bash
cargo run --release
```

**Development**: Enable verbose debug logging
```bash
ENVIRONMENT=development cargo run
```

You can also override the log level with the `RUST_LOG` environment variable:
```bash
RUST_LOG=debug cargo run        # Debug logs
RUST_LOG=info cargo run         # Info logs
RUST_LOG=warn cargo run         # Warn logs (production default)
```

## 🎨 Customization

### Adding Custom Words

Edit `app/src/core/constants.rs` to add or modify the word pairs:

```rust
pub const WORD_PAIRS: &[(&str, &str)] = &[
    ("elephant", "mammoth"),
    ("telescope", "binoculars"),
    // Add your word pairs here...
];
```

### Changing Game Settings

Modify `app/src/core/constants.rs`:

```rust
pub const MIN_PLAYERS: usize = 3;       // Minimum players to start
pub const MAX_PLAYERS: usize = 12;      // Maximum players allowed
pub const GAME_TTL_SECONDS: u64 = 3600; // Game cleanup time (1 hour)
```

## 🎯 Future Enhancements

Potential features to add:
- ⏱️ Discussion timer with countdown
- 💬 In-game text chat for remote play
- 📊 Game history and statistics
- 📚 Custom word lists by category
- 👀 Spectator mode
- 🔊 Sound effects and animations
- 📱 Progressive Web App (PWA) for mobile
- 🌐 Multi-language support
- 🎭 Custom role abilities (e.g., Knights get hints)

## 📝 License

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0). See [LICENSE.md](LICENSE.md) for details.

The AGPL-3.0 is a strong copyleft license that requires anyone who runs a modified version of this software as a network service to make the source code available to users of that service.

## 🤝 Contributing

Contributions are welcome! Feel free to:
- Report bugs
- Suggest new features
- Submit pull requests

## 🎮 Play Now!

**Play the live game at [dragonseeker.win](https://dragonseeker.win/)!**

Or run locally by starting the server and visiting http://localhost:8000:

```bash
cd app
cargo run
```

Enjoy the game! 🐉⚔️🏘️
