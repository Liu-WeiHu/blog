use crate::PgPool;
use crate::model::user_accounts::UserAccounts;

pub trait UserAccountsDao {
    // async fn select_all(&self, offset: i64, limit: i64) -> Result<Vec<UserAccounts>, sqlx::Error>;
    async fn select_by_id(&self, user_id: i32) -> Result<UserAccounts, sqlx::Error>;
}

struct UserAccountsDaoI {
    pool: PgPool,
}

pub fn new_user_accounts_dao(pool: PgPool) -> impl UserAccountsDao {
    UserAccountsDaoI { pool }
}

impl UserAccountsDao for UserAccountsDaoI {
    // async fn select_all(&self, offset: i64, limit: i64) -> Result<Vec<UserAccounts>, sqlx::Error> {
    //     sqlx::query_as!(
    //         UserAccounts,
    //         "select * from user_accounts limit $1 offset $2",
    //         limit,
    //         offset
    //     )
    //     .fetch_all(&self.pool)
    //     .await
    // }

    async fn select_by_id(&self, user_id: i32) -> Result<UserAccounts, sqlx::Error> {
        sqlx::query_as!(
            UserAccounts,
            "select * from user_accounts where id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await
    }
}
