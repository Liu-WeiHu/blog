use std::time::Duration;

use bytes::Bytes;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
    service::TowerToHyperService,
};
use tonic::service::Routes;
use tower_http::{catch_panic::CatchPanicLayer, trace::TraceLayer};
use http::{Request as HttpRequest, Response, HeaderMap};
use tracing::{info, debug, Span, error};


use crate::{context::GlobalContext, middleware::AuthLayer, proto_gen::proto_api::api_service_server::ApiServiceServer};

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
    let auth_layer = AuthLayer::new(ctx);
    // 构建路由
    let http_route = route::new_route();
    let grpc_route = grpc_route::GrpcRoute::new();
    // 配置并构建 gRPC 反射服务，用于客户端发现服务和方法
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(include_bytes!("proto_gen/file_descriptor_set.bin")) // 注册 proto 描述符文件
        .build_v1()
        .expect("Failed to build reflection service"); // 构建反射服务

    let grpc_route = Routes::new(ApiServiceServer::new(grpc_route)).add_service(reflection_service);
    let hybrid_server = hybrid::hybrid(http_route, grpc_route);
    let server = tower::ServiceBuilder::new()
     .layer(CatchPanicLayer::new())
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &HttpRequest<_>| {
                        let span = tracing::info_span!("http-request",
                            status_code = tracing::field::Empty,
                            method = %request.method(),
                            uri = %request.uri(),
                            version = ?request.version(),
                        );
                        for (header_name, header_value) in request.headers() {
                            if header_name == "user-agent" || header_name == "content-type" || header_name == "content-length" {
                                span.record(header_name.as_str(), header_value.to_str().unwrap_or("invalid"));
                            }
                        }

                        span
                    })
                    .on_request(|request: &HttpRequest<_>, _span: &Span| {
                        info!(
                            counter.rustfs_api_requests_total = 1_u64,
                            key_request_method = %request.method().to_string(),
                            key_request_uri_path = %request.uri().path().to_owned(),
                            "handle request api total",
                        );
                        debug!("http started method: {}, url path: {}", request.method(), request.uri().path())
                    })
                    .on_response(|response: &Response<_>, latency: Duration, _span: &Span| {
                        _span.record("http response status_code", tracing::field::display(response.status()));
                        debug!("http response generated in {:?}", latency)
                    })
                    .on_body_chunk(|chunk: &Bytes, latency: Duration, _span: &Span| {
                        info!(histogram.request.body.len = chunk.len(), "histogram request body length",);
                        debug!("http body sending {} bytes in {:?}", chunk.len(), latency)
                    })
                    .on_eos(|_trailers: Option<&HeaderMap>, stream_duration: Duration, _span: &Span| {
                        debug!("http stream closed after {:?}", stream_duration)
                    })
                    .on_failure(|_error, latency: Duration, _span: &Span| {
                        info!(counter.rustfs_api_requests_failure_total = 1_u64, "handle request api failure total");
                        debug!("http request failure error: {:?} in {:?}", _error, latency)
                    }),
            )
            .layer(tower_http::cors::CorsLayer::permissive())
            .layer(auth_layer)
            .service(hybrid_server);

    let hyper_server = TowerToHyperService::new(server);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    loop {
        let Ok((stream, _addr)) = listener.accept().await else {
            error!("Failed to accept connection");
            continue;
        };
        let svc = hyper_server.clone();
        let io = TokioIo::new(stream);
        let _ = tokio::spawn(async move {
            let builder = Builder::new(TokioExecutor::new());
            if let Err(err) = builder.serve_connection(io, svc).await {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}
