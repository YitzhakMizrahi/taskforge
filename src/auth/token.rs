use crate::error::AppError;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// Represents the claims encoded within a JWT (JSON Web Token).
#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone for potential use in middleware
pub struct Claims {
    /// Subject of the token, typically the user's unique identifier.
    pub sub: i32, // user id
    /// Expiration timestamp (seconds since epoch) for the token.
    pub exp: usize,
}

/// Generates a JWT for a given user ID.
///
/// The token is set to expire in 24 hours.
/// It requires the `JWT_SECRET` environment variable to be set for signing the token.
///
/// # Arguments
/// * `user_id` - The ID of the user for whom the token is generated.
///
/// # Returns
/// A `Result` containing the JWT string if successful.
/// Returns `AppError::InternalServerError` if `JWT_SECRET` is not set or if token encoding fails.
pub fn generate_token(user_id: i32) -> Result<String, AppError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        exp: expiration,
    };

    let secret = match std::env::var("JWT_SECRET") {
        Ok(val) => val,
        Err(_) => {
            eprintln!("[DEBUG TOKEN_FN] JWT_SECRET not found in generate_token");
            return Err(AppError::InternalServerError("JWT_SECRET not set".into()));
        }
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(format!("Failed to generate token: {}", e)))
}

/// Verifies a JWT string and decodes its claims.
///
/// It requires the `JWT_SECRET` environment variable to be set for verifying the token signature.
/// Default validation checks are applied (e.g., signature, expiration).
///
/// # Arguments
/// * `token` - The JWT string to verify.
///
/// # Returns
/// A `Result` containing the decoded `Claims` if the token is valid.
/// Returns `AppError::InternalServerError` if `JWT_SECRET` is not set.
/// Returns `AppError::Unauthorized` if the token is malformed, its signature is invalid, or it has expired.
pub fn verify_token(token: &str) -> Result<Claims, AppError> {
    let secret = match std::env::var("JWT_SECRET") {
        Ok(val) => val,
        Err(_) => {
            eprintln!("[DEBUG TOKEN_FN] JWT_SECRET not found in verify_token");
            return Err(AppError::InternalServerError("JWT_SECRET not set".into()));
        }
    };
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(), // Consider customizing validation (e.g., issuer, audience)
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use std::thread;
    use std::time::Duration;

    lazy_static! {
        static ref JWT_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    }

    // Helper to run test logic with a temporarily set JWT_SECRET
    fn run_with_temp_jwt_secret<F>(secret_value: &str, test_logic: F)
    where
        F: FnOnce(),
    {
        let _guard = JWT_ENV_LOCK.lock().unwrap(); // Acquire lock, released when _guard goes out of scope

        let original_secret_val = std::env::var("JWT_SECRET").ok();
        std::env::set_var("JWT_SECRET", secret_value);

        // Using a panic hook to ensure cleanup even if test_logic panics
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test_logic));

        if let Some(original) = original_secret_val {
            std::env::set_var("JWT_SECRET", original);
        } else {
            std::env::remove_var("JWT_SECRET");
        }

        if let Err(panic_payload) = result {
            std::panic::resume_unwind(panic_payload);
        }
    }

    #[test]
    fn test_token_generation_and_verification() {
        run_with_temp_jwt_secret("test_secret_for_gen_verify", || {
            let user_id = 1;
            let token = generate_token(user_id).unwrap();
            let claims = verify_token(&token).unwrap();
            assert_eq!(claims.sub, user_id);
        });
    }

    #[test]
    fn test_token_expiration() {
        run_with_temp_jwt_secret("test_secret_for_expiration", || {
            let user_id = 2;

            let expiration = chrono::Utc::now()
                .checked_sub_signed(chrono::Duration::hours(2))
                .expect("valid timestamp")
                .timestamp() as usize;

            let claims_expired = Claims {
                sub: user_id,
                exp: expiration,
            };
            // JWT_SECRET is set by run_with_temp_jwt_secret, no need to get it from env here directly for encode
            let expired_token = encode(
                &Header::default(),
                &claims_expired,
                &EncodingKey::from_secret("test_secret_for_expiration".as_bytes()), // Use the same secret directly
            )
            .unwrap();

            thread::sleep(Duration::from_millis(50));

            match verify_token(&expired_token) {
                Err(AppError::Unauthorized(msg)) => {
                    if !msg.contains("Invalid token: ExpiredSignature") {
                        eprintln!("Unexpected error message for expired token: {}", msg);
                    }
                    assert!(msg.contains("Invalid token: ExpiredSignature"));
                }
                Ok(_) => panic!("Token should have been invalid due to expiration"),
                Err(e) => panic!("Unexpected error type for expired token: {:?}", e),
            }
        });
    }

    #[test]
    fn test_invalid_token_signature() {
        // The hardcoded token was likely signed with a common default secret like "your-256-bit-secret" or similar.
        // We will set our JWT_SECRET to something different to ensure an InvalidSignature error.
        run_with_temp_jwt_secret("a_completely_different_secret", || {
            let token_signed_with_other_secret = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

            match verify_token(token_signed_with_other_secret) {
                Err(AppError::Unauthorized(msg)) => {
                    // We expect "InvalidSignature" because our env var JWT_SECRET is "a_completely_different_secret"
                    // while the token was signed with something else.
                    if !msg.contains("Invalid token: InvalidSignature")
                        && !msg.contains("Invalid token: InvalidToken")
                    {
                        eprintln!("Unexpected error message for invalid signature: {}. Expected InvalidSignature or InvalidToken.", msg);
                    }
                    // jsonwebtoken can return InvalidToken for a JWT that is malformed in general,
                    // or InvalidSignature if specifically the signature part is wrong.
                    // Both are acceptable failure modes if the secret doesn't match.
                    assert!(
                        msg.contains("Invalid token: InvalidSignature")
                            || msg.contains("Invalid token: InvalidToken")
                    );
                }
                Ok(_) => panic!("Token should have been invalid due to signature mismatch"),
                Err(e) => panic!("Unexpected error type for invalid signature: {:?}", e),
            }
        });
    }
}
