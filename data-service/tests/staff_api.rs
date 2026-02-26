use axum::body::Body;
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
async fn staff_test_full() {
    let app = setup_app().await;

    let file_data_test =
        std::fs::read_to_string("sample-data/staff.json").expect("staff file doesn't exist");

    // Create batch staff
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/staff/batch")
        .header("content-type", "application/json")
        .body(Body::from(file_data_test))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Get one staff by email
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let pool = init_pool(&db_url).await.unwrap();

    let record = sqlx::query!(
        "SELECT id FROM staff WHERE email = $1",
        "alice_integration@example.com"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let staff_id = record.id;

    // Get staff
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/staff/{}", staff_id))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Update staff
    let update_json = r#"
    {
        "name": "Tran Updated",
        "email": "tran_integration@example.com",
        "position": "Senior Developer",
        "status": "ACTIVE"
    }
    "#;

    let request = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/staff/{}", staff_id))
        .header("content-type", "application/json")
        .body(Body::from(update_json))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Delete staff
    let request = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/staff/{}", staff_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
