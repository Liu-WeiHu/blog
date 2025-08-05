use super::{
    Body, Deserialize, FromRequest, Json, Request, Serialize,
    response::{self, ErrCode, Resp},
};

#[derive(Deserialize, Serialize)]
pub struct Pagination {
    pub offset: i64,
    pub limit: i64,
}

#[derive(Deserialize)]
pub struct PageParams {
    pub page: Option<i64>,
    pub size: Option<i64>,
}

impl<S> FromRequest<S> for Pagination
where
    S: Send + Sync,
{
    type Rejection = Resp<()>;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let boby = Json::<PageParams>::from_request(req, state).await;
        let page_params = match boby {
            Ok(Json(page_params)) => page_params,
            Err(_) => PageParams {
                page: None,
                size: None,
            },
        };

        let limit = page_params.size.unwrap_or(10);
        let page = page_params.page.unwrap_or(1);
        if page < 1 {
            return Err(response::make_response(Err(ErrCode::InputArgsError)));
        }
        if !(1..=100).contains(&limit) {
            return Err(response::make_response(Err(ErrCode::InputArgsError)));
        }
        let offset = (page - 1) * limit;
        Ok(Pagination { offset, limit })
    }
}
