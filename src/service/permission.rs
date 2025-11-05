use std::{collections::HashMap, sync::Arc};

use crate::{
    context::AsyncContext,
    dao::permission::{RbacDao, new_rbac_dao},
    rbac::{PermissionEntry, PermissionPoints, PermissionRegistry},
    response::ErrCode,
    service::handle_error,
};
use async_trait::async_trait;

const ANONYMOUS: &str = "Anonymous User";

#[async_trait]
pub trait RbacService: Send + Sync + Clone {
    async fn get_rbac_permission(&self) -> Result<PermissionRegistry, ErrCode>;
    async fn get_user_permission(&self) -> Result<Vec<String>, ErrCode>;
}

#[derive(Clone)]
struct RbacServiceI<Ctx: AsyncContext> {
    ctx: Ctx,
}

pub fn new_rbac_service<Ctx: AsyncContext>(ctx: Ctx) -> impl RbacService {
    RbacServiceI { ctx }
}

#[async_trait]
impl<Ctx: AsyncContext> RbacService for RbacServiceI<Ctx> {

    async fn get_rbac_permission(&self) -> Result<PermissionRegistry, ErrCode> {
        tracing::debug!("get_rbac_permission start handle");

        let mut conn = self.ctx.get_db_conn().await?;
        let map = new_rbac_dao()
            .select_role_permission(&mut conn)
            .await
            .map_err(|err| handle_error(Box::new(err), "dao new_rbac_dao"))?
            .into_iter()
            .fold(
                HashMap::new(),
                |mut acc: HashMap<PermissionPoints, PermissionEntry>, item| {
                    acc.entry(item.permission_name)
                        .and_modify(|e| {
                            e.role_ids.insert(item.id);
                            e.allow_anonymous =
                                e.allow_anonymous || item.role_name == ANONYMOUS;
                        })
                        .or_insert_with(|| PermissionEntry {
                            role_ids: vec![item.id].into_iter().collect(),
                            allow_anonymous: item.role_name == ANONYMOUS,
                        });
                    acc
                },
            );
        Ok(Arc::new(map))
    }

    async fn get_user_permission(&self) -> Result<Vec<String>, ErrCode> {
        tracing::debug!("get_user_permission start handle ");

        let user = match self.ctx.get_user() {
            Some(user) => user,
            None => return Ok(Vec::new()),
        };

        let mut conn = self.ctx.get_db_conn().await?;
        new_rbac_dao()
            .select_permissions_by_user_id(&mut conn, user.id)
            .await
            .map_err(|err| handle_error(Box::new(err), "dao select_permissions_by_user_id"))
    }
}
