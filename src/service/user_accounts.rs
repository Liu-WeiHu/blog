use std::borrow::Cow;

use crate::{
    context::Context,
    dao::{
        user_accounts::{UserAccountsDao, new_user_accounts_dao},
        user_ext::{UserExtDao, new_user_ext_dao},
    },
    dto::user_accounts::{RegisterReq, UpdateUserInfoReq, UserInfo},
    init::KEYS,
    jwt::{AuthBody, Claims},
    model::{user_accounts::UserAccounts, user_ext::UserExt},
    pagination::Pagination,
    response::ErrCode,
    service::{get_conn, get_tx},
};

use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{DateTime, Utc};
use redis::Commands;

pub trait UserAccountsService: Send + Sync + Clone {
    fn list(
        &self,
        pag: Pagination,
    ) -> impl std::future::Future<Output = Result<Vec<UserAccounts>, ErrCode>> + std::marker::Send;
    fn one(
        &self,
        user_id: i32,
    ) -> impl std::future::Future<Output = Result<UserInfo, ErrCode>> + std::marker::Send;
    fn register(
        &self,
        req: RegisterReq,
    ) -> impl std::future::Future<Output = Result<UserAccounts, ErrCode>> + std::marker::Send;
    fn login(
        &self,
        email: Cow<'static, str>,
        password: Cow<'static, str>,
    ) -> impl std::future::Future<Output = Result<AuthBody, ErrCode>> + std::marker::Send;
    fn edit_info(
        &self,
        req: UpdateUserInfoReq,
    ) -> impl std::future::Future<Output = Result<UserAccounts, ErrCode>> + std::marker::Send;
}

#[derive(Clone)]
struct UserAccountsServiceI {
    ctx: Context,
}

pub fn new_user_accounts_service(ctx: Context) -> impl UserAccountsService {
    UserAccountsServiceI { ctx }
}

impl UserAccountsService for UserAccountsServiceI {
    async fn list(&self, pag: Pagination) -> Result<Vec<UserAccounts>, ErrCode> {
        tracing::debug!(
            "UserAccountsService.list offset = {}, limit = {}",
            pag.offset,
            pag.limit
        );
        let mut conn = get_conn(self.ctx.get_pool().clone()).await.unwrap();
        new_user_accounts_dao()
            .select_all(&mut conn, pag.offset, pag.limit)
            .await
            .map_err(|err| {
                tracing::error!("db err = {}", err);
                ErrCode::InternalError
            })
    }

    async fn one(&self, user_id: i32) -> Result<UserInfo, ErrCode> {
        tracing::debug!("UserAccountsService.one user_id = {}", user_id);
        let mut conn = get_conn(self.ctx.get_pool().clone()).await.unwrap();
        new_user_accounts_dao()
            .select_by_id(&mut conn, user_id)
            .await
            .map_err(|err| {
                tracing::error!("db err = {}", err);
                ErrCode::InternalError
            })
    }

    async fn register(&self, req: RegisterReq) -> Result<UserAccounts, ErrCode> {
        tracing::debug!("UserAccountsService.register req = {:?}", req);
        if !(3..50).contains(&req.username.len()) {
            return Err(ErrCode::InputNameInvalid);
        }
        if !(3..50).contains(&req.email.len()) {
            return Err(ErrCode::InputEmailInvalid);
        }
        if !(3..50).contains(&req.password.len()) {
            return Err(ErrCode::InputPasswordInvalid);
        }
        let hashed = hash(req.password.as_ref(), DEFAULT_COST).map_err(|err| {
            tracing::error!("password = {} 加密失败err = {}", &req.password, err);
            ErrCode::InternalError
        })?;
        let user = UserAccounts {
            username: req.username,
            email: req.email,
            password: Cow::Owned(hashed),
            ..Default::default()
        };

        // 开启事物, 由于 tx 实现了 Drop trait
        // drop 方法里自动rollback了.所以只需要显示commit就可以了.
        let mut tx = get_tx(self.ctx.get_pool().clone()).await.unwrap();
        let res_user = new_user_accounts_dao()
            .insert(&mut tx, user)
            .await
            .map_err(|err| {
                tracing::error!("user_accounts insert err = {}", err);
                match err {
                    sqlx::Error::Database(db_err)
                        if db_err.code().as_deref() == Some("23505")
                            && db_err.constraint() == Some("users_email_key") =>
                    {
                        ErrCode::EmailAlreadyRegistered
                    }
                    _ => ErrCode::InternalError,
                }
            })?;

        // 使用宏映射
        let user_ext = map_to_user_ext!(req, res_user.id);
        new_user_ext_dao()
            .insert(&mut tx, user_ext)
            .await
            .map_err(|e| {
                tracing::error!("user_ext insert err = {}", e);
                ErrCode::InternalError
            })?;
        tx.commit().await.map_err(|e| {
            tracing::error!("tx.commit err = {}", e);
            ErrCode::InternalError
        })?;
        Ok(res_user)
    }

    async fn login(
        &self,
        email: Cow<'static, str>,
        password: Cow<'static, str>,
    ) -> Result<AuthBody, ErrCode> {
        if email.is_empty() || password.is_empty() {
            return Err(ErrCode::InputArgsError);
        }

        let dao = new_user_accounts_dao();
        let mut conn = get_conn(self.ctx.get_pool().clone()).await.unwrap();
        let user = dao.select_by_email(&mut conn, email).await.map_err(|err| {
            tracing::error!("db err = {}", err);
            ErrCode::InternalError
        })?;
        if !verify(password.as_ref(), &user.password).map_err(|err| {
            tracing::error!("password 解密失败err = {}", err);
            ErrCode::InternalError
        })? {
            return Err(ErrCode::InputLoginInvalid);
        };

        let now = Utc::now().timestamp();

        let claims = Claims {
            sub: user.id.to_string(),
            exp: now + 7 * 24 * 60 * 60,
            iat: now,
        };
        // Create the authorization token
        let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &KEYS.encoding)
            .map_err(|err| {
                tracing::error!("jwt encode err = {}", err);
                ErrCode::InternalError
            })?;

        let mut user = user;

        user.last_login_time = Some(DateTime::from_timestamp(now, 0).unwrap().naive_utc());
        dao.update_login_time_by_id(&mut conn, user.clone())
            .await
            .map_err(|e| {
                tracing::error!("修改登陆时间出错error = {}", e);
                ErrCode::InternalError
            })?;

        let redis_key = format!("user:{}", user.id);
        let user_json = serde_json::to_string(&user).map_err(|e| {
            tracing::error!("user to json err = {}, user.id = {}", e, &user.id);
            ErrCode::InternalError
        })?;

        let _: () = self
            .ctx
            .get_redis()
            .clone()
            .set_ex(&redis_key, &user_json, 7 * 24 * 60 * 60)
            .map_err(|e| {
                tracing::error!(
                    "redis set_ex err = {}, redis_key = {}, value = {}",
                    e,
                    redis_key,
                    user_json
                );
                ErrCode::InternalError
            })?;

        Ok(AuthBody::new(token))
    }

    async fn edit_info(&self, req: UpdateUserInfoReq) -> Result<UserAccounts, ErrCode> {
        tracing::debug!("UserAccountsService.exit_info req = {:?}", req);
        if req.user_id <= 0 {
            return Err(ErrCode::InputArgsError);
        }
        if !(3..50).contains(&req.username.len()) {
            return Err(ErrCode::InputNameInvalid);
        }
        let mut tx = get_tx(self.ctx.get_pool().clone()).await.unwrap();
        let user = UserAccounts {
            id: req.user_id,
            username: req.username,
            ..Default::default()
        };
        new_user_accounts_dao()
            .update(&mut tx, user.clone())
            .await
            .map_err(|e| {
                tracing::error!("修改用户名出错 error = {}", e);
                ErrCode::InternalError
            })?;

        // 使用宏映射
        let user_ext = map_to_user_ext!(req);
        tracing::debug!("user_ext: {:?}", user_ext);
        new_user_ext_dao()
            .update_by_user_id(&mut tx, user_ext)
            .await
            .map_err(|e| {
                tracing::error!("修改用户扩展信息出错 error = {}", e);
                ErrCode::InternalError
            })?;
        tx.commit().await.map_err(|e| {
            tracing::error!("tx.commit err = {}", e);
            ErrCode::InternalError
        })?;
        Ok(user)
    }
}
