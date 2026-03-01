use crate::application::group_service::GroupService;
use crate::application::staff_service::StaffService;
use crate::application::traits::{GroupRepository, StaffRepository};
use crate::infrastructure::cache::RedisCache;
use crate::infrastructure::group_repository::GroupRepositoryPg;
use crate::infrastructure::staff_repository::StaffRepositoryPg;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub staff_service: Arc<StaffService>,
    pub group_service: Arc<GroupService>,
}

impl AppState {
    pub fn new(pool: PgPool, redis: RedisCache) -> Self {
        let staff_repo: Arc<dyn StaffRepository + Send + Sync> =
            Arc::new(StaffRepositoryPg::new(pool.clone()));

        let group_repo: Arc<dyn GroupRepository + Send + Sync> =
            Arc::new(GroupRepositoryPg::new(pool.clone()));

        let staff_service = Arc::new(StaffService::new(staff_repo));
        let group_service = Arc::new(GroupService::new(group_repo, Arc::new(redis)));

        Self {
            staff_service,
            group_service,
        }
    }
}
