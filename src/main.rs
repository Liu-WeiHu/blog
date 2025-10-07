use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
    service::TowerToHyperService,
};
use tonic::service::Routes;

use crate::{context::GlobalContext, proto_gen::proto_api::api_service_server::ApiServiceServer};

mod context;
mod controller;
mod dao;
mod dto;
mod grpc_route;
mod hybrid;
mod init;
mod jwt;
mod middleware;
mod model;
mod pagination;
mod proto_gen;
mod rbac;
mod response;
mod route;
mod service;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let pool = init::get_db_pool().await;
    let redis = init::get_redis_client().await;
    let mut ctx = GlobalContext::new(pool, redis);
    let perm = init::cache_rbac(ctx.clone()).await;
    ctx.set_perm(perm);
    let http_route = route::new_route(ctx.clone());
    let grpc_route = grpc_route::GrpcRoute::new(ctx);
    // 配置并构建 gRPC 反射服务，用于客户端发现服务和方法
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(include_bytes!("proto_gen/file_descriptor_set.bin")) // 注册 proto 描述符文件
        .build_v1()
        .expect("Failed to build reflection service"); // 构建反射服务

    let grpc_route = Routes::new(ApiServiceServer::new(grpc_route)).add_service(reflection_service);
    let server = hybrid::hybrid(http_route, grpc_route);
    let hyper_server = TowerToHyperService::new(server);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    loop {
        let Ok((stream, _addr)) = listener.accept().await else {
            tracing::error!("Failed to accept connection");
            continue;
        };
        let svc = hyper_server.clone();
        let io = TokioIo::new(stream);
        let _ = tokio::spawn(async move {
            let builder = Builder::new(TokioExecutor::new());
            if let Err(err) = builder.serve_connection(io, svc).await {
                tracing::error!("Error serving connection: {:?}", err);
            }
        });
    }
}
