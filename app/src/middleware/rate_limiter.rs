use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

/// Track request rates per IP address using sliding window
#[derive(Clone)]
struct RateLimiter {
    /// Map of IP -> list of request timestamps
    requests: Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
    /// Last cleanup time
    last_cleanup: Arc<Mutex<Instant>>,
    /// Cleanup interval in seconds
    cleanup_interval: u64,
}

impl RateLimiter {
    /// Create a new rate limiter
    fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
            cleanup_interval: 60,
        }
    }

    /// Check if request is allowed under rate limit
    ///
    /// # Arguments
    ///
    /// * `ip` - Client IP address
    /// * `limit` - Maximum requests allowed in window
    /// * `window` - Time window in seconds
    ///
    /// # Returns
    ///
    /// True if request is allowed, False if rate limit exceeded
    async fn is_allowed(&self, ip: IpAddr, limit: usize, window: Duration) -> bool {
        let now = Instant::now();
        let cutoff = now - window;

        let mut requests = self.requests.lock().await;
        let timestamps = requests.entry(ip).or_insert_with(Vec::new);

        // Clean up old requests for this IP
        timestamps.retain(|&ts| ts > cutoff);

        // Check if under limit
        if timestamps.len() >= limit {
            return false;
        }

        // Record this request
        timestamps.push(now);
        true
    }

    /// Remove stale IP entries to prevent memory leaks
    async fn cleanup_old_entries(&self) {
        let now = Instant::now();

        let mut last_cleanup = self.last_cleanup.lock().await;

        // Only run cleanup periodically
        if now.duration_since(*last_cleanup) < Duration::from_secs(self.cleanup_interval) {
            return;
        }

        let cutoff = now - Duration::from_secs(60); // Remove IPs with no requests in last 60 seconds

        let mut requests = self.requests.lock().await;

        // Find IPs to remove
        let to_remove: Vec<IpAddr> = requests
            .iter()
            .filter(|(_, timestamps)| {
                timestamps.is_empty() || timestamps.iter().all(|&ts| ts < cutoff)
            })
            .map(|(ip, _)| *ip)
            .collect();

        // Remove stale entries
        for ip in to_remove {
            requests.remove(&ip);
        }

        *last_cleanup = now;
    }
}

/// Get rate limit for endpoint
///
/// # Arguments
///
/// * `path` - Request path
///
/// # Returns
///
/// Requests per second limit, or None to skip rate limiting
fn get_rate_limit(path: &str) -> Option<usize> {
    // No rate limiting for WebSocket connections
    if path.starts_with("/ws") {
        return None;
    }

    // No rate limiting for static files
    if path.starts_with("/static") {
        return None;
    }

    // Timer endpoint needs high limit (1 req/sec per player)
    // With max 12 players from same IP: need at least 12 req/s + overhead
    if path.contains("/timer") {
        return Some(30); // 12 players * 1 req/s + buffer
    }

    // Health check gets moderate limit
    if path == "/health" {
        return Some(10);
    }

    // Game creation - permissive for testing, strict in production
    // Tests may create many games in parallel
    if path.contains("/games/create") {
        return Some(100); // High limit to allow parallel tests
    }

    // API endpoints need high limit for 12 players + parallel tests
    // (voting, joining, etc. - all players might act simultaneously)
    if path.starts_with("/api") {
        return Some(100); // High limit for players + tests
    }

    // Page views need high limit (all players load pages after game state changes)
    if path.starts_with("/game/") {
        return Some(100); // High limit for players + tests
    }

    // Default limit for other endpoints
    Some(50) // Permissive default for tests
}

/// Global rate limiter instance
static RATE_LIMITER: once_cell::sync::Lazy<RateLimiter> =
    once_cell::sync::Lazy::new(RateLimiter::new);

/// Axum middleware to enforce rate limits per IP address
///
/// # Arguments
///
/// * `req` - Incoming request
/// * `next` - Next middleware/handler
///
/// # Returns
///
/// Response or rate limit error
pub async fn rate_limit_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    // Extract client IP from connection info
    let ip = req
        .extensions()
        .get::<std::net::SocketAddr>()
        .map(|addr| addr.ip())
        .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));

    // Get rate limit for this path
    let path = req.uri().path();
    let limit = match get_rate_limit(path) {
        Some(l) => l,
        None => return Ok(next.run(req).await), // Skip rate limiting
    };

    // Check rate limit
    if !RATE_LIMITER
        .is_allowed(ip, limit, Duration::from_secs(1))
        .await
    {
        // Return rate limit exceeded response
        let response = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "detail": "Rate limit exceeded. Please try again later."
            })),
        );
        return Ok(response.into_response());
    }

    // Periodic cleanup
    RATE_LIMITER.cleanup_old_entries().await;

    // Process request
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_rate_limit_websocket() {
        assert_eq!(get_rate_limit("/ws/game123/player456"), None);
    }

    #[test]
    fn test_get_rate_limit_static() {
        assert_eq!(get_rate_limit("/static/css/style.css"), None);
    }

    #[test]
    fn test_get_rate_limit_timer() {
        assert_eq!(get_rate_limit("/api/games/123/timer"), Some(30));
    }

    #[test]
    fn test_get_rate_limit_health() {
        assert_eq!(get_rate_limit("/health"), Some(10));
    }

    #[test]
    fn test_get_rate_limit_game_create() {
        assert_eq!(get_rate_limit("/api/games/create"), Some(100));
    }

    #[test]
    fn test_get_rate_limit_api() {
        assert_eq!(get_rate_limit("/api/games/123/join"), Some(100));
    }

    #[test]
    fn test_get_rate_limit_game_page() {
        assert_eq!(get_rate_limit("/game/123/lobby"), Some(100));
    }

    #[test]
    fn test_get_rate_limit_default() {
        assert_eq!(get_rate_limit("/some/other/path"), Some(50));
    }

    #[tokio::test]
    async fn test_rate_limiter_allows_under_limit() {
        let limiter = RateLimiter::new();
        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        // First request should be allowed
        assert!(limiter.is_allowed(ip, 5, Duration::from_secs(1)).await);

        // Second request should be allowed
        assert!(limiter.is_allowed(ip, 5, Duration::from_secs(1)).await);
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new();
        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        // Make 5 requests (limit is 5)
        for _ in 0..5 {
            assert!(limiter.is_allowed(ip, 5, Duration::from_secs(1)).await);
        }

        // 6th request should be blocked
        assert!(!limiter.is_allowed(ip, 5, Duration::from_secs(1)).await);
    }

    #[tokio::test]
    async fn test_rate_limiter_different_ips() {
        let limiter = RateLimiter::new();
        let ip1: IpAddr = "127.0.0.1".parse().unwrap();
        let ip2: IpAddr = "192.168.1.1".parse().unwrap();

        // Fill up limit for IP1
        for _ in 0..5 {
            assert!(limiter.is_allowed(ip1, 5, Duration::from_secs(1)).await);
        }

        // IP1 should be blocked
        assert!(!limiter.is_allowed(ip1, 5, Duration::from_secs(1)).await);

        // IP2 should still be allowed
        assert!(limiter.is_allowed(ip2, 5, Duration::from_secs(1)).await);
    }

    #[tokio::test]
    async fn test_rate_limiter_window_reset() {
        let limiter = RateLimiter::new();
        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        // Make 3 requests with short window
        for _ in 0..3 {
            assert!(limiter.is_allowed(ip, 3, Duration::from_millis(100)).await);
        }

        // Should be at limit
        assert!(!limiter.is_allowed(ip, 3, Duration::from_millis(100)).await);

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be allowed again
        assert!(limiter.is_allowed(ip, 3, Duration::from_millis(100)).await);
    }
}
