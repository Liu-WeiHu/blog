use crate::{
    context::Context,
    response,
    service::user_ext::{UserExtService, new_user_ext_service},
};

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

pub async fn one(State(ctx): State<Context>, Path(user_id): Path<i32>) -> impl IntoResponse {
    let svc = new_user_ext_service(ctx);
    let res = svc.one(user_id).await;
    response::make_response(res)
}
