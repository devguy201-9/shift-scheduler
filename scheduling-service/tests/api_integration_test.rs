/*
This file test HTTP layer of scheduling-service — from request
The response is output by the handler via the service, but not through the actual database.

Setup: (read common/mod.rs):
  build_test_app() create Router with MockRepo is created pre-configure:
    - insert_job → Ok(())
    - get_status → Ok(Some(JobStatus::Pending)) ← all ids were response Pending
    - get_result → Ok(vec![])
    - find_by_id → Ok(Some(job)) ← all ids were found

  Exception : Uuid::nil() (all zeros) → mock response Ok(None) → handler response 404
  → This is convention used in test: nil UUID = "not existed"

How to test HTTP with axum:
  app.oneshot(request) → send request into router, receive response
  Don't need start server, don't need bind port

Scope of this file:
  Status code is correct (202, 400, 404, 200)
  Request validation (non-Monday date → 400)
  Response body have field required (schedule_id)
  Don't test worker execution (async, need DB)
  Don't test schedule content (need the worker to finish running)
*/

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
mod common;
use common::build_test_app;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

/*
POST /api/v1/schedules with Monday date valid → 202 Accepted
202 (not 201) because schedule processed ASYNC:
  - Request response with job_id
  - Worker process schedule generation in the background
  - Client poll GET /status to know when it's finished
*/
#[tokio::test]
async fn create_schedule_endpoint() {
    let app = build_test_app().await;

    let payload = json!({
        "staff_group_id": Uuid::new_v4(),
        "period_begin_date": "2025-01-06"
    });

    let response = app
        .oneshot(
            Request::post("/api/v1/schedules")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .expect("failed to build create-schedule request"),
        )
        .await
        .expect("create-schedule request failed");

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

/*
GET /api/v1/schedules/{nil_uuid}/status → 404 Not Found
Uuid::nil() = "00000000-0000-0000-0000-0000-00000000000"
Mock configured: get_status(nil) → Ok(None) → handler returns 404
This test verifies the handler handles "job does not exist" correctly
*/
#[tokio::test]
async fn get_status_returns_404_for_unknown_id() {
    let app = build_test_app().await;

    let id = Uuid::nil();

    let response = app
        .oneshot(
            Request::get(format!("/api/v1/schedules/{}/status", id))
                .body(Body::empty())
                .expect("failed to build status request"),
        )
        .await
        .expect("status request failed");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/*
GET /api/v1/schedules/{nil_uuid}/result → 404 Not Found
Similar to get_status, test the get_result handler when the job does not exist.
Important: You must check if the job exists before returning the result.
If the job does not exist but returns 200 with an empty array → the client misunderstands.
*/
#[tokio::test]
async fn get_result_returns_404_for_unknown_id() {
    let app = build_test_app().await;

    let id = Uuid::nil();

    let response = app
        .oneshot(
            Request::get(format!("/api/v1/schedules/{}/result", id))
                .body(Body::empty())
                .expect("failed to build result request"),
        )
        .await
        .expect("result request failed");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/*
Create a schedule → retrieve schedule_id from the response → get status → expect 200 OK.

This test verifies:
1. The POST response body contains a "schedule_id" field
2. The schedule_id is a valid UUID that can be used for querying
3. GET /status with that ID returns 200 (meaning the job exists)

The test does not assert the exact status value (PENDING / PROCESSING / COMPLETED)
because the mock always returns Pending. The important part is that the endpoint
works correctly and the ID can be used immediately after creation.
*/
#[tokio::test]
async fn get_status_returns_pending_after_create() {
    let app = build_test_app().await;

    let payload = json!({
        "staff_group_id": Uuid::new_v4(),
        "period_begin_date": "2025-01-06"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/schedules")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .expect("failed to build create request"),
        )
        .await
        .expect("create request failed");

    assert_eq!(create_response.status(), StatusCode::ACCEPTED);

    let body = create_response
        .into_body()
        .collect()
        .await
        .expect("read body failed")
        .to_bytes();

    let json: serde_json::Value = serde_json::from_slice(&body).expect("invalid json response");

    let id = json["schedule_id"].as_str().expect("missing schedule_id");

    let response = app
        .oneshot(
            Request::get(format!("/api/v1/schedules/{}/status", id))
                .body(Body::empty())
                .expect("failed to build status request"),
        )
        .await
        .expect("status request failed");

    assert_eq!(response.status(), StatusCode::OK);
}

/*
POST with a date other than Monday → 400 Bad Request
Test requirement: period_begin_date must be Monday
Handler must validate before creating the job — do not let the job fail after creation
2025-01-01 is Wednesday → reject immediately at the validation layer
*/
#[tokio::test]
async fn create_schedule_rejects_non_monday() {
    let app = build_test_app().await;

    let payload = json!({
        "staff_group_id": Uuid::new_v4(),
        "period_begin_date": "2025-01-01"
    });

    let response = app
        .oneshot(
            Request::post("/api/v1/schedules")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .expect("failed to build create-schedule request"),
        )
        .await
        .expect("create-schedule request failed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
