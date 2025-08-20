use std::borrow::Cow;

use crate::{dto::user_accounts::UserInfo, model::user_accounts::UserAccounts};

use sqlx::PgConnection;

#[allow(dead_code)]
pub trait UserAccountsDao: Send + Sync {
    fn select_all(
        &self,
        executor: &mut PgConnection,
        offset: i64,
        limit: i64,
    ) -> impl std::future::Future<Output = Result<Vec<UserAccounts>, sqlx::Error>> + std::marker::Send;
    fn select_one(
        &self,
        executor: &mut PgConnection,
        user_id: i32,
    ) -> impl std::future::Future<Output = Result<UserAccounts, sqlx::Error>> + std::marker::Send;
    fn select_by_id(
        &self,
        executor: &mut PgConnection,
        user_id: i32,
    ) -> impl std::future::Future<Output = Result<UserInfo, sqlx::Error>> + std::marker::Send;
    fn insert(
        &self,
        executor: &mut PgConnection,
        user: UserAccounts,
    ) -> impl std::future::Future<Output = Result<UserAccounts, sqlx::Error>> + std::marker::Send;
    fn select_by_email(
        &self,
        executor: &mut PgConnection,
        email: Cow<'static, str>,
    ) -> impl std::future::Future<Output = Result<UserAccounts, sqlx::Error>> + std::marker::Send;
    fn update_login_time_by_id(
        &self,
        executor: &mut PgConnection,
        user: UserAccounts,
    ) -> impl std::future::Future<Output = Result<UserAccounts, sqlx::Error>> + std::marker::Send;
    fn update(
        &self,
        executor: &mut PgConnection,
        user: UserAccounts,
    ) -> impl std::future::Future<Output = Result<UserAccounts, sqlx::Error>> + std::marker::Send;
}

struct UserAccountsDaoI;

pub fn new_user_accounts_dao() -> impl UserAccountsDao {
    UserAccountsDaoI {}
}

impl UserAccountsDao for UserAccountsDaoI {
    async fn select_all(
        &self,
        executor: &mut PgConnection,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<UserAccounts>, sqlx::Error> {
        sqlx::query_as!(
            UserAccounts,
            "select * from user_accounts limit $1 offset $2",
            limit,
            offset
        )
        .fetch_all(executor)
        .await
    }

    async fn select_one(
        &self,
        executor: &mut PgConnection,
        user_id: i32,
    ) -> Result<UserAccounts, sqlx::Error> {
        sqlx::query_as!(
            UserAccounts,
            "select * from user_accounts where id = $1",
            user_id
        )
        .fetch_one(executor)
        .await
    }

    async fn select_by_id(
        &self,
        executor: &mut PgConnection,
        user_id: i32,
    ) -> Result<UserInfo, sqlx::Error> {
        sqlx::query_as!(UserInfo,
            "select ua.id, ua.username, ua.email, ue.age , ue.gender, ue.education, ue.hometown, ue.address from user_accounts ua join user_ext ue on ua.id = ue.user_id where ua.id = $1"
        , user_id)
        .fetch_one(executor)
        .await
    }

    async fn insert(
        &self,
        executor: &mut PgConnection,
        mut user: UserAccounts,
    ) -> Result<UserAccounts, sqlx::Error> {
        struct InsertResult {
            id: i32,
        }

        let res = sqlx::query_as!(
                    InsertResult,
                    "INSERT INTO user_accounts (username, email, password, created_at) VALUES ($1, $2, $3, NOW())
                    RETURNING id",
                    user.username.as_ref(),
                    user.email.as_ref(),
                    user.password.as_ref(),
                )
                .fetch_one(executor)
                .await?;
        user.id = res.id;
        Ok(user)
    }

    async fn select_by_email(
        &self,
        executor: &mut PgConnection,
        email: Cow<'static, str>,
    ) -> Result<UserAccounts, sqlx::Error> {
        sqlx::query_as!(
            UserAccounts,
            "select * from user_accounts where email = $1",
            email.as_ref()
        )
        .fetch_one(executor)
        .await
    }

    async fn update_login_time_by_id(
        &self,
        executor: &mut PgConnection,
        user: UserAccounts,
    ) -> Result<UserAccounts, sqlx::Error> {
        sqlx::query!(
            "update user_accounts set last_login_time = $1 where id = $2",
            user.last_login_time,
            user.id,
        )
        .execute(executor)
        .await?;
        Ok(user)
    }

    async fn update(
        &self,
        executor: &mut PgConnection,
        user: UserAccounts,
    ) -> Result<UserAccounts, sqlx::Error> {
        sqlx::query!(
            "update user_accounts set username = $1 where id = $2",
            user.username.as_ref(),
            user.id
        )
        .execute(executor)
        .await?;
        Ok(user)
    }
}
