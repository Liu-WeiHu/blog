use std::sync::Arc;

use sqlx::{Postgres, Transaction, pool::PoolConnection};
use async_trait::async_trait;

use crate::{
    dto::user_accounts::CacheUser,
    rbac::{PermissionPoints, PermissionRegistry},
    response::ErrCode,
};

#[allow(dead_code)]
pub trait Context: Clone + Send + Sync + 'static {
    fn get_db_pool(&self) -> &sqlx::PgPool;
    fn get_redis_client(&self) -> &redis::Client;
    fn get_user(&self) -> Option<CacheUser>;
    fn can_access(&self, operate: PermissionPoints) -> Result<bool, ErrCode>;
}

#[async_trait]
pub trait AsyncContext: Context {
    async fn get_db_conn(&self) -> Result<PoolConnection<Postgres>, ErrCode>;
    async fn get_db_tx(&self) -> Result<Transaction<'static, Postgres>, ErrCode>;
}

#[derive(Clone, Debug)]
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

    fn get_user(&self) -> Option<CacheUser> {
        None
    }

    fn can_access(&self, _operate: PermissionPoints) -> Result<bool, ErrCode> {
        Err(ErrCode::UnPermission)
    }
}

#[async_trait]
impl AsyncContext for GlobalContext {
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

#[derive(Clone, Debug)]
pub struct RequestContext {
    global_ctx: Arc<GlobalContext>,
    user: Option<CacheUser>,
}

impl RequestContext {
    pub fn new(ctx: GlobalContext) -> Self {
        Self {
            global_ctx: Arc::new(ctx),
            user: None,
        }
    }

    pub fn set_user(&mut self, user: CacheUser) {
        self.user = Some(user);
    }
}

impl Context for RequestContext {
    fn get_db_pool(&self) -> &sqlx::PgPool {
        self.global_ctx.get_db_pool()
    }

    fn get_redis_client(&self) -> &redis::Client {
        self.global_ctx.get_redis_client()
    }

    fn get_user(&self) -> Option<CacheUser> {
        self.user.clone()
    }

    // 验证权限
    fn can_access(&self, operate: PermissionPoints) -> Result<bool, ErrCode> {
        let entry = self
            .global_ctx
            .perm
            .get(&operate)
            .ok_or(ErrCode::InternalError)?;

        let allowed = match &self.user {
            Some(user) => user.role_ids.iter().any(|rid| entry.role_ids.contains(rid)),
            None => entry.allow_anonymous,
        };

        allowed.then_some(true).ok_or(ErrCode::UnPermission)
    }
}

#[async_trait]
impl AsyncContext for RequestContext {
    async fn get_db_conn(&self) -> Result<PoolConnection<Postgres>, ErrCode> {
        self.global_ctx.get_db_conn().await
    }

    async fn get_db_tx(&self) -> Result<Transaction<'static, Postgres>, ErrCode> {
        self.global_ctx.get_db_tx().await
    }
}