use std::sync::Arc;
use uuid::Uuid;

use crate::application::{error::AppError, traits::StaffRepository};
use crate::domain::staff::Staff;

pub struct StaffService {
    repo: Arc<dyn StaffRepository + Send + Sync>,
}

impl StaffService {
    pub fn new(repo: Arc<dyn StaffRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    pub async fn create_staff(&self, staff: Staff) -> Result<(), AppError> {
        self.repo.create(staff).await
    }

    pub async fn get_staff(&self, id: Uuid) -> Result<Option<Staff>, AppError> {
        self.repo.find_by_id(id).await
    }
    pub async fn update_staff(&self, staff: Staff) -> Result<(), AppError> {
        self.repo.update(staff).await
    }

    pub async fn delete_staff(&self, id: Uuid) -> Result<(), AppError> {
        self.repo.delete(id).await
    }
    pub async fn batch_create(&self, list: Vec<Staff>) -> Result<(), AppError> {
        self.repo.create_batch(list).await
    }
}
