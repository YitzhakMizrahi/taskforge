use crate::error::AppError;
use bcrypt::{hash, verify};

pub fn hash_password(password: &str) -> Result<String, AppError> {
    hash(password, 12) // bcrypt default cost is 12
        .map_err(|e| AppError::InternalServerError(format!("Failed to hash password: {}", e)))
}

pub fn verify_password(password: &str, hashed_password: &str) -> Result<bool, AppError> {
    verify(password, hashed_password)
        .map_err(|e| AppError::InternalServerError(format!("Failed to verify password: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_and_verification() {
        let password = "test_password123";
        let hashed = hash_password(password).unwrap();

        assert!(verify_password(password, &hashed).unwrap());
        assert!(!verify_password("wrong_password", &hashed).unwrap());
    }

    #[test]
    fn test_verify_with_invalid_hash() {
        match verify_password("test_password123", "invalidhashformat") {
            Err(AppError::InternalServerError(msg)) => {
                // bcrypt might return a specific error for malformed hash,
                // or just fail verification. The exact message can vary.
                assert!(msg.contains("Failed to verify password"));
            }
            Ok(false) => {
                // Depending on bcrypt's behavior with malformed hashes,
                // it might return Ok(false) instead of an error.
                // This branch is to acknowledge that possibility.
            }
            Ok(true) => panic!("Password verification should fail for invalid hash format"),
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
