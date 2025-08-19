use crate::{
    AppState,
    dto::user_accounts::{LoginReq, RegisterReq, UpdateUserInfoReq},
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

pub async fn test_user(Extension(user): Extension<UserAccounts>) -> impl IntoResponse {
    response::make_response(Ok(user))
}

// #[axum::debug_handler]
pub async fn list(State(state): State<AppState>, pagination: Pagination) -> impl IntoResponse {
    let svc = new_user_accounts_service(state.pool);
    let res = svc.list(pagination).await;
    response::make_response(res)
}

// #[axum::debug_handler]
pub async fn one(State(state): State<AppState>, Path(user_id): Path<i32>) -> impl IntoResponse {
    let svc = new_user_accounts_service(state.pool);
    let res = svc.one(user_id).await;
    response::make_response(res)
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterReq>,
) -> impl IntoResponse {
    let svc = new_user_accounts_service(state.pool);
    let res = svc.register(req).await;
    response::make_response(res)
}

pub async fn login(State(state): State<AppState>, Json(req): Json<LoginReq>) -> impl IntoResponse {
    let svc = new_user_accounts_service(state.pool);
    let redis_conn = state.redis.get_connection().unwrap();
    let res = svc.login(redis_conn, req.email, req.password).await;
    response::make_response(res)
}

pub async fn edit(
    State(state): State<AppState>,
    Json(req): Json<UpdateUserInfoReq>,
) -> impl IntoResponse {
    let svc = new_user_accounts_service(state.pool);
    let res = svc.edit_info(req).await;
    response::make_response(res)
}
