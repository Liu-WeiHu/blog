use std::borrow::Cow;

use serde::Deserialize;

use crate::rbac::PermissionPoints;

#[derive(Deserialize)]
pub struct RbacRolePermission {
    pub id: i32,
    pub role_name: Cow<'static, str>,
    pub permission_name: PermissionPoints,
}

#[derive(Debug, Deserialize, Default)]
pub enum PermissionType {
    #[default]
    Operation, // 操作权限
    Visual, // 可视权限
}

impl From<String> for PermissionType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "operation" => PermissionType::Operation,
            "visual" => PermissionType::Visual,
            _ => PermissionType::Operation, // 默认值
        }
    }
}

impl AsRef<str> for PermissionType {
    fn as_ref(&self) -> &str {
        match self {
            PermissionType::Operation => "operation",
            PermissionType::Visual => "visual",
        }
    }
}
