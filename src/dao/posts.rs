use sqlx::PgConnection;

use crate::model::posts::Posts;

#[allow(dead_code)]
pub trait PostsDao: Send + Sync {
    fn select_all(
        &self,
        executor: &mut PgConnection,
        offset: i64,
        limit: i64,
    ) -> impl std::future::Future<Output = Result<Vec<Posts>, sqlx::Error>> + std::marker::Send;
    fn select_one(
        &self,
        executor: &mut PgConnection,
        user_id: i32,
    ) -> impl std::future::Future<Output = Result<Posts, sqlx::Error>> + std::marker::Send;
    fn insert(
        &self,
        executor: &mut PgConnection,
        posts: Posts,
    ) -> impl std::future::Future<Output = Result<Posts, sqlx::Error>> + std::marker::Send;
    fn update(
        &self,
        executor: &mut PgConnection,
        posts: Posts,
    ) -> impl std::future::Future<Output = Result<Posts, sqlx::Error>> + std::marker::Send;
}

struct PostsDaoI;

pub fn new_posts_dao() -> impl PostsDao {
    PostsDaoI {}
}

impl PostsDao for PostsDaoI {
    async fn select_all(
        &self,
        executor: &mut PgConnection,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Posts>, sqlx::Error> {
        sqlx::query_as!(
            Posts,
            "select * from posts limit $1 offset $2",
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
    ) -> Result<Posts, sqlx::Error> {
        sqlx::query_as!(Posts, "select * from posts where id = $1", user_id)
            .fetch_one(executor)
            .await
            .or_else(|err| match err {
                sqlx::Error::RowNotFound => Ok(Posts::default()),
                other => Err(other),
            })
    }

    async fn insert(
        &self,
        executor: &mut PgConnection,
        mut posts: Posts,
    ) -> Result<Posts, sqlx::Error> {
        struct InsertResult {
            id: i32,
        }

        let res = sqlx::query_as!(
            InsertResult,
            "INSERT INTO posts (title, content, user_id, status) VALUES ($1, $2, $3, $4)
                    RETURNING id",
            posts.title.as_ref(),
            posts.content.as_ref(),
            posts.user_id,
            posts.status.as_ref(),
        )
        .fetch_one(executor)
        .await?;
        posts.id = res.id;
        Ok(posts)
    }

    async fn update(
        &self,
        executor: &mut PgConnection,
        posts: Posts,
    ) -> Result<Posts, sqlx::Error> {
        sqlx::query!(
            "update posts set title = $1, content = $2 where id = $3",
            posts.title.as_ref(),
            posts.content.as_ref(),
            posts.user_id
        )
        .execute(executor)
        .await?;
        Ok(posts)
    }
}
