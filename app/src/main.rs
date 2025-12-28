use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use rand::Rng;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

use dragonseeker::{
    core::game_manager::GameManager,
    middleware::{rate_limiter, security_headers},
    routes::{game, gameplay, health, lobby, websocket},
    state::AppState,
};

#[tokio::main]
async fn main() {
    // Initialize tracing
    // Default to WARN level for production (shows warnings and errors only)
    // Set ENVIRONMENT=development to enable DEBUG logs
    let log_level = match std::env::var("ENVIRONMENT").as_deref() {
        Ok("development") | Ok("dev") => "dragonseeker=info,tower_http=warn",
        _ => "dragonseeker=info,tower_http=warn",
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_level.into()),
        )
        .init();

    println!("🐉 Dragonseeker game server starting...");

    // Generate secret key for token signing (64 hex characters = 32 bytes)
    let secret_key: String = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    println!("🔐 Generated secret key for token signing");

    // Initialize game manager
    let game_manager = Arc::new(RwLock::new(GameManager::new()));
    println!("🔗 Game manager initialized");

    // Set public url
    let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string());

    let public_url = if env != "development" {
        "https://dragonseeker.win".to_string()
    } else {
        "http://localhost:8000".to_string()
    };

    // Create application state
    let state = AppState {
        game_manager,
        secret_key,
        public_url,
    };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin([
            "https://dragonseeker.win".parse().unwrap(),
            "http://localhost:8000".parse().unwrap(),
            "http://127.0.0.1:8000".parse().unwrap(),
        ])
        .allow_credentials(true)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            "content-type".parse().unwrap(),
            "hx-request".parse().unwrap(),
            "hx-trigger".parse().unwrap(),
            "hx-target".parse().unwrap(),
            "hx-current-url".parse().unwrap(),
        ]);

    // Build router with all routes
    let app = Router::new()
        // Health and root
        .route("/", get(health::root))
        .route("/health", get(health::health_check))
        // Game creation and joining
        .route("/game/new", get(game::show_index))
        .route("/api/games/create", post(game::create_game))
        .route("/game/:game_id/join", get(game::show_join_page))
        .route("/api/games/:game_id/join", post(game::join_game))
        // Lobby
        .route("/game/:game_id/lobby", get(lobby::show_lobby))
        .route("/api/games/:game_id/start", post(lobby::start_game))
        .route("/api/games/:game_id/set-timer", post(lobby::set_timer))
        // Gameplay
        .route("/game/:game_id/play", get(gameplay::show_game))
        .route(
            "/api/games/:game_id/start-voting",
            post(gameplay::start_voting),
        )
        .route("/api/games/:game_id/timer", get(gameplay::get_timer))
        .route("/api/games/:game_id/vote", post(gameplay::submit_vote))
        .route("/api/games/:game_id/guess-word", post(gameplay::guess_word))
        .route("/game/:game_id/results", get(gameplay::show_results))
        // WebSocket
        .route("/ws/:game_id/:player_id", get(websocket::websocket_handler))
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        // Add state
        .with_state(state.clone())
        // Add middleware layers (applied in reverse order)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(
                    security_headers::security_headers_middleware,
                ))
                .layer(middleware::from_fn(rate_limiter::rate_limit_middleware))
                .layer(cors)
                .layer(axum::Extension(state)),
        );

    // Bind to address
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8000);
    let addr = format!("0.0.0.0:{}", port);

    println!("🚀 Server starting on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    println!("✅ Server listening on http://{}", addr);

    axum::serve(listener, app).await.expect("Server error");

    println!("👋 Shutting down game server...");
}
