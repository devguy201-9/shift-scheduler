use axum::body::Body;
use axum::http::Request;
use data_service::app_state::AppState;
use data_service::build_app;
use data_service::infrastructure::cache::RedisCache;
use data_service::infrastructure::db::init_pool;
use reqwest::StatusCode;
use sqlx::PgPool;
use tower::ServiceExt;

async fn setup_app() -> (axum::Router, PgPool) {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").unwrap();
    let redis_url = std::env::var("REDIS_URL").unwrap();

    let pool = init_pool(&db_url).await.unwrap();
    let redis = RedisCache::new(&redis_url).unwrap();

    clean_db(&pool).await;

    let state = AppState::new(pool.clone(), redis);

    (build_app(state), pool)
}

async fn clean_db(pool: &PgPool) {
    sqlx::query!("TRUNCATE group_memberships CASCADE")
        .execute(pool)
        .await
        .unwrap();

    sqlx::query!("TRUNCATE staff_groups CASCADE")
        .execute(pool)
        .await
        .unwrap();

    sqlx::query!("TRUNCATE staff CASCADE")
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn staff_test_full() {
    let (app, pool) = setup_app().await;

    let file_data_test =
        std::fs::read_to_string("sample-data/staff.json").expect("staff file doesn't exist");

    // Create batch staff
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/staff/batch")
                .header("content-type", "application/json")
                .body(Body::from(file_data_test))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Get one staff by email
    let record = sqlx::query!(
        "SELECT id FROM staff WHERE email = $1",
        "thuan_integration@example.com"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let staff_id = record.id;

    // Get staff
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/staff/{}", staff_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
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
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Delete staff
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/staff/{}", staff_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
