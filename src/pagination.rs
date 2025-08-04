use super::{
    Deserialize, FromRequestParts, Parts, Query, Serialize,
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

impl<S> FromRequestParts<S> for Pagination
where
    S: Send + Sync,
{
    type Rejection = Resp<()>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 从查询参数提取 PageParams
        let Query(page_params) = Query::<PageParams>::from_request_parts(parts, state)
            .await
            .map_err(|_| response::make_response(Err(ErrCode::InputArgsError)))?;

        let limit = page_params.size.unwrap_or(10);
        let page = page_params.page.unwrap_or(1);
        if page < 1 {
            return Err(response::make_response(Err(ErrCode::InputArgsError)));
        }
        if limit < 1 || limit > 100 {
            return Err(response::make_response(Err(ErrCode::InputArgsError)));
        }
        let offset = (page - 1) * limit;
        Ok(Pagination { offset, limit })
    }
}
