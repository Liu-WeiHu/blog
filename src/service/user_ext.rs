use crate::{
    dao::user_ext::{UserExtDao, new_user_ext_dao},
    model::user_ext::UserExt,
    response::ErrCode,
};

use sqlx::PgPool;

pub trait UserExtService: Send + Sync + Clone {
    fn one(
        &self,
        user_id: i32,
    ) -> impl std::future::Future<Output = Result<UserExt, ErrCode>> + std::marker::Send;
}

#[derive(Clone)]
struct UserExtServiceI<DAO: UserExtDao> {
    dao: DAO,
}

pub fn new_user_ext_service(pool: PgPool) -> impl UserExtService {
    let dao = new_user_ext_dao(pool);
    UserExtServiceI { dao }
}

impl<DAO: UserExtDao> UserExtService for UserExtServiceI<DAO> {
    async fn one(&self, user_id: i32) -> Result<UserExt, ErrCode> {
        tracing::debug!("UserExtService.one user_id = {}", user_id);
        self.dao.select_by_id(user_id).await.map_err(|err| {
            tracing::error!("db err = {}", err);
            ErrCode::InternalError
        })
    }
}
