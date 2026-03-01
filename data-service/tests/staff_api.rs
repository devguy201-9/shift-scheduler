use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
mod common;
use common::setup_app;

#[tokio::test]
async fn staff_test_full() {
    let (app, pool) = setup_app().await;

    let file_data_test = include_str!("../../sample-data/staff.json");

    // Create batch staff
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/staff/batch")
                .header("content-type", "application/json")
                .body(Body::from(file_data_test))
                .expect("failed to build create-staff request"),
        )
        .await
        .expect("create-staff request failed");
    assert_eq!(response.status(), StatusCode::CREATED);

    // Get one staff by email
    let record = sqlx::query!(
        "SELECT id FROM staff WHERE email = $1",
        "thuan_integration@example.com"
    )
    .fetch_one(&pool)
    .await
    .expect("failed to fetch inserted staff by email");

    let staff_id = record.id;

    // Get staff
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/staff/{}", staff_id))
                .body(Body::empty())
                .expect("failed to build get-staff by id request"),
        )
        .await
        .expect("get-staff by id request failed");
    assert_eq!(response.status(), StatusCode::OK);

    // Update staff
    let update_json = r#"
    {
        "name": "Tran Updated",
        "email": "tranthuan_integration@example.com",
        "position": "Senior Developer",
        "status": "ACTIVE"
    }
    "#;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/staff/{}", staff_id))
                .header("content-type", "application/json")
                .body(Body::from(update_json))
                .expect("failed to build update-staff by id request"),
        )
        .await
        .expect("update-staff by id request failed");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Delete staff
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/staff/{}", staff_id))
                .body(Body::empty())
                .expect("failed to build delete-staff by id request"),
        )
        .await
        .expect("delete-staff by id request failed");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn staff_create_duplicate_email_should_conflict() {
    let (app, _) = setup_app().await;

    let staff_json = r#"
    {
        "items": [
            {
                "id": "11111111-1111-1111-1111-111111111111",
                "name": "Test User",
                "email": "duplicate@example.com",
                "position": "Dev",
                "status": "ACTIVE"
            }
        ]
    }
    "#;

    // First insert
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/staff/batch")
                .header("content-type", "application/json")
                .body(Body::from(staff_json))
                .expect("build request failed"),
        )
        .await
        .expect("first insert failed");

    // Second insert same email
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/staff/batch")
                .header("content-type", "application/json")
                .body(Body::from(staff_json))
                .expect("build request failed"),
        )
        .await
        .expect("second insert failed");

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn get_non_existing_staff_should_return_404() {
    let (app, _) = setup_app().await;

    let fake_id = uuid::Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/staff/{}", fake_id))
                .body(Body::empty())
                .expect("build request failed"),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
