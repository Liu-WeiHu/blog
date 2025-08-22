use axum::{Extension, response::IntoResponse};

use crate::{
    context::RequestContext,
    response,
    service::permission::{RbacService, new_rbac_service},
};

pub async fn get_visual_permissions(
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    let res = new_rbac_service(ctx).get_user_permission().await;
    response::make_response(res)
}
