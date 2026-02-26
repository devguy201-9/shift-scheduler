use axum::async_trait;
use chrono::NaiveDate;
use mockall::mock;
use scheduling_service::application::data_client_trait::DataClient;
use scheduling_service::application::schedule_service::ScheduleService;
use scheduling_service::application::traits::ScheduleRepository;
use scheduling_service::config::RuleConfig;
use scheduling_service::domain::schedule::{ScheduleJob, ShiftAssignment};
use std::sync::Arc;
use uuid::Uuid;
use shared::types::JobStatus;

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

#[tokio::test]
async fn process_next_job_success() {
    fn mock_config() -> RuleConfig {
        RuleConfig {
            min_day_off_per_week: 1,
            max_day_off_per_week: 3,
            no_morning_after_evening: true,
            max_daily_shift_diff: 2,
        }
    }

    let mut repo = MockRepo::new();
    let mut client = MockClient::new();

    let job_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();

    let job = ScheduleJob {
        id: job_id,
        staff_group_id: group_id,
        period_begin_date: start_date,
        status: JobStatus::Pending,
        error_message: None,
        created_at: None,
        updated_at: None,
    };

    repo.expect_fetch_pending()
        .returning(move || Ok(Some(job.clone())));

    repo.expect_mark_processing().returning(|_| Ok(()));

    repo.expect_mark_completed().returning(|_| Ok(()));

    repo.expect_save_assignments().returning(|_, _| Ok(()));

    client
        .expect_get_group_members()
        .returning(|_| Ok(vec![Uuid::new_v4()]));

    let service = ScheduleService::new(Arc::new(repo), Arc::new(client), mock_config());

    let result = service.process_next_job().await;

    assert!(result.is_ok());
}
