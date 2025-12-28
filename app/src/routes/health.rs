use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};

/// Root path - redirect to index page
///
/// # Returns
///
/// Redirect to /game/new
pub async fn root() -> Redirect {
    Redirect::to("/game/new")
}

/// Health check endpoint
///
/// # Returns
///
/// JSON response with status
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let app = Router::new().route("/health", axum::routing::get(health_check));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_root_redirect() {
        let app = Router::new().route("/", axum::routing::get(root));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
    }
}
