use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use axum::{extract::FromRequestParts, http::request::Parts};
use dashmap::DashMap;

use crate::response::{self, ErrCode, make_response};

#[derive(Clone)]
pub struct TypedStorage {
    inner: Arc<DashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

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

    #[allow(dead_code)]
    pub fn with_mut<T: 'static + Send + Sync, F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.inner
            .get_mut(&TypeId::of::<T>())
            .and_then(|mut entry| entry.downcast_mut::<T>().map(f))
    }

    #[allow(dead_code)]
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

impl Context {
    pub fn new(pool: sqlx::PgPool, redis: redis::Client) -> Self {
        Self {
            pool,
            redis,
            storage: TypedStorage::new(),
        }
    }

    pub fn get_pool(&self) -> &sqlx::PgPool {
        &self.pool
    }

    pub fn get_redis(&self) -> &redis::Client {
        &self.redis
    }

    // 代理到 storage 的方法
    pub fn insert<T: 'static + Send + Sync + Clone>(&self, value: T) {
        self.storage.insert(value);
    }

    pub fn get<T: 'static + Send + Sync + Clone>(&self) -> Option<T> {
        self.storage.get()
    }

    #[allow(dead_code)]
    pub fn with_mut<T: 'static + Send + Sync, F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.storage.with_mut(f)
    }
}

impl<S> FromRequestParts<S> for Context
where
    S: Send + Sync + Any,
{
    type Rejection = response::Resp<()>;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(ctx) = (state as &dyn Any).downcast_ref::<Context>() {
            return Ok(ctx.clone());
        }

        Err(make_response(Err(ErrCode::InternalError)))
    }
}
