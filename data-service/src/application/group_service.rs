use crate::application::error::AppError;
use crate::application::traits::GroupRepository;
use crate::domain::group::StaffGroup;
use crate::domain::staff::Staff;
use crate::infrastructure::cache::RedisCache;
use std::sync::Arc;
use uuid::Uuid;

pub struct GroupService {
    repo: Arc<dyn GroupRepository + Send + Sync>,
    cache: Arc<RedisCache>,
}

impl GroupService {
    pub fn new(repo: Arc<dyn GroupRepository + Send + Sync>, cache: Arc<RedisCache>) -> Self {
        Self { repo, cache }
    }

    pub async fn create_group(&self, group: StaffGroup) -> Result<(), AppError> {
        // validate
        if group.name.trim().is_empty() {
            return Err(AppError::Validation("Group name cannot be empty".into()));
        }

        self.repo.create(group).await
    }

    pub async fn resolve_members(&self, group_id: Uuid) -> Result<Vec<Staff>, AppError> {
        let cache_key = format!("group:{}:resolved_members", group_id);

        // try cache first
        if let Ok(Some(cached)) = self.cache.get(&cache_key).await {
            if let Ok(members) = serde_json::from_str::<Vec<Staff>>(&cached) {
                return Ok(members);
            }
        }

        // Fetch data DB
        let members = self.repo.resolve_members(group_id).await?;

        // cache only if non-empty
        if !members.is_empty() {
            if let Ok(serialized) = serde_json::to_string(&members) {
                let _ = self.cache.set(&cache_key, &serialized, 60).await;
            }
        }

        Ok(members)
    }

    pub async fn add_member(&self, group_id: Uuid, staff_id: Uuid) -> Result<(), AppError> {
        // prevent duplicate membership
        if self.repo.is_member(group_id, staff_id).await? {
            return Err(AppError::Conflict("Staff already in group".into()));
        }

        self.repo.add_member(group_id, staff_id).await?;

        self.invalidate_cache(group_id).await;

        Ok(())
    }

    pub async fn remove_member(&self, group_id: Uuid, staff_id: Uuid) -> Result<(), AppError> {
        self.repo.remove_member(group_id, staff_id).await?;

        self.invalidate_cache(group_id).await;

        Ok(())
    }

    pub async fn update_group(&self, group: StaffGroup) -> Result<(), AppError> {
        let exists = self.repo.find_by_id(group.id.clone()).await?;
        if exists.is_none() {
            return Err(AppError::NotFound("Group not found".into()));
        }

        let group_id = group.id.clone();
        self.repo.update(group).await?;

        self.invalidate_cache(group_id).await;

        Ok(())
    }

    pub async fn delete_group(&self, group_id: Uuid) -> Result<(), AppError> {
        self.repo.delete(group_id).await?;

        self.invalidate_cache(group_id).await;

        Ok(())
    }

    pub async fn batch_create(&self, groups: Vec<StaffGroup>) -> Result<(), AppError> {
        if groups.is_empty() {
            return Err(AppError::Validation("Group list cannot be empty".into()));
        }

        self.repo.create_batch(groups).await
    }

    async fn invalidate_cache(&self, group_id: Uuid) {
        let key = format!("group:{}:resolved_members", group_id);
        let _ = self.cache.delete(&key).await;
    }
}
