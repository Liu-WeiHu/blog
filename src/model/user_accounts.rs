use crate::NaiveDateTime;
use crate::Serialize;

#[derive(Serialize, Clone, Debug, Default)]
pub struct UserAccounts {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: Option<NaiveDateTime>,
    pub last_login_time: Option<NaiveDateTime>,
}
