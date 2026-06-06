use crate::{
    auth::{jwt, password, repo},
    error::{AppError, AppResult},
    state::AppState,
};

pub struct AuthService<'a> {
    pub state: &'a AppState,
}

impl<'a> AuthService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password_str: &str,
    ) -> AppResult<repo::UserRow> {
        if !self.state.allow_register {
            return Err(AppError::Forbidden);
        }
        if password_str.len() < 8 {
            return Err(AppError::Validation(
                "password must be at least 8 characters".into(),
            ));
        }
        if repo::find_by_username(&self.state.db, username)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict("username already exists".into()));
        }
        let hash = password::hash(password_str)?;
        repo::insert(&self.state.db, username, email, &hash).await
    }

    pub async fn login(&self, username: &str, password_str: &str) -> AppResult<(repo::UserRow, String)> {
        let user = repo::find_by_username(&self.state.db, username)
            .await?
            .ok_or(AppError::Unauthorized)?;
        if !password::verify(password_str, &user.password_hash)? {
            return Err(AppError::Unauthorized);
        }
        let token = jwt::issue(
            &self.state.jwt_secret,
            user.id,
            &user.username,
            self.state.jwt_expires_in,
        )?;
        Ok((user, token))
    }

    pub async fn me(&self, user_id: i64) -> AppResult<repo::UserRow> {
        repo::find_by_id(&self.state.db, user_id)
            .await?
            .ok_or(AppError::Unauthorized)
    }
}
