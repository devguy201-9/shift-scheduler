use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
mod common;
use common::setup_app;

#[tokio::test]
async fn group_test_full() {
    let (app, pool) = setup_app().await;

    let file_data_test = include_str!("../../sample-data/group.json");

    // Create batch group
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/groups/batch")
                .header("content-type", "application/json")
                .body(Body::from(file_data_test))
                .expect("failed to build create group request"),
        )
        .await
        .expect("create group request failed");

    assert_eq!(response.status(), StatusCode::CREATED);

    // Get one group by name
    let group = sqlx::query!("SELECT id FROM staff_groups WHERE name = $1", "Engineering")
        .fetch_one(&pool)
        .await
        .expect("failed to fetch Engineering group from DB");

    let group_id = group.id;

    // Create batch staff
    let staff_data = include_str!("../../sample-data/staff.json");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/staff/batch")
                .header("content-type", "application/json")
                .body(Body::from(staff_data))
                .expect("failed to build create staff request"),
        )
        .await
        .expect("create staff request failed");
    assert_eq!(response.status(), StatusCode::CREATED);

    let staff_rows = sqlx::query!("SELECT id FROM staff")
        .fetch_all(&pool)
        .await
        .expect("failed to fetch staff rows");

    // Add all staff to group
    for staff in &staff_rows {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/groups/{}/members/{}", group_id, staff.id))
                    .body(Body::empty())
                    .expect("failed to build add-member request"),
            )
            .await
            .expect("add-member request failed");
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    // Verify resolve member
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/groups/{}/resolved-members", group_id))
                .body(Body::empty())
                .expect("failed to build verify resolved-members request"),
        )
        .await
        .expect("verify resolved-members request failed");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("failed to read resolved-members body");
    let members: Vec<serde_json::Value> =
        serde_json::from_slice(&body).expect("failed to deserialize resolved-members response");

    assert_eq!(members.len(), staff_rows.len());

    // Cache hit resolve member
    let response = app
        .clone()
        .oneshot(
            Request::get(format!("/api/v1/groups/{}/resolved-members", group_id))
                .body(Body::empty())
                .expect("failed to build resolved-members request"),
        )
        .await
        .expect("resolved-members request failed");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn add_duplicate_member_should_conflict() {
    let (app, pool) = setup_app().await;

    // create group
    let group_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/groups")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"TestGroup"}"#))
                .expect("failed to build create group request"),
        )
        .await
        .expect("create group request failed");

    assert_eq!(group_resp.status(), StatusCode::CREATED);

    // create staff
    let staff_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/staff")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Test","email":"a@test.com","position":"Dev","status":"Active"}"#,
                ))
                .expect("failed to build create staff request"),
        )
        .await
        .expect("create staff request failed");

    assert_eq!(staff_resp.status(), StatusCode::CREATED);

    // get ids
    let group = sqlx::query!("SELECT id FROM staff_groups LIMIT 1")
        .fetch_one(&pool)
        .await
        .expect("fetch group failed");

    let staff = sqlx::query!("SELECT id FROM staff LIMIT 1")
        .fetch_one(&pool)
        .await
        .expect("fetch staff failed");

    let uri = format!("/api/v1/groups/{}/members/{}", group.id, staff.id);

    // First add
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .expect("build request failed"),
        )
        .await
        .expect("first add failed");

    // Second add same member
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .expect("build request failed"),
        )
        .await
        .expect("second add failed");

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn remove_non_existing_member_should_return_404() {
    let (app, pool) = setup_app().await;

    // create group
    let create_group_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/groups")
                .header("content-type", "application/json")
                .body(Body::from(r#"{ "name": "Test Group" }"#))
                .expect("failed to build create group request"),
        )
        .await
        .expect("create group request failed");

    assert_eq!(create_group_response.status(), StatusCode::CREATED);

    let group = sqlx::query!("SELECT id FROM staff_groups WHERE name = $1", "Test Group")
        .fetch_one(&pool)
        .await
        .expect("fetch group failed");

    let fake_staff = uuid::Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/groups/{}/members/{}",
                    group.id, fake_staff
                ))
                .body(Body::empty())
                .expect("build request failed"),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
