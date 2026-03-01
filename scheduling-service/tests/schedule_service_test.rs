mod common;
use chrono::NaiveDate;
use common::*;
use scheduling_service::application::schedule_service::ScheduleService;
use scheduling_service::domain::schedule::ScheduleJob;
use shared::types::JobStatus;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn process_next_job_success() {
    let mut repo = MockRepo::new();
    let mut client = MockClient::new();

    let job_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();

    let job = ScheduleJob {
        id: job_id,
        staff_group_id: group_id,
        period_begin_date: NaiveDate::from_ymd_opt(2025, 1, 6).expect("invalid static test date"),
        status: JobStatus::Pending,
        error_message: None,
        created_at: None,
        updated_at: None,
    };

    repo.expect_fetch_pending()
        .returning(move || Ok(Some(job.clone())));

    repo.expect_mark_processing()
        .returning(|_| Ok(()));
    
    repo.expect_save_assignments().returning(|_, _| Ok(()));

    repo.expect_mark_completed().returning(|_| Ok(()));

    client
        .expect_get_group_members()
        .returning(|_| Ok(vec![Uuid::new_v4()]));

    let config = default_test_config();

    let service = ScheduleService::new(Arc::new(repo), Arc::new(client), config);

    let result = service.process_next_job().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn worker_returns_ok_when_no_job() {
    let mut repo = MockRepo::new();
    let client = MockClient::new();

    repo.expect_fetch_pending().times(1).returning(|| Ok(None));

    let config = default_test_config();

    let service = ScheduleService::new(Arc::new(repo), Arc::new(client), config);

    let result = service.process_next_job().await;

    assert!(result.is_ok());
}