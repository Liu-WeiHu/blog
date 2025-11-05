use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize, Debug, Clone, Copy)]
pub enum ErrCode {
    InternalError,
    InvalidToken,
    InputArgsError,
    UnAuthorized,
    InputNameInvalid,
    InputEmailInvalid,
    InputPasswordInvalid,
    InputLoginInvalid,
    EmailAlreadyRegistered,
    DbServiceUnavailable,
    RedisServiceUnavailable,
    UnPermission,
}

impl ErrCode {
    /// 获取错误码
    const fn code(self) -> i32 {
        match self {
            Self::InternalError => 100001,
            Self::DbServiceUnavailable => 100002,
            Self::RedisServiceUnavailable => 100003,
            Self::InvalidToken => 200001,
            Self::UnAuthorized => 200002,
            Self::UnPermission => 200003,
            Self::InputArgsError => 300001,
            Self::InputNameInvalid => 300002,
            Self::InputEmailInvalid => 300003,
            Self::InputPasswordInvalid => 300004,
            Self::InputLoginInvalid => 300005,
            Self::EmailAlreadyRegistered => 300006,
        }
    }

    /// 获取错误消息
    const fn message(self) -> &'static str {
        match self {
            Self::InternalError => "服务器内部错误",
            Self::DbServiceUnavailable => "数据库服务不可用",
            Self::RedisServiceUnavailable => "redis服务不可用",
            Self::InvalidToken => "无效的token",
            Self::UnAuthorized => "没有授权",
            Self::UnPermission => "权限不足",
            Self::InputArgsError => "入参错误",
            Self::InputNameInvalid => "输入名字无效",
            Self::InputEmailInvalid => "输入邮箱无效",
            Self::InputPasswordInvalid => "输入密码无效",
            Self::InputLoginInvalid => "输入邮箱或密码错误",
            Self::EmailAlreadyRegistered => "邮箱已被注册",
        }
    }
}

#[derive(Serialize)]
pub struct Resp<T: Serialize> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

pub fn make_response<T: Serialize>(input: Result<T, ErrCode>) -> Resp<T> {
    match input {
        Ok(data) => Resp {
            code: 0,
            msg: String::new(),
            data: Some(data),
        },
        Err(err_code) => Resp {
            code: err_code.code(),
            msg: err_code.message().to_string(),
            data: None,
        },
    }
}

impl<T: Serialize> IntoResponse for Resp<T> {
    fn into_response(self) -> Response {
        let status = match self.code {
            0 => StatusCode::OK,
            200001..=299999 => StatusCode::UNAUTHORIZED,
            300001..=399999 => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, axum::Json(self)).into_response()
    }
}