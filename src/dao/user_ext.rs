use crate::model::user_ext::UserExt;

use sqlx::PgConnection;
use async_trait::async_trait;

#[async_trait]
pub trait UserExtDao: Send + Sync {
    async fn select_by_id(
        &self,
        executor: &mut PgConnection,
        user_id: i32,
    ) -> Result<UserExt, sqlx::Error>;
    async fn insert(
        &self,
        executor: &mut PgConnection,
        user: UserExt,
    ) -> Result<UserExt, sqlx::Error>;
    async fn update_by_user_id(
        &self,
        executor: &mut PgConnection,
        user: UserExt,
    ) -> Result<UserExt, sqlx::Error>;
}

struct UserExtDaoI;

pub fn new_user_ext_dao() -> impl UserExtDao {
    UserExtDaoI {}
}

#[async_trait]
impl UserExtDao for UserExtDaoI {
    async fn select_by_id(
        &self,
        executor: &mut PgConnection,
        user_id: i32,
    ) -> Result<UserExt, sqlx::Error> {
        sqlx::query_as!(
            UserExt,
            "select * from user_ext where user_id = $1",
            user_id
        )
        .fetch_one(executor)
        .await
        .or_else(|err| match err {
            sqlx::Error::RowNotFound => Ok(UserExt::default()),
            other => Err(other),
        })
    }

    async fn insert(
        &self,
        executor: &mut PgConnection,
        user: UserExt,
    ) -> Result<UserExt, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO user_ext (user_id, age, gender, education, hometown, address) VALUES ($1, $2, $3, $4, $5, $6)",
            user.user_id,
            user.age,
            user.gender.as_deref(),
            user.education.as_deref(),
            user.hometown.as_deref(),
            user.address.as_deref())
        .execute(executor)
        .await?;
        Ok(user)
    }

    async fn update_by_user_id(
        &self,
        executor: &mut PgConnection,
        user: UserExt,
    ) -> Result<UserExt, sqlx::Error> {
        sqlx::query!(
            "update user_ext set age = $1, gender = $2, education = $3, hometown = $4, address = $5 where user_id = $6",
            user.age,
            user.gender.as_deref(),
            user.education.as_deref(),
            user.hometown.as_deref(),
            user.address.as_deref(),
            user.user_id)
        .execute(executor)
        .await?;
        Ok(user)
    }
}
