use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
};
use redis::Commands;

use crate::{
    context::{Context, GlobalContext, RequestContext},
    dto::user_accounts::CacheUser,
    init::KEYS,
    jwt::Claims,
    response::{ErrCode, Resp, make_response},
};

pub async fn auth_middleware(
    State(ctx): State<GlobalContext>,
    mut req: Request,
    next: middleware::Next,
) -> impl IntoResponse {
    let mut request_ctx = RequestContext::new(ctx.clone());

    // 如果有auth头就校验,没有就跳过
    if let Some(header) = req.headers().get("authorization") {
        let token = match header.to_str() {
            Ok(s) => match s.strip_prefix("Bearer ") {
                Some(t) => t,
                None => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(make_response::<Resp<()>>(Err(ErrCode::UnAuthorized))),
                    )
                        .into_response();
                }
            },
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(make_response::<Resp<()>>(Err(ErrCode::UnAuthorized))),
                )
                    .into_response();
            }
        };

        // 解码JWT
        let claims = match jsonwebtoken::decode::<Claims>(
            token,
            &KEYS.decoding,
            &jsonwebtoken::Validation::default(),
        ) {
            Ok(data) => data.claims,
            Err(e) => {
                tracing::error!("JWT decode error: {}", e);
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(make_response::<Resp<()>>(Err(ErrCode::InvalidToken))),
                )
                    .into_response();
            }
        };

        // 解析用户 ID
        let user_id = match claims.sub.parse::<i32>() {
            Ok(user_id) => user_id,
            Err(e) => {
                tracing::error!("Parse user_id error: {}", e);
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(make_response::<Resp<()>>(Err(ErrCode::InvalidToken))),
                )
                    .into_response();
            }
        };

        // 从缓存获取数据
        let mut redis_conn = match ctx.get_redis_client().get_connection() {
            Ok(redis_conn) => redis_conn,
            Err(err) => {
                tracing::error!("Redis connection error: {}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(make_response::<Resp<()>>(Err(
                        ErrCode::RedisServiceUnavailable,
                    ))),
                )
                    .into_response();
            }
        };

        let redis_key = format!("user:{user_id}");
        let user_info_json: String = match redis_conn.get(&redis_key) {
            Ok(user_info_json) => user_info_json,
            Err(e) => {
                tracing::error!("redis get key is err = {}, redis_key = {}", e, redis_key);
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(make_response::<Resp<()>>(Err(ErrCode::InvalidToken))),
                )
                    .into_response();
            }
        };

        let cache_user = match serde_json::from_str::<CacheUser>(&user_info_json) {
            Ok(user) => user,
            Err(e) => {
                tracing::error!(
                    "str to cache_user is err = {}, json = {}",
                    e,
                    &user_info_json
                );
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(make_response::<Resp<()>>(Err(ErrCode::InvalidToken))),
                )
                    .into_response();
            }
        };
        request_ctx.set_user(cache_user);
    }
    req.extensions_mut().insert(request_ctx);
    next.run(req).await
}