use axum::body::{Body, to_bytes};
use axum::http::Request;
use data_service::app_state::AppState;
use data_service::build_app;
use data_service::infrastructure::cache::RedisCache;
use data_service::infrastructure::db::init_pool;
use reqwest::StatusCode;

async fn setup_app() -> axum::Router {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").unwrap();
    let redis_url = std::env::var("REDIS_URL").unwrap();

    let pool = init_pool(&db_url).await.unwrap();
    let redis = RedisCache::new(&redis_url).unwrap();

    let state = AppState::new(pool, redis);

    build_app(state)
}

#[tokio::test]
async fn group_test_full() {
    let app = setup_app().await;

    let file_data_test =
        std::fs::read_to_string("sample-data/group.json").expect("group file doesn't exist");

    // Create batch group
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/groups/batch")
        .header("content-type", "application/json")
        .body(Body::from(file_data_test))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Get one group by name
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let pool = init_pool(&db_url).await.unwrap();

    let group = sqlx::query!("SELECT id FROM staff_groups WHERE name = $1", "Engineering")
        .fetch_one(&pool)
        .await
        .unwrap();

    let group_id = group.id;

    // Create batch staff
    let staff_data =
        std::fs::read_to_string("sample-data/staff.json").expect("staff file doesn't exist");

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/staff/batch")
        .header("content-type", "application/json")
        .body(Body::from(staff_data))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let staff_rows = sqlx::query!("SELECT id FROM staff")
        .fetch_all(&pool)
        .await
        .unwrap();

    // Add all staff to group
    for staff in staff_rows {
        let request = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/groups/{}/members/{}", group_id, staff.id))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    // Verify resolve member
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/groups/{}/resolved-members", group_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let members: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert_eq!(members.len(), staff_rows.len());
}
