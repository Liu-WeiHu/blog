use crate::{
    context::Context,
    controller::{user_accounts, user_ext},
    middleware::auth_middleware,
};

use axum::{
    Router,
    extract::{MatchedPath, Request},
    http::StatusCode,
    middleware,
    routing::{get, post, put},
};
use std::time::Duration;
use tower_http::trace::TraceLayer;

pub fn new_route(ctx: Context) -> Router {
    let users_route = Router::new()
        .route("/test", post(user_accounts::test_user))
        .route("/list", post(user_accounts::list))
        .route("/{user_id}", get(user_accounts::one))
        .route("/edit", put(user_accounts::edit));

    let user_ext_route = Router::new().route("/{user_id}", get(user_ext::one));

    let api_route = Router::new()
        .nest("/api/v1/users", users_route)
        .nest("/api/v1/user_ext", user_ext_route)
        .layer(middleware::from_fn_with_state(ctx.clone(), auth_middleware));

    let login_route = Router::new()
        .route("/register", post(user_accounts::register))
        .route("/login", post(user_accounts::login));

    let app_route = Router::new().merge(api_route).merge(login_route).layer(
        TraceLayer::new_for_http()
            .on_request(|req: &Request<_>, _span: &tracing::Span| {
                tracing::event!(
                    tracing::Level::INFO,
                    "Incoming request: method={}, uri={}, matched_path={}",
                    req.method(),
                    req.uri(),
                    req.extensions()
                        .get::<MatchedPath>()
                        .map(|matched_path| matched_path.as_str())
                        .unwrap_or("unknown")
                );
            })
            .on_response(
                |response: &axum::response::Response, latency: Duration, _span: &tracing::Span| {
                    if response.status() != StatusCode::OK {
                        tracing::event!(
                            tracing::Level::ERROR,
                            "Response completed: status={}, latency={:?}",
                            response.status(),
                            latency
                        );
                    }
                },
            ),
    );

    Router::new()
        .merge(app_route)
        .route("/", get(async || "hello, world"))
        .fallback(async || (StatusCode::NOT_FOUND, "not found!"))
        .with_state(ctx)
}
