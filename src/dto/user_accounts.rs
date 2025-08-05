use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterReq {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub email: String,
    pub password: String,
}
