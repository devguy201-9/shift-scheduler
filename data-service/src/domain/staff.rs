use serde::{Deserialize, Serialize};
use shared::types::StaffStatus;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Staff {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub position: String,
    pub status: StaffStatus,
}
