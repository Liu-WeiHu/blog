use crate::{
    AppState, response,
    service::user_ext::{UserExtService, new_user_ext_service},
};

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

pub async fn one(State(state): State<AppState>, Path(user_id): Path<i32>) -> impl IntoResponse {
    let svc = new_user_ext_service(state.pool);
    let res = svc.one(user_id).await;
    response::make_response(res)
}
