use crate::error::{AppError, AppResult};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64, // user id
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn issue(secret: &str, user_id: i64, username: &str, expires_in: i64) -> AppResult<String> {
    let now = chrono::Utc::now().timestamp();
    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        iat: now,
        exp: now + expires_in,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("jwt encode failed: {e}")))
}

pub fn verify(token: &str, secret: &str) -> AppResult<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;
    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_and_verify_round_trip() {
        let token = issue("secret", 42, "alice", 3600).unwrap();
        let claims = verify(&token, "secret").unwrap();
        assert_eq!(claims.sub, 42);
        assert_eq!(claims.username, "alice");
    }

    #[test]
    fn verify_wrong_secret_fails() {
        let token = issue("secret", 1, "bob", 3600).unwrap();
        assert!(verify(&token, "different-secret").is_err());
    }
}
