use std::time::Duration;

use crate::{
    context::GlobalContext,
    controller::{permission, posts, user_accounts, user_ext},
    middleware::auth_middleware,
};

use axum::{
    Router,
    http::{Request, Response, StatusCode},
    middleware,
    routing::{get, post, put},
};
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{Span, info, info_span, warn};

pub fn new_route(ctx: GlobalContext) -> Router {
    //  账号相关路由
    let users_route = Router::new()
        .route("/test", post(user_accounts::test_user))
        .route("/list", post(user_accounts::list))
        .route("/{user_id}", get(user_accounts::one))
        .route("/{user_id}", put(user_accounts::edit));

    // 用户扩展信息相关路由
    let user_ext_route = Router::new().route("/{user_id}", get(user_ext::one));

    // 文章相关路由
    let posts_route = Router::new()
        .route("/{id}", get(posts::one))
        .route("/list", post(posts::list))
        .route("/add", post(posts::add))
        .route("/{id}", put(posts::edit));

    // 权限相关路由
    let permission_route = Router::new().route("/user_perm", get(permission::get_visual_permissions));

    // 登陆注册相关路由
    let login_route = Router::new()
        .route("/register", post(user_accounts::register))
        .route("/login", post(user_accounts::login));

    let service_route = Router::new()
        .nest("/users", users_route)
        .nest("/user_ext", user_ext_route)
        .nest("/posts", posts_route)
        .nest("/visual", permission_route)
        .nest("/auth", login_route)
        .layer(middleware::from_fn_with_state(ctx.clone(), auth_middleware));

    let api_route = Router::new()
        .nest("/api/v1", service_route)
        // 传播 request-id 到响应头
        .layer(PropagateRequestIdLayer::x_request_id())
        // 允许跨域
        .layer(CorsLayer::permissive())
        // 允许压缩
        .layer(CompressionLayer::new())
        // 捕获 panic
        .layer(CatchPanicLayer::new())
        // 创建根 span 并记录 trace_id
        .layer(
            TraceLayer::new_for_http()
                // 记录请求 ID
                .make_span_with(|request: &Request<_>| {
                    let trace_id = request
                        .headers()
                        .get("x-request-id")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("unknown");
                    info_span!("http-request",
                        trace_id = %trace_id,
                        method = %request.method(),
                        uri = %request.uri(),
                        status_code = tracing::field::Empty,
                    )
                })
                // 记录请求方法和路径
                // .on_request(|request: &Request<_>, _span: &Span| {
                //     info!(
                //         request_method = %request.method(),
                //         request_uri_path = %request.uri().path(),
                //         "http request received"
                //     )
                // })
                .on_request(())
                // 记录响应状态码和响应时间
                .on_response(|response: &Response<_>, latency: Duration, _span: &Span| {
                    let status = response.status();
                    _span.record("status_code", tracing::field::display(status));

                    // 记录响应时间，便于性能分析
                    let latency_ms = latency.as_millis();

                    match status {
                        StatusCode::OK => {
                            if latency_ms > 1000 {
                                warn!("slow request: {:?}", latency);
                            } else {
                                info!("request completed in {:?}", latency);
                            }
                        }
                        _ => {
                            warn!("request failed with status {} in {:?}", status, latency);
                        }
                    }
                })
                // // 记录响应体发送情况
                // .on_body_chunk(|chunk: &bytes::Bytes, latency: Duration, _span: &Span| {
                //     let body = String::from_utf8_lossy(&chunk[..]);
                //     info!("http body sending {} bytes in {:?}", body, latency);
                // })
                // 记录响应体发送完成情况
                // .on_eos(|_trailers: Option<&HeaderMap>, stream_duration: Duration, _span: &Span| {
                //     warn!("http stream closed after {:?}", stream_duration)
                // })
                // 记录请求失败情况
                // .on_failure(|_error, latency: Duration, _span: &Span| {
                //     error!("http request failure error: {:?} in {:?}", _error, latency)
                // }),
                .on_failure(()),
        )
        // 生成或提取 request-id
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));

    Router::new()
        .merge(api_route)
        .route("/", get(async || "hello, world"))
        .fallback(async || (StatusCode::NOT_FOUND, "not found!"))
        .with_state(ctx)
}
