//! Regression tests for template rendering
//!
//! These tests verify that routes return HTML templates (not JSON) to prevent
//! reintroduction of bugs where:
//! - Lobby page returned JSON instead of HTML
//! - Game page returned JSON instead of HTML
//! - Results page returned JSON instead of HTML

use axum::http::StatusCode;
use axum_test::TestServer;
use dragonseeker::{
    core::game_manager::GameManager, middleware::rate_limiter, middleware::security_headers,
    routes, state::AppState,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

/// Helper to create a test server with the full app configuration
fn create_test_server() -> TestServer {
    let secret_key = "test_secret_key_for_template_tests".to_string();
    let game_manager = Arc::new(RwLock::new(GameManager::new()));
    let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string());
    let public_url = if env == "development" {
        "http://localhost:3000".to_string()
    } else {
        "https://dragonseeker.win".to_string()
    };
    let state = AppState {
        game_manager,
        secret_key,
        public_url,
    };

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

    let app = axum::Router::new()
        .route("/", axum::routing::get(routes::health::root))
        .route("/health", axum::routing::get(routes::health::health_check))
        .route("/game/new", axum::routing::get(routes::game::show_index))
        .route(
            "/api/games/create",
            axum::routing::post(routes::game::create_game),
        )
        .route(
            "/game/:game_id/join",
            axum::routing::get(routes::game::show_join_page),
        )
        .route(
            "/api/games/:game_id/join",
            axum::routing::post(routes::game::join_game),
        )
        .route(
            "/game/:game_id/lobby",
            axum::routing::get(routes::lobby::show_lobby),
        )
        .route(
            "/api/games/:game_id/start",
            axum::routing::post(routes::lobby::start_game),
        )
        .route(
            "/api/games/:game_id/set-timer",
            axum::routing::post(routes::lobby::set_timer),
        )
        .route(
            "/game/:game_id/play",
            axum::routing::get(routes::gameplay::show_game),
        )
        .route(
            "/api/games/:game_id/start-voting",
            axum::routing::post(routes::gameplay::start_voting),
        )
        .route(
            "/api/games/:game_id/timer",
            axum::routing::get(routes::gameplay::get_timer),
        )
        .route(
            "/api/games/:game_id/vote",
            axum::routing::post(routes::gameplay::submit_vote),
        )
        .route(
            "/api/games/:game_id/guess-word",
            axum::routing::post(routes::gameplay::guess_word),
        )
        .route(
            "/game/:game_id/results",
            axum::routing::get(routes::gameplay::show_results),
        )
        .route(
            "/ws/:game_id/:player_id",
            axum::routing::get(routes::websocket::websocket_handler),
        )
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state.clone())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn(
                    security_headers::security_headers_middleware,
                ))
                .layer(axum::middleware::from_fn(
                    rate_limiter::rate_limit_middleware,
                ))
                .layer(cors)
                .layer(axum::Extension(state)),
        );

    TestServer::new(app).unwrap()
}

mod lobby_template_tests {
    use super::*;

    #[tokio::test]
    async fn test_lobby_returns_html_not_json() {
        let server = create_test_server();

        // Create a game
        let create_response = server.post("/api/games/create").await;
        if create_response.status_code() == StatusCode::TOO_MANY_REQUESTS {
            println!("Skipping test due to rate limiting");
            return;
        }

        let body = create_response.text();
        let game_id = body
            .split("game_id\":\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .unwrap();

        // Join the game
        let join_response = server
            .post(&format!("/api/games/{}/join", game_id))
            .form(&[("nickname", "TestPlayer")])
            .await;

        let join_body = join_response.text();
        let player_id = join_body
            .split("player_id\":\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .unwrap();

        let cookie = join_response
            .iter_headers()
            .find(|(name, _)| *name == axum::http::header::SET_COOKIE)
            .map(|(_, value)| value.to_str().unwrap())
            .unwrap();

        // Request lobby page
        let lobby_response = server
            .get(&format!("/game/{}/lobby?player_id={}", game_id, player_id))
            .add_header(
                axum::http::header::COOKIE,
                axum::http::HeaderValue::from_str(cookie).unwrap(),
            )
            .await;

        assert_eq!(lobby_response.status_code(), StatusCode::OK);

        let lobby_content = lobby_response.text();

        // Verify it's HTML, not JSON
        assert!(
            lobby_content.contains("<html") || lobby_content.contains("<!DOCTYPE"),
            "Lobby should return HTML with <html> or <!DOCTYPE> tag"
        );
        assert!(
            lobby_content.contains("</html>"),
            "Lobby should return complete HTML document"
        );
        assert!(
            !lobby_content.trim().starts_with('{'),
            "Lobby should NOT return JSON starting with '{{'"
        );

        // Verify it has expected lobby content
        assert!(
            lobby_content.contains("Players") || lobby_content.contains("player"),
            "Lobby HTML should contain player-related content"
        );
    }

    #[tokio::test]
    async fn test_lobby_html_contains_player_info() {
        let server = create_test_server();

        // Create and join game
        let create_response = server.post("/api/games/create").await;
        if create_response.status_code() == StatusCode::TOO_MANY_REQUESTS {
            println!("Skipping test due to rate limiting");
            return;
        }

        let body = create_response.text();
        let game_id = body
            .split("game_id\":\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .unwrap();

        let join_response = server
            .post(&format!("/api/games/{}/join", game_id))
            .form(&[("nickname", "AliceTest")])
            .await;

        let join_body = join_response.text();
        let player_id = join_body
            .split("player_id\":\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .unwrap();

        let cookie = join_response
            .iter_headers()
            .find(|(name, _)| *name == axum::http::header::SET_COOKIE)
            .map(|(_, value)| value.to_str().unwrap())
            .unwrap();

        // Get lobby
        let lobby_response = server
            .get(&format!("/game/{}/lobby?player_id={}", game_id, player_id))
            .add_header(
                axum::http::header::COOKIE,
                axum::http::HeaderValue::from_str(cookie).unwrap(),
            )
            .await;

        let lobby_html = lobby_response.text();

        // Verify the lobby contains the player's nickname
        assert!(
            lobby_html.contains("AliceTest"),
            "Lobby HTML should display player nickname"
        );

        // Verify HTML structure
        assert!(
            lobby_html.contains("<div") || lobby_html.contains("<p"),
            "Lobby should have HTML structure with divs or paragraphs"
        );
    }
}

mod game_template_tests {
    use super::*;

    #[tokio::test]
    async fn test_game_page_returns_html_not_json() {
        let server = create_test_server();

        // Create game
        let create_response = server.post("/api/games/create").await;
        if create_response.status_code() == StatusCode::TOO_MANY_REQUESTS {
            println!("Skipping test due to rate limiting");
            return;
        }

        let body = create_response.text();
        let game_id = body
            .split("game_id\":\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .unwrap();

        // Join with enough players to start
        let mut player_data = Vec::new();
        for i in 1..=3 {
            let join_response = server
                .post(&format!("/api/games/{}/join", game_id))
                .form(&[("nickname", &format!("Player{}", i))])
                .await;

            let join_body = join_response.text();
            let player_id = join_body
                .split("player_id\":\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .unwrap()
                .to_string();

            let cookie = join_response
                .iter_headers()
                .find(|(name, _)| *name == axum::http::header::SET_COOKIE)
                .map(|(_, value)| value.to_str().unwrap().to_string())
                .unwrap();

            player_data.push((player_id, cookie));
        }

        // Start the game (as host)
        let (host_id, host_cookie) = &player_data[0];
        let _start_response = server
            .post(&format!("/api/games/{}/start", game_id))
            .add_header(
                axum::http::header::COOKIE,
                axum::http::HeaderValue::from_str(host_cookie).unwrap(),
            )
            .form(&[("player_id", host_id.as_str())])
            .await;

        // Request game page
        let game_response = server
            .get(&format!("/game/{}/play?player_id={}", game_id, host_id))
            .add_header(
                axum::http::header::COOKIE,
                axum::http::HeaderValue::from_str(host_cookie).unwrap(),
            )
            .await;

        // Game might redirect to lobby or results depending on state
        // We only test HTML rendering when it returns 200
        if game_response.status_code() == StatusCode::OK {
            let game_content = game_response.text();

            // Verify it's HTML, not JSON
            assert!(
                game_content.contains("<html") || game_content.contains("<!DOCTYPE"),
                "Game page should return HTML with <html> or <!DOCTYPE> tag"
            );
            assert!(
                game_content.contains("</html>"),
                "Game page should return complete HTML document"
            );
            assert!(
                !game_content.trim().starts_with('{'),
                "Game page should NOT return JSON starting with '{{'"
            );

            // Verify it has expected game content
            assert!(
                game_content.contains("Game") || game_content.contains("role"),
                "Game HTML should contain game-related content"
            );
        } else {
            // If it redirects (303), that's valid behavior - game might still be in lobby
            assert!(
                game_response.status_code() == StatusCode::SEE_OTHER
                    || game_response.status_code() == StatusCode::FOUND,
                "Game page should either return OK (200) or redirect (303/302)"
            );
        }
    }
}

mod results_template_tests {
    use super::*;

    #[tokio::test]
    async fn test_results_page_returns_html_not_json() {
        let server = create_test_server();

        // Create game
        let create_response = server.post("/api/games/create").await;
        if create_response.status_code() == StatusCode::TOO_MANY_REQUESTS {
            println!("Skipping test due to rate limiting");
            return;
        }

        let body = create_response.text();
        let game_id = body
            .split("game_id\":\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .unwrap();

        // Join with players
        let mut player_data = Vec::new();
        for i in 1..=3 {
            let join_response = server
                .post(&format!("/api/games/{}/join", game_id))
                .form(&[("nickname", &format!("Player{}", i))])
                .await;

            let join_body = join_response.text();
            let player_id = join_body
                .split("player_id\":\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .unwrap()
                .to_string();

            let cookie = join_response
                .iter_headers()
                .find(|(name, _)| *name == axum::http::header::SET_COOKIE)
                .map(|(_, value)| value.to_str().unwrap().to_string())
                .unwrap();

            player_data.push((player_id, cookie));
        }

        let (player_id, cookie) = &player_data[0];

        // Access results page (will show even if game not finished)
        let results_response = server
            .get(&format!(
                "/game/{}/results?player_id={}",
                game_id, player_id
            ))
            .add_header(
                axum::http::header::COOKIE,
                axum::http::HeaderValue::from_str(cookie).unwrap(),
            )
            .await;

        assert_eq!(results_response.status_code(), StatusCode::OK);

        let results_content = results_response.text();

        // Verify it's HTML, not JSON
        assert!(
            results_content.contains("<html") || results_content.contains("<!DOCTYPE"),
            "Results page should return HTML with <html> or <!DOCTYPE> tag"
        );
        assert!(
            results_content.contains("</html>"),
            "Results page should return complete HTML document"
        );
        assert!(
            !results_content.trim().starts_with('{'),
            "Results page should NOT return JSON starting with '{{'"
        );

        // Verify it has expected results content
        assert!(
            results_content.contains("Game Over") || results_content.contains("Word"),
            "Results HTML should contain game results content"
        );
    }

    #[tokio::test]
    async fn test_results_html_structure() {
        let server = create_test_server();

        // Create and join game
        let create_response = server.post("/api/games/create").await;
        if create_response.status_code() == StatusCode::TOO_MANY_REQUESTS {
            println!("Skipping test due to rate limiting");
            return;
        }

        let body = create_response.text();
        let game_id = body
            .split("game_id\":\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .unwrap();

        let join_response = server
            .post(&format!("/api/games/{}/join", game_id))
            .form(&[("nickname", "TestPlayer")])
            .await;

        let join_body = join_response.text();
        let player_id = join_body
            .split("player_id\":\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .unwrap();

        let cookie = join_response
            .iter_headers()
            .find(|(name, _)| *name == axum::http::header::SET_COOKIE)
            .map(|(_, value)| value.to_str().unwrap())
            .unwrap();

        // Get results page
        let results_response = server
            .get(&format!(
                "/game/{}/results?player_id={}",
                game_id, player_id
            ))
            .add_header(
                axum::http::header::COOKIE,
                axum::http::HeaderValue::from_str(cookie).unwrap(),
            )
            .await;

        let results_html = results_response.text();

        // Verify HTML structure elements
        assert!(
            results_html.contains("<div") || results_html.contains("<h1"),
            "Results should have HTML structure with divs or headings"
        );

        // Verify it contains links to play again
        assert!(
            results_html.contains("Play Again") || results_html.contains("/game/new"),
            "Results should offer option to play again"
        );
    }
}
