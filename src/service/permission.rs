use std::{collections::HashMap, sync::Arc};

use crate::{
    context::Context,
    dao::permission::{RbacDao, new_rbac_dao},
    rbac::PermissionRegistry,
    response::ErrCode,
    service::handle_error,
};

pub trait RbacService: Send + Sync + Clone {
    fn get_rbac_permission(
        &self,
    ) -> impl std::future::Future<Output = Result<PermissionRegistry, ErrCode>> + std::marker::Send;

    fn get_user_permission(
        &self,
        user_id: i32,
    ) -> impl std::future::Future<Output = Result<Vec<String>, ErrCode>> + std::marker::Send;
}

#[derive(Clone)]
struct RbacServiceI {
    ctx: Context,
}

pub fn new_rbac_service(ctx: Context) -> impl RbacService {
    RbacServiceI { ctx }
}

impl RbacService for RbacServiceI {
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
                |mut acc: HashMap<String, HashMap<i32, String>>, item| {
                    acc.entry(item.permission_name.into_owned())
                        .or_default()
                        .insert(item.id, item.role_name.into_owned());
                    acc
                },
            );
        Ok(Arc::new(map))
    }

    #[tracing::instrument(skip(self), fields(user_id = %user_id))]
    async fn get_user_permission(&self, user_id: i32) -> Result<Vec<String>, ErrCode> {
        tracing::debug!("get_user_permission start handle");

        let mut conn = self.ctx.get_db_conn().await?;
        new_rbac_dao()
            .select_permissions_by_user_id(&mut conn, user_id)
            .await
            .map_err(|err| handle_error(Box::new(err), "dao select_permissions_by_user_id"))
    }
}
