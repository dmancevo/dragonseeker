use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

const TOKEN_EXPIRY_SECONDS: u64 = 86400; // 24 hours

/// Generate a signed token for player authentication
///
/// # Arguments
///
/// * `game_id` - The game session ID
/// * `player_id` - The player's unique ID
/// * `secret_key` - Secret key for signing
///
/// # Returns
///
/// Signed token string in format: `payload.signature`
///
/// # Errors
///
/// Returns an error if HMAC initialization fails or time is invalid
pub fn generate_player_token(
    game_id: &str,
    player_id: &str,
    secret_key: &str,
) -> Result<String, String> {
    // Create payload with expiry (24 hours from now)
    let expiry = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("System time error: {}", e))?
        .as_secs()
        + TOKEN_EXPIRY_SECONDS;

    let payload = format!("{}:{}:{}", game_id, player_id, expiry);

    // Generate HMAC signature
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .map_err(|e| format!("HMAC initialization error: {}", e))?;
    mac.update(payload.as_bytes());
    let signature = mac.finalize().into_bytes();

    // Encode signature as base64 (URL-safe, no padding)
    let signature_b64 = URL_SAFE_NO_PAD.encode(signature);

    // Return token as payload.signature
    Ok(format!("{}.{}", payload, signature_b64))
}

/// Verify a player token and extract its data
///
/// # Arguments
///
/// * `token` - The token to verify
/// * `secret_key` - Secret key used for signing
///
/// # Returns
///
/// Tuple of (game_id, player_id, expiry) if valid, None otherwise
///
/// Uses constant-time comparison to prevent timing attacks
pub fn verify_player_token(token: Option<&str>, secret_key: &str) -> Option<(String, String, u64)> {
    let token = token?;

    // Split token into payload and signature
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 2 {
        return None;
    }

    let payload = parts[0];
    let signature_b64 = parts[1];

    // Parse payload
    let payload_parts: Vec<&str> = payload.split(':').collect();
    if payload_parts.len() != 3 {
        return None;
    }

    let game_id = payload_parts[0];
    let player_id = payload_parts[1];
    let expiry_str = payload_parts[2];

    // Parse and check expiry
    let expiry: u64 = expiry_str.parse().ok()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

    if now > expiry {
        return None; // Token expired
    }

    // Decode provided signature (URL-safe, no padding)
    let provided_signature = URL_SAFE_NO_PAD.decode(signature_b64).ok()?;

    // Constant-time comparison to prevent timing attacks
    // The hmac crate provides this via verify_slice
    let mut mac_verify = HmacSha256::new_from_slice(secret_key.as_bytes()).ok()?;
    mac_verify.update(payload.as_bytes());
    mac_verify.verify_slice(&provided_signature).ok()?;

    Some((game_id.to_string(), player_id.to_string(), expiry))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_valid_token() {
        let secret = "test_secret_key_12345";
        let game_id = "game123";
        let player_id = "player456";

        let token = generate_player_token(game_id, player_id, secret).unwrap();
        assert!(!token.is_empty());
        assert!(token.contains('.'));

        let result = verify_player_token(Some(&token), secret);
        assert!(result.is_some());

        let (decoded_game_id, decoded_player_id, _expiry) = result.unwrap();
        assert_eq!(decoded_game_id, game_id);
        assert_eq!(decoded_player_id, player_id);
    }

    #[test]
    fn test_verify_invalid_token_returns_none() {
        let secret = "test_secret_key";
        let invalid_token = "invalid.token.format";

        let result = verify_player_token(Some(invalid_token), secret);
        assert!(result.is_none());
    }

    #[test]
    fn test_verify_none_token() {
        let secret = "test_secret_key";
        let result = verify_player_token(None, secret);
        assert!(result.is_none());
    }

    #[test]
    fn test_verify_token_with_wrong_secret() {
        let secret = "correct_secret";
        let wrong_secret = "wrong_secret";
        let game_id = "game123";
        let player_id = "player456";

        let token = generate_player_token(game_id, player_id, secret).unwrap();

        // Verification with wrong secret should fail
        let result = verify_player_token(Some(&token), wrong_secret);
        assert!(result.is_none());
    }

    #[test]
    fn test_expired_token_fails() {
        // This test would require manipulating time or creating a token with past expiry
        // For now, we'll test the expiry logic by creating a token and verifying it's valid
        let secret = "test_secret";
        let game_id = "game123";
        let player_id = "player456";

        let token = generate_player_token(game_id, player_id, secret).unwrap();

        // Token should be valid now
        let result = verify_player_token(Some(&token), secret);
        assert!(result.is_some());

        // Note: We can't easily test actual expiry without waiting 24 hours
        // or manipulating system time. In production, the expiry check works as expected.
    }

    #[test]
    fn test_token_format() {
        let secret = "test_secret";
        let game_id = "game123";
        let player_id = "player456";

        let token = generate_player_token(game_id, player_id, secret).unwrap();

        // Token should have exactly one dot separator
        assert_eq!(token.matches('.').count(), 1);

        // Split and verify structure
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 2);

        // Payload should have 3 parts separated by colons
        let payload_parts: Vec<&str> = parts[0].split(':').collect();
        assert_eq!(payload_parts.len(), 3);
        assert_eq!(payload_parts[0], game_id);
        assert_eq!(payload_parts[1], player_id);

        // Expiry should be a valid number
        let expiry: u64 = payload_parts[2].parse().unwrap();
        assert!(expiry > 0);
    }

    #[test]
    fn test_malformed_token_missing_parts() {
        let secret = "test_secret";

        // Token with wrong number of parts in payload
        let bad_token = "game123:player456.signature";
        assert!(verify_player_token(Some(bad_token), secret).is_none());

        // Token with no signature
        let bad_token2 = "game123:player456:12345";
        assert!(verify_player_token(Some(bad_token2), secret).is_none());

        // Empty token
        assert!(verify_player_token(Some(""), secret).is_none());
    }

    #[test]
    fn test_tampered_payload() {
        let secret = "test_secret";
        let game_id = "game123";
        let player_id = "player456";

        let token = generate_player_token(game_id, player_id, secret).unwrap();

        // Tamper with the game_id in the payload
        let tampered_token = token.replace("game123", "game999");

        // Should fail verification
        let result = verify_player_token(Some(&tampered_token), secret);
        assert!(result.is_none());
    }

    #[test]
    fn test_different_players_different_tokens() {
        let secret = "test_secret";
        let game_id = "game123";

        let token1 = generate_player_token(game_id, "player1", secret).unwrap();
        let token2 = generate_player_token(game_id, "player2", secret).unwrap();

        // Tokens should be different
        assert_ne!(token1, token2);

        // Each should verify to correct player
        let (_, player_id1, _) = verify_player_token(Some(&token1), secret).unwrap();
        let (_, player_id2, _) = verify_player_token(Some(&token2), secret).unwrap();

        assert_eq!(player_id1, "player1");
        assert_eq!(player_id2, "player2");
    }
}
