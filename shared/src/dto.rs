use crate::types::ShiftType;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct ScheduleRequestView {
    pub staff_group_id: Uuid,
    pub period_begin_date: NaiveDate,
}

#[derive(Serialize, Deserialize)]
pub struct ShiftAssignmentView {
    pub staff_id: Uuid,
    pub date: NaiveDate,
    pub shift: ShiftType,
}
