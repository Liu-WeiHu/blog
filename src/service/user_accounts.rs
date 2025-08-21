use std::borrow::Cow;

use crate::{
    context::Context,
    dao::{
        user_accounts::{UserAccountsDao, new_user_accounts_dao},
        user_ext::{UserExtDao, new_user_ext_dao},
    },
    dto::user_accounts::{RegisterReq, UserInfo},
    init::KEYS,
    jwt::{AuthBody, Claims},
    model::{user_accounts::UserAccounts, user_ext::UserExt},
    pagination::Pagination,
    response::ErrCode,
    service::handle_error,
};

use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{DateTime, Utc};
use redis::Commands;

const TOKEN_EXPIRE_SECONDS: i64 = 7 * 24 * 60 * 60; // 7天
const REDIS_EXPIRE_SECONDS: u64 = 7 * 24 * 60 * 60; // 7天

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
        req: RegisterReq,
        user_id: i32,
    ) -> impl std::future::Future<Output = Result<UserAccounts, ErrCode>> + std::marker::Send;
}

#[derive(Clone)]
struct UserAccountsServiceI {
    ctx: Context,
}

pub fn new_user_accounts_service(ctx: Context) -> impl UserAccountsService {
    UserAccountsServiceI { ctx }
}

impl UserAccountsServiceI {
    /// 生成JWT令牌
    fn generate_token(&self, user_id: i32, now: i64) -> Result<String, ErrCode> {
        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + TOKEN_EXPIRE_SECONDS,
            iat: now,
        };

        jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &KEYS.encoding)
            .map_err(|err| handle_error(Box::new(err), "generate_token"))
    }

    /// 缓存用户信息到Redis
    async fn cache_user_info(&self, user: &UserAccounts) -> Result<(), ErrCode> {
        let redis_key = format!("user:{}", user.id);
        let user_json = serde_json::to_string(user)
            .map_err(|err| handle_error(Box::new(err), "cache_user_info"))?;

        let _: () = self
            .ctx
            .get_redis_client()
            .clone()
            .set_ex(&redis_key, &user_json, REDIS_EXPIRE_SECONDS)
            .map_err(|err| handle_error(Box::new(err), "redis set_ex"))?;

        Ok(())
    }
}

impl UserAccountsService for UserAccountsServiceI {
    #[tracing::instrument(skip(self), fields(offset = pag.offset, limit = pag.limit))]
    async fn list(&self, pag: Pagination) -> Result<Vec<UserAccounts>, ErrCode> {
        tracing::debug!("Listing users with pagination");

        let mut conn = self.ctx.get_db_conn().await?;
        new_user_accounts_dao()
            .select_all(&mut conn, pag.offset, pag.limit)
            .await
            .map_err(|err| handle_error(Box::new(err), "dao select_all"))
    }

    #[tracing::instrument(skip(self), fields(user_id = %user_id))]
    async fn one(&self, user_id: i32) -> Result<UserInfo, ErrCode> {
        tracing::debug!("Getting user");

        let mut conn = self.ctx.get_db_conn().await?;
        new_user_accounts_dao()
            .select_by_id(&mut conn, user_id)
            .await
            .map_err(|err| handle_error(Box::new(err), "dao get_user_by_id"))
    }

    #[tracing::instrument(skip(self, req), fields(username = %req.username, email = %req.email))]
    async fn register(&self, req: RegisterReq) -> Result<UserAccounts, ErrCode> {
        tracing::debug!("Registering new user");

        if !(3..50).contains(&req.username.len()) {
            return Err(ErrCode::InputNameInvalid);
        }
        if !(3..50).contains(&req.email.len()) {
            return Err(ErrCode::InputEmailInvalid);
        }
        if !(3..50).contains(&req.password.len()) {
            return Err(ErrCode::InputPasswordInvalid);
        }
        let hashed = hash(req.password.as_ref(), DEFAULT_COST)
            .map_err(|err| handle_error(Box::new(err), "hash password"))?;
        let user = UserAccounts {
            username: req.username,
            email: req.email,
            password: Cow::Owned(hashed),
            ..Default::default()
        };

        // 开启事物, 由于 tx 实现了 Drop trait
        // drop 方法里自动rollback了.所以只需要显示commit就可以了.
        let mut tx = self.ctx.get_db_tx().await?;
        let res_user = new_user_accounts_dao()
            .insert(&mut tx, user)
            .await
            .map_err(|err| handle_error(Box::new(err), "dao insert user"))?;

        // 使用宏映射
        let user_ext = map_to_user_ext!(req, res_user.id);
        new_user_ext_dao()
            .insert(&mut tx, user_ext)
            .await
            .map_err(|err| handle_error(Box::new(err), "dao insert user_ext"))?;
        tx.commit()
            .await
            .map_err(|err| handle_error(Box::new(err), "tx commit"))?;
        Ok(res_user)
    }

    #[tracing::instrument(skip(self, password), fields(email = %email))]
    async fn login(
        &self,
        email: Cow<'static, str>,
        password: Cow<'static, str>,
    ) -> Result<AuthBody, ErrCode> {
        tracing::debug!("User login attempt");

        if email.is_empty() || password.is_empty() {
            return Err(ErrCode::InputArgsError);
        }

        let dao = new_user_accounts_dao();
        let mut conn = self.ctx.get_db_conn().await?;
        let user = dao
            .select_by_email(&mut conn, email)
            .await
            .map_err(|err| handle_error(Box::new(err), "dao select_by_email"))?;
        if !verify(password.as_ref(), &user.password).map_err(|_err| ErrCode::InputLoginInvalid)? {
            return Err(ErrCode::InputLoginInvalid);
        };

        let now = Utc::now().timestamp();
        // 生成JWT令牌
        let token = self.generate_token(user.id, now)?;

        let mut user = user;
        user.last_login_time = Some(DateTime::from_timestamp(now, 0).unwrap().naive_utc());
        dao.update_login_time_by_id(&mut conn, user.clone())
            .await
            .map_err(|err| handle_error(Box::new(err), "dao update_login_time_by_id"))?;

        // 缓存token
        self.cache_user_info(&user).await?;
        Ok(AuthBody::new(token))
    }

    #[tracing::instrument(skip(self), fields(user_id = user_id, username = %req.username))]
    async fn edit_info(&self, req: RegisterReq, user_id: i32) -> Result<UserAccounts, ErrCode> {
        tracing::debug!("Editing user info");

        if user_id <= 0 {
            return Err(ErrCode::InputArgsError);
        }
        if !(3..50).contains(&req.username.len()) {
            return Err(ErrCode::InputNameInvalid);
        }
        let mut tx = self.ctx.get_db_tx().await?;
        let user = UserAccounts {
            id: user_id,
            username: req.username,
            ..Default::default()
        };
        new_user_accounts_dao()
            .update(&mut tx, user.clone())
            .await
            .map_err(|err| handle_error(Box::new(err), "dao update"))?;

        // 使用宏映射
        let user_ext = map_to_user_ext!(req, user_id);
        tracing::debug!("user_ext: {:?}", user_ext);
        new_user_ext_dao()
            .update_by_user_id(&mut tx, user_ext)
            .await
            .map_err(|err| handle_error(Box::new(err), "dao update_by_user_id"))?;
        tx.commit()
            .await
            .map_err(|err| handle_error(Box::new(err), "tx commit"))?;
        Ok(user)
    }
}
