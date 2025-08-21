use tracing::error;

use crate::response::ErrCode;

macro_rules! map_to_user_ext {
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

/// 统一的错误处理
fn handle_error(err: Box<dyn std::error::Error>, operation: &str) -> ErrCode {
    error!("Operation '{}' failed: {}", operation, err);

    if let Some(sqlx_err) = err.downcast_ref::<sqlx::Error>() {
        return handle_sqlx_error(sqlx_err);
    }

    if let Some(redis_err) = err.downcast_ref::<redis::RedisError>() {
        return handle_redis_error(redis_err);
    }

    // 默认返回内部错误
    ErrCode::InternalError
}

/// 处理 SQLx 错误
fn handle_sqlx_error(err: &sqlx::Error) -> ErrCode {
    match err {
        sqlx::Error::Database(db_err) => {
            // 处理特定的数据库约束错误
            if db_err.code().as_deref() == Some("23505")
                && db_err.constraint() == Some("users_email_key")
            {
                return ErrCode::EmailAlreadyRegistered;
            }
            ErrCode::InternalError
        }
        sqlx::Error::PoolClosed | sqlx::Error::PoolTimedOut => ErrCode::DbServiceUnavailable,
        _ => ErrCode::InternalError,
    }
}

/// 处理 Redis 错误
fn handle_redis_error(err: &redis::RedisError) -> ErrCode {
    match err.kind() {
        redis::ErrorKind::AuthenticationFailed => ErrCode::RedisServiceUnavailable,
        redis::ErrorKind::IoError => ErrCode::RedisServiceUnavailable,
        _ => ErrCode::InternalError,
    }
}
