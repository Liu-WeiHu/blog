use std::borrow::Cow;

use chrono::NaiveDateTime;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub enum PostsStatus {
    #[default]
    Published,
    Draft,
    Deleted,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct Posts {
    pub id: i32,
    pub title: Cow<'static, str>,
    pub content: Cow<'static, str>,
    pub user_id: i32,
    pub status: PostsStatus,
    pub created_at: Option<NaiveDateTime>,
}

// 实现 From<String> trait
impl From<String> for PostsStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "published" => PostsStatus::Published,
            "draft" => PostsStatus::Draft,
            "deleted" => PostsStatus::Deleted,
            _ => PostsStatus::Published, // 默认值
        }
    }
}

impl AsRef<str> for PostsStatus {
    fn as_ref(&self) -> &str {
        match self {
            PostsStatus::Published => "published",
            PostsStatus::Draft => "draft",
            PostsStatus::Deleted => "deleted",
        }
    }
}
