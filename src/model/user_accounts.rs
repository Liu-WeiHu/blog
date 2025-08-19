use std::borrow::Cow;

use chrono::NaiveDateTime;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct UserAccounts {
    pub id: i32,
    pub username: Cow<'static, str>,
    pub email: Cow<'static, str>,
    #[serde(skip_serializing, skip_deserializing)]
    pub password: Cow<'static, str>,
    pub created_at: Option<NaiveDateTime>,
    pub last_login_time: Option<NaiveDateTime>,
}
