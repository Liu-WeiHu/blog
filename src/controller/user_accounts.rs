use crate::{
    context::Context,
    dto::user_accounts::{CacheUser, LoginReq, RegisterReq},
    pagination::Pagination,
    response::{self, ErrCode},
    service::user_accounts::{UserAccountsService, new_user_accounts_service},
};

use axum::{
    extract::{Json, Path},
    response::IntoResponse,
};

pub async fn test_user(ctx: Context) -> impl IntoResponse {
    let user = ctx.get::<CacheUser>().ok_or(ErrCode::UnAuthorized);
    response::make_response(user)
}

// #[axum::debug_handler]
pub async fn list(ctx: Context, pagination: Pagination) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.list(pagination).await;
    response::make_response(res)
}

// #[axum::debug_handler]
pub async fn one(ctx: Context, Path(user_id): Path<i32>) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.one(user_id).await;
    response::make_response(res)
}

pub async fn register(ctx: Context, Json(req): Json<RegisterReq>) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.register(req).await;
    response::make_response(res)
}

pub async fn login(ctx: Context, Json(req): Json<LoginReq>) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.login(req.email, req.password).await;
    response::make_response(res)
}

pub async fn edit(
    ctx: Context,
    Path(user_id): Path<i32>,
    Json(req): Json<RegisterReq>,
) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.edit_info(req, user_id).await;
    response::make_response(res)
}
