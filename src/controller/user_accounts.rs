use crate::{
    context::Context,
    dto::user_accounts::{LoginReq, RegisterReq, UpdateUserInfoReq},
    model::user_accounts::UserAccounts,
    pagination::Pagination,
    response,
    service::user_accounts::{UserAccountsService, new_user_accounts_service},
};

use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};

pub async fn test_user(State(ctx): State<Context>) -> impl IntoResponse {
    let user = ctx.get::<UserAccounts>();
    response::make_response(Ok(user))
}

// #[axum::debug_handler]
pub async fn list(State(ctx): State<Context>, pagination: Pagination) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.list(pagination).await;
    response::make_response(res)
}

// #[axum::debug_handler]
pub async fn one(State(ctx): State<Context>, Path(user_id): Path<i32>) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.one(user_id).await;
    response::make_response(res)
}

pub async fn register(
    State(ctx): State<Context>,
    Json(req): Json<RegisterReq>,
) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.register(req).await;
    response::make_response(res)
}

pub async fn login(State(ctx): State<Context>, Json(req): Json<LoginReq>) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.login(req.email, req.password).await;
    response::make_response(res)
}

pub async fn edit(
    State(ctx): State<Context>,
    Json(req): Json<UpdateUserInfoReq>,
) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.edit_info(req).await;
    response::make_response(res)
}
