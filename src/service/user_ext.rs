use crate::{
    context::{AsyncContext},
    dao::user_ext::{new_user_ext_dao, UserExtDao},
    model::user_ext::UserExt,
    response::ErrCode,
    service::handle_error,
};
use async_trait::async_trait;

#[async_trait]
pub trait UserExtService: Send + Sync + Clone {
    async fn one(&self, user_id: i32) -> Result<UserExt, ErrCode>;
}

#[derive(Clone)]
struct UserExtServiceI<Ctx: AsyncContext> {
    ctx: Ctx,
}

pub fn new_user_ext_service<Ctx: AsyncContext>(ctx: Ctx) -> impl UserExtService {
    UserExtServiceI { ctx }
}

#[async_trait]
impl<Ctx: AsyncContext> UserExtService for UserExtServiceI<Ctx> {
    #[tracing::instrument(skip(self), fields(user_id = %user_id))]
    async fn one(&self, user_id: i32) -> Result<UserExt, ErrCode> {
        tracing::debug!("get user_ext");
        let mut conn = self.ctx.get_db_conn().await?;
        new_user_ext_dao()
            .select_by_id(&mut conn, user_id)
            .await
            .map_err(|err| handle_error(Box::new(err), "dao select_by_id"))
    }
}
