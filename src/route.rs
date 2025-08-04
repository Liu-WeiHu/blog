use super::{
    Deserialize, Duration, Level, MatchedPath, Next, PgPool, Request, Router, Serialize,
    StatusCode, TraceLayer, controller, event, get, middleware,
};

pub fn new_route(pool: PgPool) -> Router {
    let app = Router::new()
        .route("/", get(async || "hello, world"))
        .route("/list", get(controller::user_accounts::list))
        .route("/{user_id}", get(controller::user_accounts::one))
        .layer(
            TraceLayer::new_for_http()
                .on_request(|req: &Request<_>, _span: &tracing::Span| {
                    event!(
                        Level::INFO,
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
                    |response: &axum::response::Response,
                     latency: Duration,
                     _span: &tracing::Span| {
                        event!(
                            Level::INFO,
                            "Response completed: status={}, latency={:?}",
                            response.status(),
                            latency
                        );
                    },
                ),
        )
        .with_state(pool);

    app.fallback(async || (StatusCode::NOT_FOUND, "not found!"))
}
