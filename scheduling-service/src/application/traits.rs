use crate::domain::schedule::{ScheduleJob, ShiftAssignment};
use axum::async_trait;
use chrono::NaiveDate;
use shared::types::JobStatus;
use uuid::Uuid;

#[async_trait]
pub trait ScheduleRepository: Send + Sync {
    async fn insert_job(
        &self,
        id: Uuid,
        staff_group_id: Uuid,
        period_begin_date: NaiveDate,
    ) -> anyhow::Result<()>;
    async fn fetch_pending(&self) -> anyhow::Result<Option<ScheduleJob>>;
    async fn mark_processing(&self, id: Uuid) -> anyhow::Result<()>;

    async fn mark_completed(&self, id: Uuid) -> anyhow::Result<()>;

    async fn mark_failed(&self, id: Uuid, error: &str) -> anyhow::Result<()>;

    async fn save_assignments(
        &self,
        job_id: Uuid,
        assignments: Vec<ShiftAssignment>,
    ) -> anyhow::Result<()>;

    async fn get_status(&self, id: Uuid) -> anyhow::Result<Option<JobStatus>>;

    async fn get_result(&self, id: Uuid) -> anyhow::Result<Vec<ShiftAssignment>>;
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> anyhow::Result<Option<ScheduleJob>>;
}
