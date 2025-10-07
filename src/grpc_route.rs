
use crate::{context:: {RequestContext}, 
    proto_gen::proto_api::api_service_server::ApiService, 
    service::user_accounts::{new_user_accounts_service, UserAccountsService},
};

#[derive(Clone)]
pub struct GrpcRoute {
    ctx: RequestContext,
}

impl GrpcRoute {
	pub fn new(ctx: crate::context::GlobalContext) -> Self {
		let rc = RequestContext::new(ctx);
		Self { ctx: rc }
	}
}

#[tonic::async_trait]
impl ApiService for GrpcRoute {
	async fn list(
		&self,
		request: tonic::Request<crate::proto_gen::proto_api::Pagination>,
	) -> std::result::Result<tonic::Response<crate::proto_gen::proto_api::GetListResponse>, tonic::Status> {
		// let svc = new_user_accounts_service(self.ctx);
        // let res = svc.list(request).await;
        // tonic::Response::new(res)
		todo!()
	}

	async fn one(
		&self,
		request: tonic::Request<crate::proto_gen::proto_api::GetUserOneRequest>,
	) -> std::result::Result<tonic::Response<crate::proto_gen::proto_api::UserResponse>, tonic::Status> {
		todo!()
	}

	async fn register(
		&self,
		request: tonic::Request<crate::proto_gen::proto_api::RegisterUserRequest>,
	) -> std::result::Result<tonic::Response<crate::proto_gen::proto_api::UserResponse>, tonic::Status> {
		todo!()
	}

	async fn edit(
		&self,
		request: tonic::Request<crate::proto_gen::proto_api::EditUserRequest>,
	) -> std::result::Result<tonic::Response<crate::proto_gen::proto_api::UserResponse>, tonic::Status> {
		todo!()
	}

	async fn login(
		&self,
		request: tonic::Request<crate::proto_gen::proto_api::LoginRequest>,
	) -> std::result::Result<tonic::Response<crate::proto_gen::proto_api::AuthBodyResponse>, tonic::Status> {
		todo!()
	}
}