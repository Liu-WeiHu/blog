use super::{IntoResponse, Response, Serialize, StatusCode};

#[derive(Serialize)]
pub enum ErrCode {
    InternalError,
    InvalidToken,
    InputArgsError,
    UnAuthorized,
}

#[derive(Serialize)]
pub struct Resp<T: Serialize> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

pub fn make_response<T: Serialize>(input: Result<T, ErrCode>) -> Resp<T> {
    let (code, msg, data) = match input {
        Ok(data) => (0, "".to_string(), Some(data)),
        Err(err_code) => match err_code {
            ErrCode::InternalError => (100001, "服务器内部错误".to_string(), None),
            ErrCode::InvalidToken => (200001, "无效的token".to_string(), None),
            ErrCode::UnAuthorized => (200002, "没有授权".to_string(), None),
            ErrCode::InputArgsError => (300001, "入参错误".to_string(), None),
        },
    };

    Resp { code, msg, data }
}

impl<T: Serialize + Send + Sync> IntoResponse for Resp<T> {
    fn into_response(self) -> Response {
        let status = match self.code {
            0 => StatusCode::OK,
            200000..300000 => StatusCode::UNAUTHORIZED,
            300000..400000 => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, axum::Json(self)).into_response()
    }
}
