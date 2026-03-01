use std::sync::Arc;
use uuid::Uuid;
use shared::types::StaffStatus;
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
        // validate data before create
        if staff.name.trim().is_empty() {
            return Err(AppError::Validation("Staff name cannot be empty".into()));
        }

        if !staff.email.contains('@') {
            return Err(AppError::Validation("Invalid email format".into()));
        }

        // check if email already exists
        if self.repo.exists_by_email(&staff.email).await? {
            return Err(AppError::Conflict("Email already exists".into()));
        }

        self.repo.create(staff).await
    }

    pub async fn get_staff(&self, id: Uuid) -> Result<Option<Staff>, AppError> {
        self.repo.find_by_id(id).await
    }
    pub async fn update_staff(&self, staff: Staff) -> Result<(), AppError> {
        let existing = self.repo.find_by_id(staff.id).await?;
        let existing = match existing {
            Some(s) => s,
            None => return Err(AppError::NotFound("Staff not found".into())),
        };

        // prevent email collision
        if staff.email != existing.email && self.repo.exists_by_email(&staff.email).await? {
            return Err(AppError::Conflict("Email already exists".into()));
        }

        self.repo.update(staff).await
    }

    pub async fn delete_staff(&self, id: Uuid) -> Result<(), AppError> {
        self.repo.delete(id).await
    }

    pub async fn inactivate_staff(&self, id: Uuid) -> Result<(), AppError> {
        let mut staff = match self.repo.find_by_id(id).await? {
            Some(s) => s,
            None => return Err(AppError::NotFound("Staff not found".into())),
        };

        if staff.status == StaffStatus::Inactive {
            return Err(AppError::Conflict("Staff already inactive".into()));
        }

        staff.status = StaffStatus::Inactive;
        self.repo.update(staff).await
    }

    pub async fn batch_create(&self, list: Vec<Staff>) -> Result<(), AppError> {
        if list.is_empty() {
            return Err(AppError::Validation("Batch list cannot be empty".into()));
        }

        // detect duplicate email inside batch
        let mut emails = std::collections::HashSet::new();
        for staff in &list {
            if !emails.insert(&staff.email) {
                return Err(AppError::Conflict(format!(
                    "Duplicate email in batch: {}",
                    staff.email
                )));
            }
        }

        self.repo.create_batch(list).await
    }
}
