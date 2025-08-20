use axum::{
    Json,
    extract::{Request, State},
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

    tracing::debug!("Extracted token: {}", token);

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
    let mut redis_conn = ctx.get_redis().get_connection().unwrap();
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

// #[derive(Deserialize)]
// struct UserId {
//     user_id: i32,
// }

// async fn user_info_middleware(
//     State(pool): State<PgPool>,
//     req: Request,
//     next: Next,
// ) -> impl IntoResponse {
//     // 分离请求的 parts 和 body
//     let (parts, body) = req.into_parts();
//
//     // 收集 body 字节
//     let (bytes, user) = match body.collect().await {
//         Ok(collected) => {
//             let bytes = collected.to_bytes();
//             // 创建临时请求用于 JSON 提取
//             let temp_body = Body::from(bytes.clone());
//             let temp_req = Request::from_parts(parts.clone(), temp_body);
//
//             // 提取 JSON 并查询用户信息
//             let user = if let Ok(Json(user_id)) = Json::<UserId>::from_request(temp_req, &()).await
//             {
//                 let dao = new_user_accounts_dao(pool);
//                 dao.select_by_id(user_id.user_id).await.ok()
//             } else {
//                 None
//             };
//             (bytes, user)
//         }
//         Err(_) => (Bytes::new(), None),
//     };
//
//     let req = Request::from_parts(parts, Body::from(bytes));
//
//     // 将 UserInfo 插入到请求的 extensions 中
//     let mut req = req;
//     req.extensions_mut().insert(user);
//     next.run(req).await
// }
