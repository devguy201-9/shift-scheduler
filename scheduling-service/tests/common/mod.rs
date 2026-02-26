use axum::{async_trait, Router};
use chrono::NaiveDate;
use mockall::mock;
use scheduling_service::application::data_client_trait::DataClient;
use scheduling_service::application::schedule_service::ScheduleService;
use scheduling_service::application::traits::ScheduleRepository;
use scheduling_service::config::RuleConfig;
use scheduling_service::domain::schedule::{ScheduleJob, ShiftAssignment};
use scheduling_service::{
    app_state::AppState,
    presentation::schedule_handler::{create_schedule, get_result, get_status},
};
use shared::types::JobStatus;
use std::sync::Arc;
use uuid::Uuid;

mock! {
    pub Repo {}

    #[async_trait]
    impl ScheduleRepository for Repo {
        async fn insert_job(&self, id: Uuid, staff_group_id: Uuid, period_begin_date: NaiveDate) -> anyhow::Result<()>;
        async fn fetch_pending(&self) -> anyhow::Result<Option<ScheduleJob>>;
        async fn mark_processing(&self, id: Uuid) -> anyhow::Result<()>;
        async fn mark_completed(&self, id: Uuid) -> anyhow::Result<()>;
        async fn mark_failed(&self, id: Uuid, error: &str) -> anyhow::Result<()>;
        async fn save_assignments(&self, job_id: Uuid, assignments: Vec<ShiftAssignment>) -> anyhow::Result<()>;
        async fn get_status(&self, id: Uuid) -> anyhow::Result<Option<JobStatus>>;
        async fn get_result(&self, id: Uuid) -> anyhow::Result<Vec<ShiftAssignment>>;
    }
}

mock! {
    pub Client {}

    #[async_trait]
    impl DataClient for Client {
        async fn get_group_members(&self, group_id: Uuid) -> anyhow::Result<Vec<Uuid>>;
    }
}

pub async fn build_test_app() -> Router {
    let repo = Arc::new(MockRepo::new());
    let client = Arc::new(MockClient::new());

    let config = RuleConfig {
        min_day_off_per_week: 1,
        max_day_off_per_week: 3,
        no_morning_after_evening: true,
        max_daily_shift_diff: 2,
    };

    let schedule_service = Arc::new(ScheduleService::new(repo, client, config));

    let state = AppState { schedule_service };

    Router::new()
        .route("/api/v1/schedules", axum::routing::post(create_schedule))
        .route(
            "/api/v1/schedules/:id/status",
            axum::routing::get(get_status),
        )
        .route(
            "/api/v1/schedules/:id/result",
            axum::routing::get(get_result),
        )
        .with_state(state)
}
