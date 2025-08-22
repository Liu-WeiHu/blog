use crate::{
    context::RequestContext,
    response,
    service::user_ext::{UserExtService, new_user_ext_service},
};

use axum::{Extension, extract::Path, response::IntoResponse};

pub async fn one(
    Extension(ctx): Extension<RequestContext>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    let svc = new_user_ext_service(ctx);
    let res = svc.one(user_id).await;
    response::make_response(res)
}
