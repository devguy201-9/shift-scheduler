use async_trait::async_trait;
use uuid::Uuid;

use crate::application::error::AppError;
use crate::domain::{group::StaffGroup, staff::Staff};

#[async_trait]
pub trait StaffRepository: Send + Sync {
    async fn create(&self, staff: Staff) -> Result<(), AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Staff>, AppError>;
    async fn update(&self, staff: Staff) -> Result<(), AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
    async fn create_batch(&self, staff: Vec<Staff>) -> Result<(), AppError>;
}

#[async_trait]
pub trait GroupRepository: Send + Sync {
    async fn create(&self, group: StaffGroup) -> Result<(), AppError>;
    async fn resolve_members(&self, group_id: Uuid) -> Result<Vec<Staff>, AppError>;
    async fn add_member(&self, group_id: Uuid, staff_id: Uuid) -> Result<(), AppError>;
    async fn remove_member(&self, group_id: Uuid, staff_id: Uuid) -> Result<(), AppError>;
    async fn update(&self, group: StaffGroup) -> Result<(), AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
    async fn create_batch(&self, groups: Vec<StaffGroup>) -> Result<(), AppError>;
}
