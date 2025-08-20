use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct RegisterReq {
    pub username: Cow<'static, str>,
    pub email: Cow<'static, str>,
    pub password: Cow<'static, str>,

    pub age: Option<i32>,
    pub gender: Option<String>,
    pub education: Option<String>,
    pub hometown: Option<String>,
    pub address: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateUserInfoReq {
    pub user_id: i32,
    pub username: Cow<'static, str>,

    pub age: Option<i32>,
    pub gender: Option<String>,
    pub education: Option<String>,
    pub hometown: Option<String>,
    pub address: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub email: Cow<'static, str>,
    pub password: Cow<'static, str>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct UserInfo {
    pub id: i32,
    pub username: Cow<'static, str>,
    pub email: Cow<'static, str>,
    pub age: Option<i32>,
    pub gender: Option<String>,
    pub education: Option<String>,
    pub hometown: Option<String>,
    pub address: Option<String>,
}
