use crate::{
    context::Context,
    controller::{permission, posts, user_accounts, user_ext},
    middleware::{auth_middleware, log_request, log_response},
};

use axum::{
    Router,
    http::StatusCode,
    middleware,
    routing::{get, post, put},
};
use tower_http::trace::TraceLayer;

pub fn new_route(ctx: Context) -> Router {
    let users_route = Router::new()
        .route("/test", post(user_accounts::test_user))
        .route("/list", post(user_accounts::list))
        .route("/{user_id}", get(user_accounts::one))
        .route("/{user_id}", put(user_accounts::edit));

    let user_ext_route = Router::new().route("/{user_id}", get(user_ext::one));

    let posts_route = Router::new()
        .route("/{id}", get(posts::one))
        .route("/list", post(posts::list))
        .route("/add", post(posts::add))
        .route("/{id}", put(posts::edit));

    let permission_route =
        Router::new().route("/user_perm", get(permission::get_visual_permissions));

    let service_route = Router::new()
        .nest("/users", users_route)
        .nest("/user_ext", user_ext_route)
        .nest("/posts", posts_route)
        .nest("/visual", permission_route)
        .layer(middleware::from_fn_with_state(ctx.clone(), auth_middleware));

    let api_route = Router::new().nest("/api/v1", service_route);

    let login_route = Router::new()
        .route("/register", post(user_accounts::register))
        .route("/login", post(user_accounts::login));

    let app_route = Router::new().merge(api_route).merge(login_route).layer(
        TraceLayer::new_for_http()
            .on_request(log_request)
            .on_response(log_response),
    );

    Router::new()
        .merge(app_route)
        .route("/", get(async || "hello, world"))
        .fallback(async || (StatusCode::NOT_FOUND, "not found!"))
        .with_state(ctx)
}
