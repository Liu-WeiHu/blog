use tonic::service::Routes;

use crate::{controller_grpc::user_accounts::UserRoute, proto_gen::proto_api::user_service_server::UserServiceServer};


pub fn new_grpc_route() -> Routes {
	// 配置并构建 gRPC 反射服务，用于客户端发现服务和方法
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(include_bytes!("proto_gen/file_descriptor_set.bin")) // 注册 proto 描述符文件
        .build_v1()
        .expect("Failed to build reflection service"); // 构建反射服务
	Routes::new(UserServiceServer::new(UserRoute::new())).add_service(reflection_service)
}