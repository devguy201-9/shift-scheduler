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
        async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ScheduleJob>>;
    }
}

mock! {
    pub Client {}

    #[async_trait]
    impl DataClient for Client {
        async fn get_group_members(&self, group_id: Uuid) -> anyhow::Result<Vec<Uuid>>;
    }
}

pub fn default_test_config() -> RuleConfig {
    RuleConfig {
        min_day_off_per_week: 1,
        max_day_off_per_week: 3,
        no_morning_after_evening: true,
        max_daily_shift_diff: 2,
    }
}

#[allow(dead_code)]
pub async fn build_test_app() -> Router {
    let mut repo = MockRepo::new();
    let client = MockClient::new();

    repo.expect_insert_job().returning(|_, _, _| Ok(()));

    repo.expect_get_status()
        .returning(|_| Ok(Some(JobStatus::Pending)));

    repo.expect_get_result().returning(|_| Ok(vec![]));

    repo.expect_find_by_id().returning(|id| {
        Ok(Some(ScheduleJob {
            id,
            staff_group_id: Uuid::new_v4(),
            period_begin_date: NaiveDate::from_ymd_opt(2025, 1, 6)
                .expect("invalid static test date"),
            status: JobStatus::Pending,
            error_message: None,
            created_at: None,
            updated_at: None,
        }))
    });

    let config = default_test_config();

    let service = Arc::new(ScheduleService::new(
        Arc::new(repo),
        Arc::new(client),
        config,
    ));

    let state = AppState {
        schedule_service: service,
    };

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
