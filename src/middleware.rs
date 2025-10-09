use std::{future::Future, pin::Pin};

use http::{Request as HttpRequest, Response};
use hyper::body::Incoming;
use redis::Commands;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::{
    context::{Context as Ctx, GlobalContext, RequestContext},
    dto::user_accounts::CacheUser,
    hybrid::{HybridBody},
    init::KEYS,
    jwt::Claims,
    response::{self, ErrCode, Resp},
};

#[derive(Clone)]
pub struct AuthLayer {
    global_ctx: GlobalContext,
}

impl AuthLayer {
    pub fn new(global_ctx: GlobalContext) -> Self {
        Self { global_ctx }
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
            global_ctx: self.global_ctx.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthService<S> {
    inner: S,
    global_ctx: GlobalContext,
}

impl<S, RestBody, GrpcBody> Service<HttpRequest<Incoming>> for AuthService<S>
where
    S: Service<HttpRequest<Incoming>, Response = Response<HybridBody<RestBody, GrpcBody>>>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send + 'static,
    RestBody: From<Vec<u8>> + Default + Send + 'static,
    GrpcBody: Default + Send + 'static,
{
    type Response = Response<HybridBody<RestBody, GrpcBody>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, mut req: HttpRequest<Incoming>) -> Self::Future {
        let mut rc = RequestContext::new(self.global_ctx.clone());

        if req.headers().contains_key(http::header::AUTHORIZATION) {
            let cache_user = req
                .headers()
                .get(http::header::AUTHORIZATION)
                .ok_or(ErrCode::UnAuthorized)
                .and_then(|v| v.to_str().map_err(|_| ErrCode::UnAuthorized))
                .and_then(|s| s.strip_prefix("Bearer ").ok_or(ErrCode::UnAuthorized))
                .and_then(|token| self.authenticate_user(token));

            match cache_user {
                Ok(user) => rc.set_user(user),
                Err(err) => {
                    // 构造错误响应
                    let is_grpc = req
                        .headers()
                        .get(http::header::CONTENT_TYPE)
                        .and_then(|v| v.to_str().ok())
                        .map(|content_type| content_type.starts_with("application/grpc"))
                        .unwrap_or(false);

                    if is_grpc {
                        let grpc_body = GrpcBody::default();
                        let hybrid_body = HybridBody::Grpc { grpc_body };
                        let mut response = Response::new(hybrid_body);
                        *response.status_mut() = http::StatusCode::OK;
                        response.headers_mut().insert(
                            http::header::CONTENT_TYPE,
                            http::HeaderValue::from_static("application/grpc"),
                        );

                        response
                            .headers_mut()
                            .insert("grpc-status", http::HeaderValue::from_static("16"));

                        // 设置错误消息
                        let error_message = match err {
                            ErrCode::UnAuthorized => "没有授权",
                            ErrCode::InvalidToken => "无效的token",
                            _ => "Internal error",
                        };

                        response.headers_mut().insert(
                            "grpc-message",
                            http::HeaderValue::from_str(error_message).unwrap_or_else(|_| {
                                http::HeaderValue::from_static("Authentication failed")
                            }),
                        );

                        return Box::pin(async move { Ok(response) });
                    } else {
                        let json_response = response::make_response::<Resp<()>>(Err(err));
                        let json_body = match serde_json::to_string(&json_response) {
                            Ok(body) => body,
                            Err(_) => r#"{"code":500,"msg":"Internal Server Error","data":null}"#
                                .to_string(),
                        };

                        let rest_body = RestBody::from(json_body.into_bytes());
                        let hybrid_body = HybridBody::Rest { rest_body };
                        let mut response = Response::new(hybrid_body);
                        *response.status_mut() = http::StatusCode::UNAUTHORIZED;

                        response.headers_mut().insert(
                            http::header::CONTENT_TYPE,
                            http::HeaderValue::from_static("application/json"),
                        );

                        return Box::pin(async move { Ok(response) });
                    }
                }
            }
        }

        req.extensions_mut().insert(rc);
        // Otherwise, forward to the next service
        let mut inner = self.inner.clone();
        Box::pin(async move { inner.call(req).await.map_err(Into::into) })
    }
}

impl<S> AuthService<S> {
    fn authenticate_user(&self, token: &str) -> Result<CacheUser, ErrCode> {
        let claims = jsonwebtoken::decode::<Claims>(
            token,
            &KEYS.decoding,
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|e| {
            tracing::error!("JWT decode error: {}", e);
            ErrCode::InvalidToken
        })?;

        let user_id = claims.claims.sub.parse::<i32>().map_err(|e| {
            tracing::error!("Parse user_id error: {}", e);
            ErrCode::InvalidToken
        })?;

        let mut redis_conn = self
            .global_ctx
            .get_redis_client()
            .get_connection()
            .map_err(|err| {
                tracing::error!("Redis connection error: {}", err);
                ErrCode::InvalidToken
            })?;

        let redis_key = format!("user:{user_id}");
        let user_info_json: String = redis_conn.get(&redis_key).map_err(|e| {
            tracing::error!("redis get key is err = {}, redis_key = {}", e, redis_key);
            ErrCode::InvalidToken
        })?;

        let cache_user = serde_json::from_str::<CacheUser>(&user_info_json).map_err(|e| {
            tracing::error!(
                "str to cache_user is err = {}, json = {}",
                e,
                &user_info_json
            );
            ErrCode::InvalidToken
        })?;

        Ok(cache_user)
    }
}

// pub async fn auth_middleware(
//     State(ctx): State<GlobalContext>,
//     mut req: Request,
//     next: middleware::Next,
// ) -> impl IntoResponse {
//     let mut request_ctx = RequestContext::new(ctx.clone());

//     // 如果有auth头就校验,没有就跳过
//     if let Some(header) = req.headers().get("authorization") {
//         let token = match header.to_str() {
//             Ok(s) => match s.strip_prefix("Bearer ") {
//                 Some(t) => t,
//                 None => {
//                     return (
//                         StatusCode::UNAUTHORIZED,
//                         Json(make_response::<Resp<()>>(Err(ErrCode::UnAuthorized))),
//                     )
//                         .into_response();
//                 }
//             },
//             Err(_) => {
//                 return (
//                     StatusCode::UNAUTHORIZED,
//                     Json(make_response::<Resp<()>>(Err(ErrCode::UnAuthorized))),
//                 )
//                     .into_response();
//             }
//         };

//         // 解码JWT
//         let claims = match jsonwebtoken::decode::<Claims>(
//             token,
//             &KEYS.decoding,
//             &jsonwebtoken::Validation::default(),
//         ) {
//             Ok(data) => data.claims,
//             Err(e) => {
//                 tracing::error!("JWT decode error: {}", e);
//                 return (
//                     StatusCode::UNAUTHORIZED,
//                     Json(make_response::<Resp<()>>(Err(ErrCode::InvalidToken))),
//                 )
//                     .into_response();
//             }
//         };

//         // 解析用户 ID
//         let user_id = match claims.sub.parse::<i32>() {
//             Ok(user_id) => user_id,
//             Err(e) => {
//                 tracing::error!("Parse user_id error: {}", e);
//                 return (
//                     StatusCode::UNAUTHORIZED,
//                     Json(make_response::<Resp<()>>(Err(ErrCode::InvalidToken))),
//                 )
//                     .into_response();
//             }
//         };

//         // 从缓存获取数据
//         let mut redis_conn = match ctx.get_redis_client().get_connection() {
//             Ok(redis_conn) => redis_conn,
//             Err(err) => {
//                 tracing::error!("Redis connection error: {}", err);
//                 return (
//                     StatusCode::INTERNAL_SERVER_ERROR,
//                     Json(make_response::<Resp<()>>(Err(
//                         ErrCode::RedisServiceUnavailable,
//                     ))),
//                 )
//                     .into_response();
//             }
//         };

//         let redis_key = format!("user:{user_id}");
//         let user_info_json: String = match redis_conn.get(&redis_key) {
//             Ok(user_info_json) => user_info_json,
//             Err(e) => {
//                 tracing::error!("redis get key is err = {}, redis_key = {}", e, redis_key);
//                 return (
//                     StatusCode::UNAUTHORIZED,
//                     Json(make_response::<Resp<()>>(Err(ErrCode::InvalidToken))),
//                 )
//                     .into_response();
//             }
//         };

//         let cache_user = match serde_json::from_str::<CacheUser>(&user_info_json) {
//             Ok(user) => user,
//             Err(e) => {
//                 tracing::error!(
//                     "str to cache_user is err = {}, json = {}",
//                     e,
//                     &user_info_json
//                 );
//                 return (
//                     StatusCode::UNAUTHORIZED,
//                     Json(make_response::<Resp<()>>(Err(ErrCode::InvalidToken))),
//                 )
//                     .into_response();
//             }
//         };
//         request_ctx.set_user(cache_user);
//     }
//     req.extensions_mut().insert(request_ctx);
//     next.run(req).await
// }
