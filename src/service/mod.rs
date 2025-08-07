use sqlx::{PgPool, Postgres, Transaction, pool::PoolConnection};

macro_rules! map_to_user_ext {
    ($req:expr) => {
        UserExt {
            user_id: $req.user_id,
            age: $req.age,
            gender: $req.gender,
            education: $req.education,
            hometown: $req.hometown,
            address: $req.address,
            ..Default::default()
        }
    };

    ($req:expr, $user_id:expr) => {
        UserExt {
            user_id: $user_id,
            age: $req.age,
            gender: $req.gender,
            education: $req.education,
            hometown: $req.hometown,
            address: $req.address,
            ..Default::default()
        }
    };
}

pub mod user_accounts;
pub mod user_ext;

pub async fn get_tx(pool: PgPool) -> Result<Transaction<'static, Postgres>, sqlx::Error> {
    pool.begin().await
}

pub async fn get_conn(pool: PgPool) -> Result<PoolConnection<Postgres>, sqlx::Error> {
    pool.acquire().await
}
