use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct UserExt {
    pub id: i32,
    pub user_id: i32,
    pub age: Option<i32>,
    pub gender: Option<String>,
    pub education: Option<String>,
    pub hometown: Option<String>,
    pub address: Option<String>,
}
