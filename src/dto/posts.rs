use std::borrow::Cow;

#[derive(serde::Deserialize)]
pub struct AddPostsReq {
    pub title: Cow<'static, str>,
    pub content: Cow<'static, str>,
}
