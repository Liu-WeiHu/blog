use crate::{
    response,
    service::user_ext::{UserExtService, new_user_ext_service},
};

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use sqlx::PgPool;

pub async fn one(State(pool): State<PgPool>, Path(user_id): Path<i32>) -> impl IntoResponse {
    let svc = new_user_ext_service(pool);
    let res = svc.one(user_id).await;
    response::make_response(res)
}
