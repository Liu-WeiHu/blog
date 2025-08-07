use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
};
use sqlx::PgPool;

use crate::{
    dao::user_accounts::{UserAccountsDao, new_user_accounts_dao},
    init::KEYS,
    jwt::Claims,
    response::{ErrCode, Resp, make_response},
};

pub async fn auth_middleware(
    State(pool): State<PgPool>,
    mut req: Request,
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

    // 查询用户
    let mut conn = pool.acquire().await.unwrap();
    let user = match new_user_accounts_dao().select_one(&mut conn, user_id).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Database query error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(make_response::<Resp<()>>(Err(ErrCode::InternalError))),
            )
                .into_response();
        }
    };

    // 加入到扩展里
    req.extensions_mut().insert(user);
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
