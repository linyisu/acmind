use crate::{
    auth::{middleware::UserContext, repo, service::AuthService},
    error::AppResult,
    state::AppState,
};
use axum::{
    extract::State,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
}

pub fn protected_router() -> Router<AppState> {
    Router::new().route("/auth/me", get(me))
}

#[derive(Deserialize)]
pub struct RegisterReq {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResp {
    pub id: i64,
    pub username: String,
    pub email: String,
}

impl From<repo::UserRow> for UserResp {
    fn from(u: repo::UserRow) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
        }
    }
}

#[derive(Serialize)]
pub struct LoginResp {
    pub token: String,
    pub user: UserResp,
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterReq>,
) -> AppResult<Json<UserResp>> {
    let svc = AuthService::new(&state);
    let user = svc
        .register(&req.username, &req.email, &req.password)
        .await?;
    Ok(Json(user.into()))
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginReq>,
) -> AppResult<Json<LoginResp>> {
    let svc = AuthService::new(&state);
    let (user, token) = svc.login(&req.username, &req.password).await?;
    Ok(Json(LoginResp {
        token,
        user: user.into(),
    }))
}

async fn me(
    State(state): State<AppState>,
    Extension(ctx): Extension<UserContext>,
) -> AppResult<Json<UserResp>> {
    let svc = AuthService::new(&state);
    let user = svc.me(ctx.user_id).await?;
    Ok(Json(user.into()))
}
