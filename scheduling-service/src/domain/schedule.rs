use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use shared::types::{JobStatus, ShiftType};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleJob {
    pub id: Uuid,
    pub staff_group_id: Uuid,
    pub period_begin_date: NaiveDate,
    pub status: JobStatus,
    pub error_message: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftAssignment {
    pub id: Uuid,
    pub staff_id: Uuid,
    pub date: NaiveDate,
    pub shift: ShiftType,
}
