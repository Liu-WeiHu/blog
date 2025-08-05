use crate::model::user_accounts::UserAccounts;

use sqlx::PgPool;

pub trait UserAccountsDao: Send + Sync + Clone {
    fn select_all(
        &self,
        offset: i64,
        limit: i64,
    ) -> impl std::future::Future<Output = Result<Vec<UserAccounts>, sqlx::Error>> + std::marker::Send;
    fn select_by_id(
        &self,
        user_id: i32,
    ) -> impl std::future::Future<Output = Result<UserAccounts, sqlx::Error>> + std::marker::Send;
    fn insert(
        &self,
        user: UserAccounts,
    ) -> impl std::future::Future<Output = Result<UserAccounts, sqlx::Error>> + std::marker::Send;
    fn select_by_email(
        &self,
        email: String,
    ) -> impl std::future::Future<Output = Result<UserAccounts, sqlx::Error>> + std::marker::Send;

    fn update_login_time_by_id(
        &self,
        user: UserAccounts,
    ) -> impl std::future::Future<Output = Result<UserAccounts, sqlx::Error>> + std::marker::Send;
}

#[derive(Clone)]
struct UserAccountsDaoI {
    pool: PgPool,
}

pub fn new_user_accounts_dao(pool: PgPool) -> impl UserAccountsDao {
    UserAccountsDaoI { pool }
}

impl UserAccountsDao for UserAccountsDaoI {
    async fn select_all(&self, offset: i64, limit: i64) -> Result<Vec<UserAccounts>, sqlx::Error> {
        sqlx::query_as!(
            UserAccounts,
            "select * from user_accounts limit $1 offset $2",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn select_by_id(&self, user_id: i32) -> Result<UserAccounts, sqlx::Error> {
        sqlx::query_as!(
            UserAccounts,
            "select * from user_accounts where id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn insert(&self, user: UserAccounts) -> Result<UserAccounts, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO user_accounts (username, email, password, created_at) VALUES ($1, $2, $3, NOW())",
            user.username,
            user.email,
            user.password,
        )
        .execute(&self.pool)
        .await?;

        Ok(user)
    }

    async fn select_by_email(&self, email: String) -> Result<UserAccounts, sqlx::Error> {
        sqlx::query_as!(
            UserAccounts,
            "select * from user_accounts where email = $1",
            email
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn update_login_time_by_id(
        &self,
        user: UserAccounts,
    ) -> Result<UserAccounts, sqlx::Error> {
        sqlx::query!(
            "update user_accounts set last_login_time = $1 where id = $2",
            user.last_login_time,
            user.id,
        )
        .execute(&self.pool)
        .await?;
        Ok(user)
    }
}
