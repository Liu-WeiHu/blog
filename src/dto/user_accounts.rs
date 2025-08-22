use std::borrow::Cow;

use chrono::NaiveDateTime;
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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CacheUser {
    pub id: i32,
    pub username: Cow<'static, str>,
    pub email: Cow<'static, str>,
    pub created_at: Option<NaiveDateTime>,
    pub last_login_time: Option<NaiveDateTime>,
    pub role_ids: Vec<i32>,
}
