use sqlx::{Postgres, Transaction, pool::PoolConnection};

use crate::{
    dto::user_accounts::CacheUser,
    rbac::{ANONYMOUS, PermissionPoints, PermissionRegistry},
    response::ErrCode,
};

pub trait Context: Clone + Send + Sync + 'static {
    fn get_db_conn(&self)
    -> impl Future<Output = Result<PoolConnection<Postgres>, ErrCode>> + Send;
    fn get_db_tx(
        &self,
    ) -> impl Future<Output = Result<Transaction<'static, Postgres>, ErrCode>> + Send;
    fn get_db_pool(&self) -> &sqlx::PgPool;
    fn get_redis_client(&self) -> &redis::Client;
    fn get_user(&self) -> &Option<CacheUser>;
    fn can_access(&self, operate: PermissionPoints) -> Result<bool, ErrCode>;
}

#[derive(Clone)]
pub struct GlobalContext {
    pool: sqlx::PgPool,
    redis: redis::Client,
    perm: PermissionRegistry,
}

#[allow(dead_code)]
impl Context for GlobalContext {
    fn get_db_pool(&self) -> &sqlx::PgPool {
        &self.pool
    }

    fn get_redis_client(&self) -> &redis::Client {
        &self.redis
    }

    async fn get_db_conn(&self) -> Result<PoolConnection<Postgres>, ErrCode> {
        self.pool.acquire().await.map_err(|e| {
            tracing::error!("Failed to get database connection: {}", e);
            ErrCode::InternalError
        })
    }

    async fn get_db_tx(&self) -> Result<Transaction<'static, Postgres>, ErrCode> {
        self.pool.begin().await.map_err(|e| {
            tracing::error!("Failed to get database transaction: {}", e);
            ErrCode::InternalError
        })
    }

    fn get_user(&self) -> &Option<CacheUser> {
        &None
    }

    fn can_access(&self, _operate: PermissionPoints) -> Result<bool, ErrCode> {
        Err(ErrCode::UnPermission)
    }
}

impl GlobalContext {
    pub fn new(pool: sqlx::PgPool, redis: redis::Client) -> Self {
        Self {
            pool,
            redis,
            perm: PermissionRegistry::default(),
        }
    }

    pub fn set_perm(&mut self, perm: PermissionRegistry) {
        self.perm = perm;
    }
}

#[derive(Clone)]
pub struct RequestContext {
    global_ctx: GlobalContext,
    user: Option<CacheUser>,
}

impl RequestContext {
    pub fn new(ctx: GlobalContext) -> Self {
        Self {
            global_ctx: ctx,
            user: None,
        }
    }

    pub fn set_user(&mut self, user: CacheUser) {
        self.user = Some(user);
    }
}

impl Context for RequestContext {
    async fn get_db_conn(&self) -> Result<PoolConnection<Postgres>, ErrCode> {
        self.global_ctx.get_db_conn().await
    }

    async fn get_db_tx(&self) -> Result<Transaction<'static, Postgres>, ErrCode> {
        self.global_ctx.get_db_tx().await
    }

    fn get_db_pool(&self) -> &sqlx::PgPool {
        self.global_ctx.get_db_pool()
    }

    fn get_redis_client(&self) -> &redis::Client {
        self.global_ctx.get_redis_client()
    }

    fn get_user(&self) -> &Option<CacheUser> {
        &self.user
    }

    // 验证权限
    fn can_access(&self, operate: PermissionPoints) -> Result<bool, ErrCode> {
        let map = self
            .global_ctx
            .perm
            .get(&operate)
            .ok_or(ErrCode::InternalError)?;

        let can_access = match &self.user {
            Some(user) => user
                .role_ids
                .iter()
                .any(|role_id| map.contains_key(role_id)),
            None => map.values().any(|x| *x == ANONYMOUS),
        };

        can_access.then_some(true).ok_or(ErrCode::UnPermission)
    }
}
