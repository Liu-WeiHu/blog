use sqlx::PgConnection;

use crate::{
    dto::permission::{PermissionType, RbacRolePermission},
    model::rbac::UserRole,
};
use async_trait::async_trait;

#[allow(dead_code)]
#[async_trait]
pub trait RbacDao: Send + Sync {
   async fn select_role_permission(
        &self,
        executor: &mut PgConnection,
    ) -> Result<Vec<RbacRolePermission>, sqlx::Error>;

    async fn select_permissions_by_user_id(
        &self,
        executor: &mut PgConnection,
        user_id: i32,
    ) -> Result<Vec<String>, sqlx::Error>;

    async fn select_user_role_by_user_id(
        &self,
        executor: &mut PgConnection,
        user_id: i32,
    ) -> Result<Vec<UserRole>, sqlx::Error>;
}

struct RbacDaoI;

pub fn new_rbac_dao() -> impl RbacDao {
    RbacDaoI {}
}

#[async_trait]
impl RbacDao for RbacDaoI {
    async fn select_role_permission(
        &self,
        executor: &mut PgConnection,
    ) -> Result<Vec<RbacRolePermission>, sqlx::Error> {
        sqlx::query_as!(
            RbacRolePermission,
            "select r.id,  r.name as role_name , p.name as permission_name  from roles r 
join role_permissions rp  on r.id  = rp.role_id
join permissions p on rp.permission_id  = p.id"
        )
        .fetch_all(executor)
        .await
    }

    async fn select_permissions_by_user_id(
        &self,
        executor: &mut PgConnection,
        user_id: i32,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query!(
            "select p.name as permission_name from user_roles ur 
join role_permissions rp on ur.role_id = rp.role_id
join permissions p on rp.permission_id = p.id
where ur.user_id = $1 and p.permission_type = $2",
            user_id,
            PermissionType::Visual.as_ref()
        )
        .fetch_all(executor)
        .await
        .or_else(|err| match err {
            sqlx::Error::RowNotFound => Ok(Vec::new()),
            other => Err(other),
        })?;

        Ok(rows.into_iter().map(|row| row.permission_name).collect())
    }

    async fn select_user_role_by_user_id(
        &self,
        executor: &mut PgConnection,
        user_id: i32,
    ) -> Result<Vec<UserRole>, sqlx::Error> {
        sqlx::query_as!(
            UserRole,
            "select * from user_roles where user_id = $1",
            user_id,
        )
        .fetch_all(executor)
        .await
        .or_else(|err| match err {
            sqlx::Error::RowNotFound => Ok(Vec::new()),
            other => Err(other),
        })
    }
}
