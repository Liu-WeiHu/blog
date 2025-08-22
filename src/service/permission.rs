use std::{collections::HashMap, sync::Arc};

use crate::{
    context::Context,
    dao::permission::{RbacDao, new_rbac_dao},
    rbac::{PermissionPoints, PermissionRegistry},
    response::ErrCode,
    service::handle_error,
};

pub trait RbacService: Send + Sync + Clone {
    fn get_rbac_permission(
        &self,
    ) -> impl Future<Output = Result<PermissionRegistry, ErrCode>> + Send;
    fn get_user_permission(&self) -> impl Future<Output = Result<Vec<String>, ErrCode>> + Send;
}

#[derive(Clone)]
struct RbacServiceI<Ctx: Context> {
    ctx: Ctx,
}

pub fn new_rbac_service<Ctx: Context>(ctx: Ctx) -> impl RbacService {
    RbacServiceI { ctx }
}

impl<Ctx: Context> RbacService for RbacServiceI<Ctx> {
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
                |mut acc: HashMap<PermissionPoints, HashMap<i32, String>>, item| {
                    acc.entry(item.permission_name)
                        .or_default()
                        .insert(item.id, item.role_name.into_owned());
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
