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
        self.repo.create(group).await
    }

    pub async fn resolve_members(&self, group_id: Uuid) -> Result<Vec<Staff>, AppError> {
        let cache_key = format!("group:{}:resolved_members", group_id);

        // Get cache
        if let Some(cached) = self.cache.get(&cache_key).await? {
            let members: Vec<Staff> = serde_json::from_str(&cached).map_err(|_| AppError::Cache)?;
            return Ok(members);
        }

        // Fetch data DB
        let members = self.repo.resolve_members(group_id).await?;

        // Cache data
        let serialized = serde_json::to_string(&members).map_err(|_| AppError::Cache)?;

        self.cache.set(&cache_key, &serialized, 60).await?;

        Ok(members)
    }

    pub async fn add_member(&self, group_id: Uuid, staff_id: Uuid) -> Result<(), AppError> {
        self.repo.add_member(group_id, staff_id).await?;

        let key = format!("group:{}:resolved_members", group_id);
        self.cache.delete(&key).await?;

        Ok(())
    }

    pub async fn remove_member(&self, group_id: Uuid, staff_id: Uuid) -> Result<(), AppError> {
        self.repo.remove_member(group_id, staff_id).await?;

        let key = format!("group:{}:resolved_members", group_id);
        self.cache.delete(&key).await?;

        Ok(())
    }

    pub async fn update_group(&self, group: StaffGroup) -> Result<(), AppError> {
        let key = format!("group:{}:resolved_members", group.id);
        self.repo.update(group).await?;

        self.cache.delete(&key).await?;

        Ok(())
    }

    pub async fn delete_group(&self, group_id: Uuid) -> Result<(), AppError> {
        let key = format!("group:{}:resolved_members", group_id);
        self.repo.delete(group_id).await?;
        self.cache.delete(&key).await?;

        Ok(())
    }

    pub async fn batch_create(&self, groups: Vec<StaffGroup>) -> Result<(), AppError> {
        self.repo.create_batch(groups).await
    }
}
