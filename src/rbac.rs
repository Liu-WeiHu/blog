use std::{collections::{HashMap, HashSet}, sync::Arc};

// 就初始化一次,然后只读不写,不涉及数据安全问题.
// PermissionRegistry 将每个权限点映射到一份 PermissionEntry，
// 内含允许的 role id 集合与是否允许匿名访问的标记。
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct PermissionEntry {
    pub role_ids: HashSet<i32>,
    pub allow_anonymous: bool,
}

pub type PermissionRegistry = Arc<HashMap<PermissionPoints, PermissionEntry>>;

#[derive(Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize, Debug)]
pub enum PermissionPoints {
    // 可视权限点
    ViewPostCreation, // 查看文章创建按钮
    ViewPostEdit,     // 查看文章编辑按钮
    ViewPostDetail,   // 查看文章详情按钮
    ViewPostDelete,   // 查看文章删除按钮

    // 操作权限点
    CreatePost, // 创建文章
    EditPost,   // 编辑文章
    GetPost,    // 查看文章详情
    DeletePost, // 删除文章
    ListPost,   // 列表文章

    Unknown, // 未知权限点
}

impl From<String> for PermissionPoints {
    fn from(s: String) -> Self {
        match s.as_str() {
            "view_post_creation" => PermissionPoints::ViewPostCreation,
            "view_post_edit" => PermissionPoints::ViewPostEdit,
            "view_post_detail" => PermissionPoints::ViewPostDetail,
            "view_post_delete" => PermissionPoints::ViewPostDelete,
            "create_post" => PermissionPoints::CreatePost,
            "edit_post" => PermissionPoints::EditPost,
            "get_post" => PermissionPoints::GetPost,
            "delete_post" => PermissionPoints::DeletePost,
            "list_post" => PermissionPoints::ListPost,
            _ => PermissionPoints::Unknown,
        }
    }
}

impl AsRef<str> for PermissionPoints {
    fn as_ref(&self) -> &str {
        match self {
            PermissionPoints::ViewPostCreation => "view_post_creation",
            PermissionPoints::ViewPostEdit => "view_post_edit",
            PermissionPoints::ViewPostDetail => "view_post_detail",
            PermissionPoints::ViewPostDelete => "view_post_delete",
            PermissionPoints::CreatePost => "create_post",
            PermissionPoints::EditPost => "edit_post",
            PermissionPoints::GetPost => "get_post",
            PermissionPoints::DeletePost => "delete_post",
            PermissionPoints::ListPost => "list_post",
            PermissionPoints::Unknown => "unknown",
        }
    }
}
