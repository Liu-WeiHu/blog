use crate::{
    dto::user_accounts::{LoginReq, RegisterReq},
    model::user_accounts::UserAccounts,
    pagination::Pagination,
    response,
    service::user_accounts::{UserAccountsService, new_user_accounts_service},
};

use axum::{
    Extension,
    extract::{Json, Path, State},
    response::IntoResponse,
};
use sqlx::PgPool;

pub async fn test_user(Extension(user): Extension<UserAccounts>) -> impl IntoResponse {
    response::make_response(Ok(user))
}

// #[axum::debug_handler]
pub async fn list(State(pool): State<PgPool>, pagination: Pagination) -> impl IntoResponse {
    let svc = new_user_accounts_service(pool);
    let res = svc.list(pagination).await;
    response::make_response(res)
}

// #[axum::debug_handler]
pub async fn one(State(pool): State<PgPool>, Path(user_id): Path<i32>) -> impl IntoResponse {
    let svc = new_user_accounts_service(pool);
    let res = svc.one(user_id).await;
    response::make_response(res)
}

pub async fn register(
    State(pool): State<PgPool>,
    Json(req): Json<RegisterReq>,
) -> impl IntoResponse {
    let svc = new_user_accounts_service(pool);
    let user = UserAccounts {
        username: req.username,
        email: req.email,
        password: req.password,
        ..Default::default()
    };
    let res = svc.register(user).await;
    response::make_response(res)
}

pub async fn login(State(pool): State<PgPool>, Json(req): Json<LoginReq>) -> impl IntoResponse {
    let svc = new_user_accounts_service(pool);
    let res = svc.login(req.email, req.password).await;
    response::make_response(res)
}
