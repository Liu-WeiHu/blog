use super::{LazyLock, PgPool, PgPoolOptions, jwt::Keys};

pub static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

pub async fn get_db_pool() -> PgPool {
    let dsn = std::env::var("DATABASE_URL").expect("DB_DSN must be set");
    PgPoolOptions::new()
        .connect(&dsn)
        .await
        .expect("db connect is error")
}
