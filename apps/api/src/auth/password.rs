use crate::error::{AppError, AppResult};

pub fn hash(password: &str) -> AppResult<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("bcrypt hash failed: {e}")))
}

pub fn verify(password: &str, hash: &str) -> AppResult<bool> {
    bcrypt::verify(password, hash)
        .map_err(|e| AppError::Internal(format!("bcrypt verify failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_round_trip() {
        let h = hash("hunter2").unwrap();
        assert!(verify("hunter2", &h).unwrap());
        assert!(!verify("wrong", &h).unwrap());
    }
}
