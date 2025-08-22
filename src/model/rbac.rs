use serde::Deserialize;

#[derive(Deserialize, Default)]
#[allow(dead_code)]
pub struct UserRole {
    pub user_id: i32,
    pub role_id: i32,
}
