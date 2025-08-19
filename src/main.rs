use sqlx::PgPool;

mod controller;
mod dao;
mod dto;
mod init;
mod jwt;
mod middleware;
mod model;
mod pagination;
mod response;
mod route;
mod service;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    redis: redis::Client,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let pool = init::get_db_pool().await;
    let redis = init::get_redis_client().await;
    let app_state = AppState { pool, redis };
    let app = route::new_route(app_state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
