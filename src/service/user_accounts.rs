use crate::PgPool;
use crate::dao::user_accounts::UserAccountsDao;
use crate::dao::user_accounts::new_user_accounts_dao;
use crate::debug;
use crate::model::user_accounts::UserAccounts;
use crate::pagination::Pagination;
use crate::response::ErrCode;

pub trait UserAccountsService: Send + Sync + Clone {
    fn list(
        &self,
        pag: Pagination,
    ) -> impl std::future::Future<Output = Result<Vec<UserAccounts>, ErrCode>> + std::marker::Send;
    fn one(
        &self,
        user_id: i32,
    ) -> impl std::future::Future<Output = Result<UserAccounts, ErrCode>> + std::marker::Send;
}

#[derive(Clone)]
struct UserAccountsServiceI<DAO: UserAccountsDao> {
    dao: DAO,
}

pub fn new_user_accounts_service(pool: PgPool) -> impl UserAccountsService {
    let dao = new_user_accounts_dao(pool);
    UserAccountsServiceI { dao }
}

impl<DAO: UserAccountsDao> UserAccountsService for UserAccountsServiceI<DAO> {
    async fn list(&self, pag: Pagination) -> Result<Vec<UserAccounts>, ErrCode> {
        debug!(
            "UserAccountsService.list offset = {}, limit = {}",
            pag.offset, pag.limit
        );
        match self.dao.select_all(pag.offset, pag.limit).await {
            Ok(users) => Ok(users),
            Err(_) => Err(ErrCode::InternalError),
        }
    }

    async fn one(&self, user_id: i32) -> Result<UserAccounts, ErrCode> {
        debug!("UserAccountsService.one user_id = {}", user_id);
        match self.dao.select_by_id(user_id).await {
            Ok(user) => Ok(user),
            Err(sqlx::Error::RowNotFound) => Err(ErrCode::InputArgsError),
            Err(_) => Err(ErrCode::InternalError),
        }
    }
}
