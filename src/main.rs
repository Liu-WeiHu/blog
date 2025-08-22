use crate::context::Context;

mod context;
mod controller;
mod dao;
mod dto;
mod init;
mod jwt;
mod middleware;
mod model;
mod pagination;
mod rbac;
mod response;
mod route;
mod service;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let pool = init::get_db_pool().await;
    let redis = init::get_redis_client().await;
    let ctx = Context::new(pool, redis);
    let rbac = init::cache_rbac(ctx.clone()).await;
    ctx.insert(rbac);
    let app = route::new_route(ctx);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
