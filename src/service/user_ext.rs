use crate::{
    dao::user_ext::{UserExtDao, new_user_ext_dao},
    model::user_ext::UserExt,
    response::ErrCode,
    service::get_conn,
};

use sqlx::PgPool;

pub trait UserExtService: Send + Sync + Clone {
    fn one(
        &self,
        user_id: i32,
    ) -> impl std::future::Future<Output = Result<UserExt, ErrCode>> + std::marker::Send;
}

#[derive(Clone)]
struct UserExtServiceI {
    pool: PgPool,
}

pub fn new_user_ext_service(pool: PgPool) -> impl UserExtService {
    UserExtServiceI { pool }
}

impl UserExtService for UserExtServiceI {
    async fn one(&self, user_id: i32) -> Result<UserExt, ErrCode> {
        tracing::debug!("UserExtService.one user_id = {}", user_id);
        let mut conn = get_conn(self.pool.clone()).await.unwrap();
        new_user_ext_dao()
            .select_by_id(&mut conn, user_id)
            .await
            .map_err(|err| {
                tracing::error!("db err = {}", err);
                ErrCode::InternalError
            })
    }
}
