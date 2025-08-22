use crate::{
    context::Context,
    jwt::Keys,
    rbac::PermissionRegistry,
    service::permission::{RbacService, new_rbac_service},
};

use redis::Client;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::LazyLock;

pub static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

pub async fn get_db_pool() -> PgPool {
    let dsn = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .connect(&dsn)
        .await
        .expect("db connect is error")
}

pub async fn get_redis_client() -> Client {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    redis::Client::open(redis_url).expect("redis connect is error")
}

pub async fn cache_rbac<C: Context>(ctx: C) -> PermissionRegistry {
    new_rbac_service(ctx)
        .get_rbac_permission()
        .await
        .expect("get rbac permission error")
}
