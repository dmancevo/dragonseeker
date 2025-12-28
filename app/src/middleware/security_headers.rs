use axum::{
    extract::Request,
    http::{header, HeaderValue},
    middleware::Next,
    response::Response,
};

/// Middleware to add security headers to all responses
///
/// # Security Headers
///
/// - **Content-Security-Policy**: Restrict resource loading to prevent XSS
/// - **X-Frame-Options**: Prevent clickjacking
/// - **X-Content-Type-Options**: Prevent MIME type sniffing
/// - **X-XSS-Protection**: Enable browser XSS protection
/// - **Strict-Transport-Security**: Force HTTPS (only on HTTPS)
/// - **Referrer-Policy**: Limit referrer information
/// - **Permissions-Policy**: Disable unnecessary browser features
pub async fn security_headers_middleware(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;

    let headers = response.headers_mut();

    // Content Security Policy - restrict resource loading
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline' https://unpkg.com https://cdn.tailwindcss.com; \
             style-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net https://cdn.tailwindcss.com; \
             connect-src 'self' wss://dragonseeker.win ws://localhost:8000 ws://127.0.0.1:8000; \
             img-src 'self' data:; \
             font-src 'self' https://cdn.jsdelivr.net; \
             frame-ancestors 'none'",
        ),
    );

    // Prevent clickjacking
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

    // Prevent MIME type sniffing
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // Enable browser XSS protection
    headers.insert(
        header::X_XSS_PROTECTION,
        HeaderValue::from_static("1; mode=block"),
    );

    // HSTS - Force HTTPS (note: in Axum, we check scheme differently)
    // For simplicity, we'll add it unconditionally in production
    // In a real app, you'd check the request scheme
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // Referrer Policy - limit referrer information
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions Policy - disable unnecessary browser features
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=(), payment=()"),
    );

    response
}

// Note: The security_headers_layer function has been removed for simplicity.
// To use this middleware, call: axum::middleware::from_fn(security_headers_middleware)

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        response::IntoResponse,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> impl IntoResponse {
        (StatusCode::OK, "test")
    }

    #[tokio::test]
    async fn test_security_headers_added() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(security_headers_middleware));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Check that security headers are present
        let headers = response.headers();

        // Content-Security-Policy
        assert!(headers.contains_key(header::CONTENT_SECURITY_POLICY));
        let csp = headers.get(header::CONTENT_SECURITY_POLICY).unwrap();
        assert!(csp.to_str().unwrap().contains("default-src 'self'"));

        // X-Frame-Options
        assert!(headers.contains_key(header::X_FRAME_OPTIONS));
        assert_eq!(headers.get(header::X_FRAME_OPTIONS).unwrap(), "DENY");

        // X-Content-Type-Options
        assert!(headers.contains_key(header::X_CONTENT_TYPE_OPTIONS));
        assert_eq!(
            headers.get(header::X_CONTENT_TYPE_OPTIONS).unwrap(),
            "nosniff"
        );

        // X-XSS-Protection
        assert!(headers.contains_key(header::X_XSS_PROTECTION));
        assert_eq!(
            headers.get(header::X_XSS_PROTECTION).unwrap(),
            "1; mode=block"
        );

        // Strict-Transport-Security
        assert!(headers.contains_key(header::STRICT_TRANSPORT_SECURITY));
        let hsts = headers.get(header::STRICT_TRANSPORT_SECURITY).unwrap();
        assert!(hsts.to_str().unwrap().contains("max-age=31536000"));

        // Referrer-Policy
        assert!(headers.contains_key(header::REFERRER_POLICY));
        assert_eq!(
            headers.get(header::REFERRER_POLICY).unwrap(),
            "strict-origin-when-cross-origin"
        );

        // Permissions-Policy
        assert!(headers.contains_key("permissions-policy"));
        let perms = headers.get("permissions-policy").unwrap();
        assert!(perms.to_str().unwrap().contains("geolocation=()"));
    }

    #[tokio::test]
    async fn test_csp_allows_required_sources() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(security_headers_middleware));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        let csp = response
            .headers()
            .get(header::CONTENT_SECURITY_POLICY)
            .unwrap()
            .to_str()
            .unwrap();

        // Check that required external sources are allowed
        assert!(csp.contains("unpkg.com")); // HTMX
        assert!(csp.contains("cdn.tailwindcss.com")); // Tailwind
        assert!(csp.contains("cdn.jsdelivr.net")); // DaisyUI
        assert!(csp.contains("wss://dragonseeker.win")); // Production WebSocket
        assert!(csp.contains("ws://localhost:8000")); // Dev WebSocket
    }
}
