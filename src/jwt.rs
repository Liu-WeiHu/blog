use crate::{DecodingKey, Deserialize, EncodingKey, Serialize};

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Serialize)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}

impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

// impl<S> FromRequestParts<S> for Claims
// where
//     S: Send + Sync,
// {
//     type Rejection = Resp<()>;
//
//     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
//         let token = parts
//             .headers
//             .get("authorization")
//             .ok_or_else(|| response::make_response(Err(ErrCode::UnAuthorized)))?
//             .to_str()
//             .map_err(|_| response::make_response(Err(ErrCode::UnAuthorized)))?
//             .strip_prefix("Bearer ")
//             .ok_or_else(|| response::make_response(Err(ErrCode::UnAuthorized)))?;
//
//         debug!("Extracted token: {}", token);
//
//         let token_data =
//             decode::<Claims>(token, &KEYS.decoding, &Validation::default()).map_err(|e| {
//                 error!("JWT decode error: {}", e);
//                 response::make_response(Err(ErrCode::InvalidToken))
//             })?;
//
//         Ok(token_data.claims)
//     }
// }
