use crate::{
    context::{AsyncContext},
    dao::posts::{new_posts_dao, PostsDao},
    model::posts::Posts,
    pagination::Pagination,
    rbac::PermissionPoints,
    response::ErrCode,
    service::handle_error,
};
use async_trait::async_trait;

#[async_trait]
pub trait PostsService: Send + Sync + Clone {
    async fn list(
        &self,
        pag: Pagination,
    ) -> Result<Vec<Posts>, ErrCode>;
    async fn one(&self, id: i32) -> Result<Posts, ErrCode>;
    async fn edit(
        &self,
        req: Posts,
        id: i32,
    ) -> Result<Posts, ErrCode>;
    async fn add(&self, req: Posts) -> Result<Posts, ErrCode>;
}

#[derive(Clone)]
struct PostsServiceI<Ctx: AsyncContext> {
    ctx: Ctx,
}

pub fn new_posts_service<Ctx: AsyncContext>(ctx: Ctx) -> impl PostsService {
    PostsServiceI { ctx }
}

#[async_trait]
impl<Ctx: AsyncContext> PostsService for PostsServiceI<Ctx> {
    #[tracing::instrument(skip(self), fields(offset = pag.offset, limit = pag.limit))]
    async fn list(&self, pag: Pagination) -> Result<Vec<Posts>, ErrCode> {
        tracing::debug!("Listing posts with pagination");

        self.ctx.can_access(PermissionPoints::ListPost)?;
        let mut conn = self.ctx.get_db_conn().await?;
        new_posts_dao()
            .select_all(&mut conn, pag.offset, pag.limit)
            .await
            .map_err(|err| handle_error(Box::new(err), "dao select_all"))
    }

    #[tracing::instrument(skip(self), fields(id = %id))]
    async fn one(&self, id: i32) -> Result<Posts, ErrCode> {
        tracing::debug!("Getting posts");

        self.ctx.can_access(PermissionPoints::GetPost)?;
        let mut conn = self.ctx.get_db_conn().await?;
        new_posts_dao()
            .select_one(&mut conn, id)
            .await
            .map_err(|err| handle_error(Box::new(err), "dao select_one"))
    }

    #[tracing::instrument(skip(self), fields(id = id, title = %req.title))]
    async fn edit(&self, mut req: Posts, id: i32) -> Result<Posts, ErrCode> {
        tracing::debug!("Editing posts");

        self.ctx.can_access(PermissionPoints::EditPost)?;
        if id <= 0 {
            return Err(ErrCode::InputArgsError);
        }

        req.id = id;

        let user = self.ctx.get_user().ok_or(ErrCode::UnAuthorized)?;
        req.user_id = user.id;
        let mut conn = self.ctx.get_db_conn().await?;
        new_posts_dao()
            .update(&mut conn, req.clone())
            .await
            .map_err(|err| handle_error(Box::new(err), "dao update"))?;
        Ok(req)
    }

    #[tracing::instrument(skip(self), fields(title = %req.title))]
    async fn add(&self, mut req: Posts) -> Result<Posts, ErrCode> {
        tracing::debug!("Create posts");

        self.ctx.can_access(PermissionPoints::CreatePost)?;

        let user = self.ctx.get_user().ok_or(ErrCode::UnAuthorized)?;
        let mut conn = self.ctx.get_db_conn().await?;
        req.user_id = user.id;
        new_posts_dao()
            .insert(&mut conn, req.clone())
            .await
            .map_err(|err| handle_error(Box::new(err), "dao insert"))?;
        Ok(req)
    }
}
