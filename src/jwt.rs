use super::{
    Authorization, Bearer, DecodingKey, Deserialize, EncodingKey, FromRequestParts, Parts,
    RequestPartsExt, Serialize, TypedHeader, Validation, decode,
    init::KEYS,
    response::{self, ErrCode, Resp},
};

pub struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

#[derive(Serialize)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = Resp<()>;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| response::make_response(Err(ErrCode::InvalidToken)))?;

        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| response::make_response(Err(ErrCode::InvalidToken)))?;

        Ok(token_data.claims)
    }
}
