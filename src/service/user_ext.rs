use crate::{
    context::Context,
    dao::user_ext::{UserExtDao, new_user_ext_dao},
    model::user_ext::UserExt,
    response::ErrCode,
    service::handle_error,
};

pub trait UserExtService: Send + Sync + Clone {
    fn one(
        &self,
        user_id: i32,
    ) -> impl std::future::Future<Output = Result<UserExt, ErrCode>> + std::marker::Send;
}

#[derive(Clone)]
struct UserExtServiceI {
    ctx: Context,
}

pub fn new_user_ext_service(ctx: Context) -> impl UserExtService {
    UserExtServiceI { ctx }
}

impl UserExtService for UserExtServiceI {
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
