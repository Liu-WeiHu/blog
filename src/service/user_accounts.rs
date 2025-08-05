use crate::{
    dao::user_accounts::{UserAccountsDao, new_user_accounts_dao},
    init::KEYS,
    jwt::{AuthBody, Claims},
    model::user_accounts::UserAccounts,
    pagination::Pagination,
    response::ErrCode,
};

use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub trait UserAccountsService: Send + Sync + Clone {
    fn list(
        &self,
        pag: Pagination,
    ) -> impl std::future::Future<Output = Result<Vec<UserAccounts>, ErrCode>> + std::marker::Send;
    fn one(
        &self,
        user_id: i32,
    ) -> impl std::future::Future<Output = Result<UserAccounts, ErrCode>> + std::marker::Send;
    fn register(
        &self,
        user: UserAccounts,
    ) -> impl std::future::Future<Output = Result<UserAccounts, ErrCode>> + std::marker::Send;
    fn login(
        &self,
        email: String,
        password: String,
    ) -> impl std::future::Future<Output = Result<AuthBody, ErrCode>> + std::marker::Send;
}

#[derive(Clone)]
struct UserAccountsServiceI<DAO: UserAccountsDao> {
    dao: DAO,
}

pub fn new_user_accounts_service(pool: PgPool) -> impl UserAccountsService {
    let dao = new_user_accounts_dao(pool);
    UserAccountsServiceI { dao }
}

impl<DAO: UserAccountsDao> UserAccountsService for UserAccountsServiceI<DAO> {
    async fn list(&self, pag: Pagination) -> Result<Vec<UserAccounts>, ErrCode> {
        tracing::debug!(
            "UserAccountsService.list offset = {}, limit = {}",
            pag.offset,
            pag.limit
        );
        self.dao
            .select_all(pag.offset, pag.limit)
            .await
            .map_err(|err| {
                tracing::error!("db err = {}", err);
                ErrCode::InternalError
            })
    }

    async fn one(&self, user_id: i32) -> Result<UserAccounts, ErrCode> {
        tracing::debug!("UserAccountsService.one user_id = {}", user_id);
        self.dao.select_by_id(user_id).await.map_err(|err| {
            tracing::error!("db err = {}", err);
            ErrCode::InternalError
        })
    }

    async fn register(&self, mut user: UserAccounts) -> Result<UserAccounts, ErrCode> {
        tracing::debug!("UserAccountsService.register user = {:?}", user);
        if !(3..50).contains(&user.username.len()) {
            return Err(ErrCode::InputNameInvalid);
        }
        if !(3..50).contains(&user.email.len()) {
            return Err(ErrCode::InputEmailInvalid);
        }
        if !(3..50).contains(&user.password.len()) {
            return Err(ErrCode::InputPasswordInvalid);
        }
        let hashed = hash(&user.password, DEFAULT_COST).map_err(|err| {
            tracing::error!("password = {} 加密失败err = {}", &user.password, err);
            ErrCode::InternalError
        })?;
        user.password = hashed;
        self.dao.insert(user).await.map_err(|err| {
            tracing::error!("db err = {}", err);
            ErrCode::InternalError
        })
    }

    async fn login(&self, email: String, password: String) -> Result<AuthBody, ErrCode> {
        if email.is_empty() || password.is_empty() {
            return Err(ErrCode::InputArgsError);
        }

        let user = self.dao.select_by_email(email).await.map_err(|err| {
            tracing::error!("db err = {}", err);
            ErrCode::InternalError
        })?;
        if !verify(password, &user.password).map_err(|err| {
            tracing::error!("password 解密失败err = {}", err);
            ErrCode::InternalError
        })? {
            return Err(ErrCode::InputPasswordInvalid);
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
        self.dao.update_login_time_by_id(user).await.map_err(|e| {
            tracing::error!("修改登陆时间出错error = {}", e);
            ErrCode::InternalError
        })?;

        Ok(AuthBody::new(token))
    }
}
