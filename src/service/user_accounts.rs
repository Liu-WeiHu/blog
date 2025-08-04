use crate::PgPool;
use crate::dao::user_accounts::UserAccountsDao;
use crate::dao::user_accounts::new_user_accounts_dao;
use crate::model::user_accounts::UserAccounts;
use crate::pagination::Pagination;
use crate::response::ErrCode;

pub trait UserAccountsService {
    // async fn list(&self, pag: Pagination) -> Result<Vec<UserAccounts>, ErrCode>;
    async fn one(&self, user_id: i32) -> Result<UserAccounts, ErrCode>;
}

struct UserAccountsServiceI<DAO: UserAccountsDao> {
    dao: DAO,
}

pub fn new_user_accounts_service(pool: PgPool) -> impl UserAccountsService {
    let dao = new_user_accounts_dao(pool);
    UserAccountsServiceI { dao }
}

impl<DAO: UserAccountsDao> UserAccountsService for UserAccountsServiceI<DAO> {
    // async fn list(&self, pag: Pagination) -> Result<Vec<UserAccounts>, ErrCode> {
    //     match self.dao.select_all(pag.offset, pag.limit).await {
    //         Ok(users) => Ok(users),
    //         Err(_) => Err(ErrCode::InternalError),
    //     }
    // }

    async fn one(&self, user_id: i32) -> Result<UserAccounts, ErrCode> {
        match self.dao.select_by_id(user_id).await {
            Ok(user) => Ok(user),
            Err(sqlx::Error::RowNotFound) => Err(ErrCode::InputArgsError),
            Err(_) => Err(ErrCode::InternalError),
        }
    }
}
