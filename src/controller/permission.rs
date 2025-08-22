use axum::response::IntoResponse;

use crate::{
    context::Context,
    dto::user_accounts::CacheUser,
    response::{self, ErrCode},
    service::permission::{RbacService, new_rbac_service},
};

pub async fn get_visual_permissions(ctx: Context) -> impl IntoResponse {
    let user = match ctx.get::<CacheUser>() {
        Some(user) => user,
        None => return response::make_response(Err(ErrCode::UnAuthorized)),
    };
    let res = new_rbac_service(ctx).get_user_permission(user.id).await;
    response::make_response(res)
}
