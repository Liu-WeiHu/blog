use prost_types::Timestamp;

use crate::{
    context::{Context, RequestContext},
    pagination::PageParams,
    proto_gen::proto_api::user_service_server::UserService,
    response::{make_response, ErrCode},
    service::user_accounts::{new_user_accounts_service, UserAccountsService},
};

#[derive(Clone)]
pub struct UserRoute {}

impl UserRoute {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl UserService for UserRoute {
    async fn test_user(
        &self,
        request: tonic::Request<crate::proto_gen::proto_api::Empty>,
    ) -> std::result::Result<
        tonic::Response<crate::proto_gen::proto_api::TestUserResponse>,
        tonic::Status,
    > {
        let ctx = request
            .extensions()
            .get::<RequestContext>()
            .cloned()
            .ok_or(tonic::Status::unauthenticated("No context found"))?;
        let user = ctx
            .get_user()
            .as_ref()
            .cloned()
            .ok_or(ErrCode::UnAuthorized);
        let resp = make_response(user);
        let response = crate::proto_gen::proto_api::TestUserResponse {
            code: resp.code,
            msg: resp.msg,
            data: resp.data.map(|user| crate::proto_gen::proto_api::TestUser {
                id: user.id,
                username: user.username.into(),
                email: user.email.into(),
                created_at: user.created_at.map(|dt| Timestamp {
                    seconds: dt.and_utc().timestamp(),
                    nanos: dt.and_utc().timestamp_subsec_nanos() as i32,
                }),
                last_login_time: user.last_login_time.map(|dt| Timestamp {
                    seconds: dt.and_utc().timestamp(),
                    nanos: dt.and_utc().timestamp_subsec_nanos() as i32,
                }),
                role_ids: user.role_ids,
            }),
        };
        Ok(tonic::Response::new(response))
    }

    async fn list(
        &self,
        request: tonic::Request<crate::proto_gen::proto_api::Pagination>,
    ) -> std::result::Result<
        tonic::Response<crate::proto_gen::proto_api::ListResponse>,
        tonic::Status,
    > {
        let ctx = request
            .extensions()
            .get::<RequestContext>()
            .cloned()
            .ok_or(tonic::Status::unauthenticated("No context found"))?;
        let svc = new_user_accounts_service(ctx);
        let page = PageParams {
            page: Some(request.get_ref().page),
            size: Some(request.get_ref().size),
        }
        .to_pagination();
        let res = svc.list(page).await;
        let resp = make_response(res);
        let response = crate::proto_gen::proto_api::ListResponse {
            code: resp.code,
            msg: resp.msg,
            data: resp
                .data
                .map(|list| crate::proto_gen::proto_api::GetListResponse {
                    users: list
                        .users
                        .into_iter()
                        .map(|user| crate::proto_gen::proto_api::UserResponse {
                            id: user.id,
                            username: user.username.to_string(),
                            email: user.email.to_string(),
                            created_at: user.created_at.map(|dt| Timestamp {
                                seconds: dt.and_utc().timestamp(),
                                nanos: dt.and_utc().timestamp_subsec_nanos() as i32,
                            }),
                            last_login_time: user.last_login_time.map(|dt| Timestamp {
                                seconds: dt.and_utc().timestamp(),
                                nanos: dt.and_utc().timestamp_subsec_nanos() as i32,
                            }),
                        })
                        .collect(),
                    total: list.total,
                }),
        };
        Ok(tonic::Response::new(response))
    }

    async fn one(
        &self,
        request: tonic::Request<crate::proto_gen::proto_api::GetUserOneRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::proto_gen::proto_api::UserOneResponse>,
        tonic::Status,
    > {
        let ctx = request
            .extensions()
            .get::<RequestContext>()
            .cloned()
            .ok_or(tonic::Status::unauthenticated("No context found"))?;
        let svc = new_user_accounts_service(ctx);
        let user_id = request.get_ref().id;
        let res = svc.one(user_id).await;
        let resp = make_response(res);
        let response = crate::proto_gen::proto_api::UserOneResponse {
            code: resp.code,
            msg: resp.msg,
            data: resp
                .data
                .map(|user| crate::proto_gen::proto_api::OneResponse {
                    code: user.id,
                    username: user.username.to_string(),
                    email: user.email.to_string(),
                    age: user.age,
                    gender: user.gender,
                    education: user.education,
                    hometown: user.hometown,
                    address: user.address,
                }),
        };
        Ok(tonic::Response::new(response))
    }

    async fn register(
        &self,
        request: tonic::Request<crate::proto_gen::proto_api::RegisterUserRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::proto_gen::proto_api::RegisterResponse>,
        tonic::Status,
    > {
        let ctx = request
            .extensions()
            .get::<RequestContext>()
            .cloned()
            .ok_or(tonic::Status::unauthenticated("No context found"))?;
        let svc = new_user_accounts_service(ctx);
        let req = request.into_inner();
        let register_req = crate::dto::user_accounts::RegisterReq {
            username: req.username.into(),
            email: req.email.into(),
            password: req.password.into(),
            age: req.age,
            gender: req.gender,
            education: req.education,
            hometown: req.hometown,
            address: req.address,
        };
        let res = svc.register(register_req).await;
        let resp = make_response(res);
        let response = crate::proto_gen::proto_api::RegisterResponse {
            code: resp.code,
            msg: resp.msg,
            data: resp
                .data
                .map(|user| crate::proto_gen::proto_api::UserResponse {
                    id: user.id,
                    username: user.username.to_string(),
                    email: user.email.to_string(),
                    created_at: user.created_at.map(|dt| Timestamp {
                        seconds: dt.and_utc().timestamp(),
                        nanos: dt.and_utc().timestamp_subsec_nanos() as i32,
                    }),
                    last_login_time: user.last_login_time.map(|dt| Timestamp {
                        seconds: dt.and_utc().timestamp(),
                        nanos: dt.and_utc().timestamp_subsec_nanos() as i32,
                    }),
                }),
        };
        Ok(tonic::Response::new(response))
        // todo!()
    }

    async fn edit(
        &self,
        request: tonic::Request<crate::proto_gen::proto_api::EditUserRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::proto_gen::proto_api::RegisterResponse>,
        tonic::Status,
    > {
        let ctx = request
            .extensions()
            .get::<RequestContext>()
            .cloned()
            .ok_or(tonic::Status::unauthenticated("No context found"))?;
        let svc = new_user_accounts_service(ctx);
        let req = request.into_inner();
        let Some(user) = req.user else {
            return Err(tonic::Status::invalid_argument("No user info found"));
        };
        let edit_req = crate::dto::user_accounts::RegisterReq {
            username: user.username.into(),
            email: user.email.into(),
            password: user.password.into(),
            age: user.age,
            gender: user.gender,
            education: user.education,
            hometown: user.hometown,
            address: user.address,
        };
        let user_id = req.id;
        let res = svc.edit_info(edit_req, user_id).await;
        let resp = make_response(res);
        let response = crate::proto_gen::proto_api::RegisterResponse {
            code: resp.code,
            msg: resp.msg,
            data: resp
                .data
                .map(|user| crate::proto_gen::proto_api::UserResponse {
                    id: user.id,
                    username: user.username.to_string(),
                    email: user.email.to_string(),
                    created_at: user.created_at.map(|dt| Timestamp {
                        seconds: dt.and_utc().timestamp(),
                        nanos: dt.and_utc().timestamp_subsec_nanos() as i32,
                    }),
                    last_login_time: user.last_login_time.map(|dt| Timestamp {
                        seconds: dt.and_utc().timestamp(),
                        nanos: dt.and_utc().timestamp_subsec_nanos() as i32,
                    }),
                }),
        };
        Ok(tonic::Response::new(response))
    }

    async fn login(
        &self,
        request: tonic::Request<crate::proto_gen::proto_api::LoginRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::proto_gen::proto_api::AuthResponse>,
        tonic::Status,
    > {
        let ctx = request
            .extensions()
            .get::<RequestContext>()
            .cloned()
            .ok_or(tonic::Status::unauthenticated("No context found"))?;
        let svc = new_user_accounts_service(ctx);
        let req = request.into_inner();
        let email = req.email;
        let password = req.password;
        let res = svc.login(email.into(), password.into()).await;
        let resp = make_response(res);
        let response = crate::proto_gen::proto_api::AuthResponse {
            code: resp.code,
            msg: resp.msg,
            data: resp
                .data
                .map(|data| crate::proto_gen::proto_api::AuthBodyResponse {
                    access_token: data.access_token,
                    token_type: data.token_type,
                }),
        };
        Ok(tonic::Response::new(response))
    }
}
