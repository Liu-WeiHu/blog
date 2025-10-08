
use crate::{context::RequestContext, pagination::PageParams, proto_gen::proto_api::api_service_server::ApiService, response::make_response, service::user_accounts::{new_user_accounts_service, UserAccountsService}
};

#[derive(Clone)]
pub struct GrpcRoute {
}

impl GrpcRoute {
	pub fn new() -> Self {
		Self {  }
	}
}

#[tonic::async_trait]
impl ApiService for GrpcRoute {
	async fn list(
		&self,
		request: tonic::Request<crate::proto_gen::proto_api::Pagination>,
	) -> std::result::Result<tonic::Response<crate::proto_gen::proto_api::ListResponse>, tonic::Status> {
		let ctx = request.extensions().get::<RequestContext>().cloned().ok_or(tonic::Status::unauthenticated("No context found"))?;
		let svc = new_user_accounts_service(ctx);
		let page = PageParams {
			page: Some(request.get_ref().page),
			size: Some(request.get_ref().size),
		}.to_pagination();
		let res = svc.list(page).await;
		let resp = make_response(res);
		Ok(tonic::Response::new(resp))
	}

	async fn one(
		&self,
		request: tonic::Request<crate::proto_gen::proto_api::GetUserOneRequest>,
	) -> std::result::Result<tonic::Response<crate::proto_gen::proto_api::UserOneResponse>, tonic::Status> {
		todo!()
	}

	async fn register(
		&self,
		request: tonic::Request<crate::proto_gen::proto_api::RegisterUserRequest>,
	) -> std::result::Result<tonic::Response<crate::proto_gen::proto_api::UserOneResponse>, tonic::Status> {
		todo!()
	}

	async fn edit(
		&self,
		request: tonic::Request<crate::proto_gen::proto_api::EditUserRequest>,
	) -> std::result::Result<tonic::Response<crate::proto_gen::proto_api::UserOneResponse>, tonic::Status> {
		todo!()
	}

	async fn login(
		&self,
		request: tonic::Request<crate::proto_gen::proto_api::LoginRequest>,
	) -> std::result::Result<tonic::Response<crate::proto_gen::proto_api::AuthResponse>, tonic::Status> {
		todo!()
	}
}