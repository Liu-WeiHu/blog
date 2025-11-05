use crate::context::GlobalContext;

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
mod logs;

#[tokio::main]
async fn main() {
    logs::init_tracing();
    let pool = init::get_db_pool().await;
    let redis = init::get_redis_client().await;
    let mut ctx = GlobalContext::new(pool, redis);
    let perm = init::cache_rbac(ctx.clone()).await;
    ctx.set_perm(perm);
    let app = route::new_route(ctx);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
