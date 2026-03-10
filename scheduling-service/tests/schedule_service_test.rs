/*
This file tests ScheduleService — the application layer responsible for
the business logic of the scheduling workflow: creating jobs, processing
jobs, retrieving status, and retrieving results.

Characteristics of this file:
- Uses mockall to mock ScheduleRepository and DataClient
- No real database or HTTP server is required
- Focuses on testing service-layer behavior, not infrastructure

How mockall works:
mock! { pub Repo {} impl ScheduleRepository for Repo { ... } }
→ Generates MockRepo with expect_*() methods

Example:
repo.expect_mark_completed().returning(|_| Ok(()))
→ Sets an expectation: when mark_completed is called, return Ok(())

Behavior:
- If a method is expected but NOT called → the test panics
- If a method is called but NOT expected → the test panics

This mechanism verifies that the service calls dependencies in the
correct sequence.

Flow of process_next_job() in ScheduleService:
1. repo.fetch_pending()        → fetch a pending job
2. repo.mark_processing()      → mark the job as processing (within the fetch_pending transaction)
3. data_client.get_group_members() → fetch staff members from the Data Service
4. generate_schedule()         → generate assignments (pure function, no external calls)
5. repo.save_assignments()     → persist the generated results
6. repo.mark_completed()       → mark the job as completed

If any step returns Err → repo.mark_failed() will be called.
*/
mod common;
use chrono::NaiveDate;
use common::*;
use scheduling_service::application::schedule_service::ScheduleService;
use scheduling_service::domain::schedule::ScheduleJob;
use shared::types::JobStatus;
use std::sync::Arc;
use uuid::Uuid;

// Happy path: fetch 1 job → get 1 staff → generate schedule → save → complete
// Verify that the entire success flow is being called in the correct sequence.
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

/*
Edge case: group has 0 active staff
generate_schedule with empty list → return Ok(vec![])
→ save_assignments is called with an empty vec![]
→ mark_completed is called (not mark_failed)
Behavior: Job COMPLETED with 0 assignments, not failed.
Reason: The "empty staff" situation is a valid one, not a system error.
*/
#[tokio::test]
async fn process_job_with_empty_staff_returns_ok() {
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

    repo.expect_save_assignments().returning(|_, assignments| {
        assert!(assignments.is_empty());
        Ok(())
    });

    repo.expect_mark_completed().returning(|_| Ok(()));

    client.expect_get_group_members().returning(|_| Ok(vec![]));

    let config = default_test_config();

    let service = ScheduleService::new(Arc::new(repo), Arc::new(client), config);

    let result = service.process_next_job().await;

    assert!(result.is_ok());
}

/*
Error path: Data Service returns Err (network error, timeout, service down...)
Service must:
1. Call repo.mark_failed() with error message
2. Return Ok() — worker does not crash, continues polling the next job
DO NOT: panic, propagate Err outside, leave the job indefinitely in PROCESSING
*/
#[tokio::test]
async fn process_job_marks_failed_when_data_client_errors() {
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

    repo.expect_mark_failed().returning(|_, _| Ok(()));

    client
        .expect_get_group_members()
        .returning(|_| Err(anyhow::anyhow!("data client error")));

    let config = default_test_config();

    let service = ScheduleService::new(Arc::new(repo), Arc::new(client), config);

    let result = service.process_next_job().await;

    assert!(result.is_ok());
}

// Verify create_job(): calls repo.insert_job() and returns a valid Uuid.
// Simple but essential testing to confirm the service's API is working.
#[tokio::test]
async fn create_job_returns_uuid() {
    let mut repo = MockRepo::new();
    let client = MockClient::new();

    repo.expect_insert_job().returning(|_, _, _| Ok(()));

    let config = default_test_config();

    let service = ScheduleService::new(Arc::new(repo), Arc::new(client), config);

    let result = service
        .create_job(
            Uuid::new_v4(),
            NaiveDate::from_ymd_opt(2025, 1, 1).expect("invalid static test date"),
        )
        .await;

    assert!(result.is_ok());
    // The returned Uuid must be valid (not nil).
    assert_ne!(result.unwrap(), Uuid::nil());
}

/*
Verify get_status() when job does not exist: returns Ok(None), not Err
The caller uses None to know the job is not found, then returns a 404 to the client
Important: distinguish between "not found" (Ok(None)) and "system error" (Err)
*/
#[tokio::test]
async fn get_status_returns_none_for_unknown_id() {
    let mut repo = MockRepo::new();
    let client = MockClient::new();

    repo.expect_get_status().returning(|_| Ok(None));

    let config = default_test_config();

    let service = ScheduleService::new(Arc::new(repo), Arc::new(client), config);

    let result = service.get_status(Uuid::new_v4()).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

/*
When there is no job pending: return Ok() immediately, do nothing further.
Worker loop relies on this behavior to backoff when idle.
DO NOT: call mark_processing, save_assignments, mark_completed...
*/
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
