use serde::{Deserialize, Serialize};

/// Request to join a game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinGameRequest {
    /// Player's display name (1-20 characters, alphanumeric and spaces only)
    pub nickname: String,
}

impl JoinGameRequest {
    /// Validate and clean the nickname
    ///
    /// # Arguments
    ///
    /// * `nickname` - Raw nickname input
    ///
    /// # Returns
    ///
    /// Cleaned nickname if valid, error message otherwise
    ///
    /// # Validation Rules
    ///
    /// - Must not be empty after trimming
    /// - Length: 1-20 characters
    /// - Only alphanumeric characters and spaces allowed
    pub fn validate_nickname(nickname: &str) -> Result<String, String> {
        let cleaned = nickname.trim();

        if cleaned.is_empty() {
            return Err("Nickname cannot be empty".to_string());
        }

        if cleaned.len() > 20 {
            return Err("Nickname must be 20 characters or less".to_string());
        }

        // Check if all characters are alphanumeric or spaces
        if !cleaned
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_whitespace())
        {
            return Err("Nickname must contain only letters, numbers, and spaces".to_string());
        }

        Ok(cleaned.to_string())
    }

    /// Create a new JoinGameRequest with validated nickname
    ///
    /// # Arguments
    ///
    /// * `nickname` - Raw nickname input
    ///
    /// # Returns
    ///
    /// JoinGameRequest if nickname is valid, error otherwise
    pub fn new(nickname: String) -> Result<Self, String> {
        let validated_nickname = Self::validate_nickname(&nickname)?;
        Ok(Self {
            nickname: validated_nickname,
        })
    }
}

/// Request to vote for a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRequest {
    /// ID of player to vote for
    pub target_id: String,
}

/// Dragon's word guess request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuessWordRequest {
    /// The guessed word (1-50 characters)
    pub guess: String,
}

impl GuessWordRequest {
    /// Clean and validate the guess
    ///
    /// # Arguments
    ///
    /// * `guess` - Raw guess input
    ///
    /// # Returns
    ///
    /// Cleaned guess (trimmed and lowercased) if valid, error otherwise
    pub fn validate_guess(guess: &str) -> Result<String, String> {
        let cleaned = guess.trim().to_lowercase();

        if cleaned.is_empty() {
            return Err("Guess cannot be empty".to_string());
        }

        if cleaned.len() > 50 {
            return Err("Guess must be 50 characters or less".to_string());
        }

        Ok(cleaned)
    }

    /// Create a new GuessWordRequest with validated guess
    ///
    /// # Arguments
    ///
    /// * `guess` - Raw guess input
    ///
    /// # Returns
    ///
    /// GuessWordRequest if guess is valid, error otherwise
    pub fn new(guess: String) -> Result<Self, String> {
        let validated_guess = Self::validate_guess(&guess)?;
        Ok(Self {
            guess: validated_guess,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_nickname_valid() {
        assert_eq!(
            JoinGameRequest::validate_nickname("Alice").unwrap(),
            "Alice"
        );
        assert_eq!(
            JoinGameRequest::validate_nickname("Bob123").unwrap(),
            "Bob123"
        );
        assert_eq!(
            JoinGameRequest::validate_nickname("Charlie 456").unwrap(),
            "Charlie 456"
        );
    }

    #[test]
    fn test_validate_nickname_trim() {
        assert_eq!(
            JoinGameRequest::validate_nickname("  Alice  ").unwrap(),
            "Alice"
        );
    }

    #[test]
    fn test_validate_nickname_empty() {
        assert!(JoinGameRequest::validate_nickname("").is_err());
        assert!(JoinGameRequest::validate_nickname("   ").is_err());
    }

    #[test]
    fn test_validate_nickname_too_long() {
        let long_name = "a".repeat(21);
        assert!(JoinGameRequest::validate_nickname(&long_name).is_err());
    }

    #[test]
    fn test_validate_nickname_invalid_chars() {
        assert!(JoinGameRequest::validate_nickname("Alice!").is_err());
        assert!(JoinGameRequest::validate_nickname("Bob@123").is_err());
        assert!(JoinGameRequest::validate_nickname("Charlie#").is_err());
    }

    #[test]
    fn test_join_game_request_new() {
        let request = JoinGameRequest::new("Alice".to_string()).unwrap();
        assert_eq!(request.nickname, "Alice");
    }

    #[test]
    fn test_join_game_request_new_invalid() {
        assert!(JoinGameRequest::new("".to_string()).is_err());
        assert!(JoinGameRequest::new("Alice!".to_string()).is_err());
    }

    #[test]
    fn test_validate_guess_valid() {
        assert_eq!(
            GuessWordRequest::validate_guess("elephant").unwrap(),
            "elephant"
        );
        assert_eq!(GuessWordRequest::validate_guess("TIGER").unwrap(), "tiger");
    }

    #[test]
    fn test_validate_guess_trim_and_lowercase() {
        assert_eq!(
            GuessWordRequest::validate_guess("  LION  ").unwrap(),
            "lion"
        );
    }

    #[test]
    fn test_validate_guess_empty() {
        assert!(GuessWordRequest::validate_guess("").is_err());
        assert!(GuessWordRequest::validate_guess("   ").is_err());
    }

    #[test]
    fn test_validate_guess_too_long() {
        let long_guess = "a".repeat(51);
        assert!(GuessWordRequest::validate_guess(&long_guess).is_err());
    }

    #[test]
    fn test_guess_word_request_new() {
        let request = GuessWordRequest::new("ELEPHANT".to_string()).unwrap();
        assert_eq!(request.guess, "elephant");
    }

    #[test]
    fn test_guess_word_request_new_invalid() {
        assert!(GuessWordRequest::new("".to_string()).is_err());
    }

    #[test]
    fn test_vote_request_creation() {
        let request = VoteRequest {
            target_id: "player123".to_string(),
        };
        assert_eq!(request.target_id, "player123");
    }
}
