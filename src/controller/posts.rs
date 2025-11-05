use axum::{Extension, Json, extract::Path, response::IntoResponse};

use crate::{
    context::RequestContext,
    dto::posts::AddPostsReq,
    model::posts::Posts,
    pagination::Pagination,
    response,
    service::posts::{PostsService, new_posts_service},
};

pub async fn list(
    Extension(ctx): Extension<RequestContext>,
    pagination: Pagination,
) -> impl IntoResponse {
    let svc = new_posts_service(ctx);
    let res = svc.list(pagination).await;
    response::make_response(res)
}

// #[axum::debug_handler]
pub async fn one(
    Extension(ctx): Extension<RequestContext>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let svc = new_posts_service(ctx);
    let res = svc.one(id).await;
    response::make_response(res)
}

pub async fn add(
    Extension(ctx): Extension<RequestContext>,
    Json(req): Json<AddPostsReq>,
) -> impl IntoResponse {
    let svc = new_posts_service(ctx);
    let posts = Posts {
        title: req.title,
        content: req.content,
        ..Default::default()
    };
    let res = svc.add(posts).await;
    response::make_response(res)
}

pub async fn edit(
    Extension(ctx): Extension<RequestContext>,
    Path(id): Path<i32>,
    Json(req): Json<AddPostsReq>,
) -> impl IntoResponse {
    let svc = new_posts_service(ctx);
    let posts = Posts {
        title: req.title,
        content: req.content,
        ..Default::default()
    };
    let res = svc.edit(posts, id).await;
    response::make_response(res)
}
