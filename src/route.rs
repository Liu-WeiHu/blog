use crate::{
    controller::{permission, posts, user_accounts, user_ext},
};

use axum::{
    Router,
    http::StatusCode,
    routing::{get, post, put},
};

pub fn new_route() -> Router {
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
    let permission_route =
        Router::new().route("/user_perm", get(permission::get_visual_permissions));

    // 登陆注册相关路由
    let login_route = Router::new()
        .route("/register", post(user_accounts::register))
        .route("/login", post(user_accounts::login));

    let service_route = Router::new()
        .nest("/users", users_route)
        .nest("/user_ext", user_ext_route)
        .nest("/posts", posts_route)
        .nest("/visual", permission_route)
        .nest("/auth", login_route);

    let api_route = Router::new().nest("/api/v1", service_route);

    Router::new()
        .merge(api_route)
        .route("/", get(async || "hello, world"))
        .fallback(async || (StatusCode::NOT_FOUND, "not found!"))
}
