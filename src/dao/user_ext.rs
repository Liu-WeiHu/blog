use crate::model::user_ext::UserExt;

use sqlx::PgPool;

pub trait UserExtDao: Send + Sync + Clone {
    fn select_by_id(
        &self,
        user_id: i32,
    ) -> impl std::future::Future<Output = Result<UserExt, sqlx::Error>> + std::marker::Send;
    fn insert(
        &self,
        user: UserExt,
    ) -> impl std::future::Future<Output = Result<UserExt, sqlx::Error>> + std::marker::Send;
    fn update_by_id(
        &self,
        user: UserExt,
    ) -> impl std::future::Future<Output = Result<UserExt, sqlx::Error>> + std::marker::Send;
}

#[derive(Clone)]
struct UserExtDaoI {
    pool: PgPool,
}

pub fn new_user_ext_dao(pool: PgPool) -> impl UserExtDao {
    UserExtDaoI { pool }
}

impl UserExtDao for UserExtDaoI {
    async fn select_by_id(&self, user_id: i32) -> Result<UserExt, sqlx::Error> {
        sqlx::query_as::<_, UserExt>("select * from user_ext where user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
    }

    async fn insert(&self, user: UserExt) -> Result<UserExt, sqlx::Error> {
        sqlx::query(
            "INSERT INTO user_ext (user_id, age, gender, education, hometown, address) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind( user.user_id)
            .bind( user.age )
            .bind( &user.gender )
            .bind( &user.education )
            .bind( &user.hometown )
            .bind( &user.address )
        .execute(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_by_id(&self, user: UserExt) -> Result<UserExt, sqlx::Error> {
        sqlx::query(
            "update user_ext set age = $1, gender = $2, education = $3, hometown = $4, address = $5 where user_id = $6")
            .bind( user.age )
            .bind( &user.gender )
            .bind( &user.education )
            .bind( &user.hometown )
            .bind( &user.address )
            .bind( user.user_id )
        .execute(&self.pool)
        .await?;
        Ok(user)
    }
}
