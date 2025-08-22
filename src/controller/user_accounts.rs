use crate::{
    context::{Context, RequestContext},
    dto::user_accounts::{LoginReq, RegisterReq},
    pagination::Pagination,
    response::{self, ErrCode},
    service::user_accounts::{UserAccountsService, new_user_accounts_service},
};

use axum::{
    Extension,
    extract::{Json, Path},
    response::IntoResponse,
};

pub async fn test_user(Extension(ctx): Extension<RequestContext>) -> impl IntoResponse {
    let user = ctx
        .get_user()
        .as_ref()
        .cloned()
        .ok_or(ErrCode::UnAuthorized);
    response::make_response(user)
}

// #[axum::debug_handler]
pub async fn list(
    Extension(ctx): Extension<RequestContext>,
    pagination: Pagination,
) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.list(pagination).await;
    response::make_response(res)
}

// #[axum::debug_handler]
pub async fn one(
    Extension(ctx): Extension<RequestContext>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.one(user_id).await;
    response::make_response(res)
}

pub async fn register(
    Extension(ctx): Extension<RequestContext>,
    Json(req): Json<RegisterReq>,
) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.register(req).await;
    response::make_response(res)
}

pub async fn login(
    Extension(ctx): Extension<RequestContext>,
    Json(req): Json<LoginReq>,
) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.login(req.email, req.password).await;
    response::make_response(res)
}

pub async fn edit(
    Extension(ctx): Extension<RequestContext>,
    Path(user_id): Path<i32>,
    Json(req): Json<RegisterReq>,
) -> impl IntoResponse {
    let svc = new_user_accounts_service(ctx);
    let res = svc.edit_info(req, user_id).await;
    response::make_response(res)
}
