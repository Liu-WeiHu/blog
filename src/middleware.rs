use std::time::Duration;

use axum::{
    Json,
    extract::{MatchedPath, Request, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
};
use redis::Commands;

use crate::{
    context::Context,
    init::KEYS,
    jwt::Claims,
    model::user_accounts::UserAccounts,
    response::{ErrCode, Resp, make_response},
};

pub async fn auth_middleware(
    State(ctx): State<Context>,
    req: Request,
    next: middleware::Next,
) -> impl IntoResponse {
    // 提取 token
    let token = match req.headers().get("authorization") {
        Some(header) => match header.to_str() {
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
        },
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(make_response::<Resp<()>>(Err(ErrCode::UnAuthorized))),
            )
                .into_response();
        }
    };

    // 解码 JWT
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

    let user = match serde_json::from_str::<UserAccounts>(&user_info_json) {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("str to user is err = {}, json = {}", e, &user_info_json);
            return (
                StatusCode::UNAUTHORIZED,
                Json(make_response::<Resp<()>>(Err(ErrCode::InvalidToken))),
            )
                .into_response();
        }
    };

    // 加入到扩展里
    // req.extensions_mut().insert(user);
    ctx.insert(user);
    next.run(req).await
}

pub fn log_request<B>(req: &Request<B>, _span: &tracing::Span) {
    let method = req.method();
    let uri = req.uri();
    let matched_path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|path| path.as_str())
        .unwrap_or("unknown");

    tracing::info!(
        method = %method,
        uri = %uri,
        matched_path = %matched_path,
        "Incoming request"
    );
}

pub fn log_response(response: &axum::response::Response, latency: Duration, _span: &tracing::Span) {
    let status = response.status();

    match status {
        status if status.is_success() => {
            tracing::info!(
                status = %status,
                latency_ms = latency.as_millis(),
                "Request completed successfully"
            );
        }
        _ => {
            tracing::error!(
                status = %status,
                latency_ms = latency.as_millis(),
                "Request completed error"
            );
        }
    }
}
