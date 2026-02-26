use axum::body::Body;
use axum::http::{Request, StatusCode};
mod common;
use common::build_test_app;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn create_schedule_endpoint() {
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
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

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
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
