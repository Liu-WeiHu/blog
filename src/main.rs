mod controller;
mod dao;
mod dto;
mod init;
mod jwt;
mod model;
mod pagination;
mod response;
mod route;
mod service;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let pool = init::get_db_pool().await;
    let app = route::new_route(pool);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
