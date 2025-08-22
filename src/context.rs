use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use axum::{extract::FromRequestParts, http::request::Parts};
use dashmap::DashMap;
use sqlx::{Postgres, Transaction, pool::PoolConnection};

use crate::{
    dto::user_accounts::CacheUser,
    rbac::{ANONYMOUS, PermissionPoints, PermissionRegistry},
    response::{self, ErrCode, make_response},
};

#[derive(Clone)]
pub struct TypedStorage {
    inner: Arc<DashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

#[allow(dead_code)]
impl TypedStorage {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }

    pub fn insert<T: 'static + Send + Sync + Clone>(&self, value: T) {
        self.inner.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: 'static + Send + Sync + Clone>(&self) -> Option<T> {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|entry| entry.downcast_ref::<T>().cloned())
    }

    pub fn with_mut<T: 'static + Send + Sync, F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.inner
            .get_mut(&TypeId::of::<T>())
            .and_then(|mut entry| entry.downcast_mut::<T>().map(f))
    }

    pub fn remove<T: 'static + Send + Sync>(&self) -> Option<T> {
        self.inner
            .remove(&TypeId::of::<T>())
            .map(|(_, boxed)| boxed)
            .and_then(|boxed| boxed.downcast().ok())
            .map(|boxed| *boxed)
    }
}

#[derive(Clone)]
pub struct Context {
    pool: sqlx::PgPool,
    redis: redis::Client,
    storage: TypedStorage,
}

#[allow(dead_code)]
impl Context {
    pub fn new(pool: sqlx::PgPool, redis: redis::Client) -> Self {
        Self {
            pool,
            redis,
            storage: TypedStorage::new(),
        }
    }

    pub fn get_db_pool(&self) -> &sqlx::PgPool {
        &self.pool
    }

    pub fn get_redis_client(&self) -> &redis::Client {
        &self.redis
    }

    pub async fn get_db_conn(&self) -> Result<PoolConnection<Postgres>, ErrCode> {
        self.pool.acquire().await.map_err(|e| {
            tracing::error!("Failed to get database connection: {}", e);
            ErrCode::InternalError
        })
    }

    pub async fn get_db_tx(&self) -> Result<Transaction<'static, Postgres>, ErrCode> {
        self.pool.begin().await.map_err(|e| {
            tracing::error!("Failed to get database transaction: {}", e);
            ErrCode::InternalError
        })
    }

    // 代理到 storage 的方法
    pub fn insert<T: 'static + Send + Sync + Clone>(&self, value: T) {
        self.storage.insert(value);
    }

    pub fn get<T: 'static + Send + Sync + Clone>(&self) -> Option<T> {
        self.storage.get()
    }

    pub fn with_mut<T: 'static + Send + Sync, F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.storage.with_mut(f)
    }

    // 验证权限
    pub fn can_access(&self, operate: PermissionPoints) -> Result<bool, ErrCode> {
        let pr = self
            .get::<PermissionRegistry>()
            .ok_or(ErrCode::InternalError)?;
        let map = pr.get(&operate).ok_or(ErrCode::InternalError)?;

        let can_access = match self.get::<CacheUser>() {
            Some(user) => user
                .role_ids
                .iter()
                .any(|role_id| map.contains_key(role_id)),
            None => map.values().any(|x| *x == ANONYMOUS),
        };

        can_access.then_some(true).ok_or(ErrCode::UnPermission)
    }
}

impl<S> FromRequestParts<S> for Context
where
    S: Send + Sync + Any,
{
    type Rejection = response::Resp<()>;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        (state as &dyn Any)
            .downcast_ref::<Context>()
            .cloned()
            .ok_or(make_response(Err(ErrCode::InternalError)))
    }
}
