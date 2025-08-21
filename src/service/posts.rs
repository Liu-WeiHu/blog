use crate::{
    context::Context,
    dao::posts::{PostsDao, new_posts_dao},
    model::{posts::Posts, user_accounts::UserAccounts},
    pagination::Pagination,
    response::ErrCode,
    service::handle_error,
};

pub trait PostsService: Send + Sync + Clone {
    fn list(
        &self,
        pag: Pagination,
    ) -> impl std::future::Future<Output = Result<Vec<Posts>, ErrCode>> + std::marker::Send;
    fn one(
        &self,
        id: i32,
    ) -> impl std::future::Future<Output = Result<Posts, ErrCode>> + std::marker::Send;
    fn edit(
        &self,
        req: Posts,
        id: i32,
    ) -> impl std::future::Future<Output = Result<Posts, ErrCode>> + std::marker::Send;
    fn add(
        &self,
        req: Posts,
    ) -> impl std::future::Future<Output = Result<Posts, ErrCode>> + std::marker::Send;
}

#[derive(Clone)]
struct PostsServiceI {
    ctx: Context,
}

pub fn new_posts_service(ctx: Context) -> impl PostsService {
    PostsServiceI { ctx }
}

impl PostsService for PostsServiceI {
    #[tracing::instrument(skip(self), fields(offset = pag.offset, limit = pag.limit))]
    async fn list(&self, pag: Pagination) -> Result<Vec<Posts>, ErrCode> {
        tracing::debug!("Listing posts with pagination");

        let mut conn = self.ctx.get_db_conn().await?;
        new_posts_dao()
            .select_all(&mut conn, pag.offset, pag.limit)
            .await
            .map_err(|err| handle_error(Box::new(err), "dao select_all"))
    }

    #[tracing::instrument(skip(self), fields(id = %id))]
    async fn one(&self, id: i32) -> Result<Posts, ErrCode> {
        tracing::debug!("Getting posts");

        let mut conn = self.ctx.get_db_conn().await?;
        new_posts_dao()
            .select_one(&mut conn, id)
            .await
            .map_err(|err| handle_error(Box::new(err), "dao select_one"))
    }

    #[tracing::instrument(skip(self), fields(id = id, title = %req.title))]
    async fn edit(&self, mut req: Posts, id: i32) -> Result<Posts, ErrCode> {
        tracing::debug!("Editing posts");

        if id <= 0 {
            return Err(ErrCode::InputArgsError);
        }

        req.id = id;
        let user = self
            .ctx
            .get::<UserAccounts>()
            .ok_or(ErrCode::InternalError)?;

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
        tracing::debug!("Editing posts");

        let user = self
            .ctx
            .get::<UserAccounts>()
            .ok_or(ErrCode::InternalError)?;

        let mut conn = self.ctx.get_db_conn().await?;
        req.user_id = user.id;
        new_posts_dao()
            .insert(&mut conn, req.clone())
            .await
            .map_err(|err| handle_error(Box::new(err), "dao insert"))?;
        Ok(req)
    }
}
