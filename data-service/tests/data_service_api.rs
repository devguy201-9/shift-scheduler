use axum::body::Body;
use axum::http::{Request, StatusCode};
use data_service::infrastructure::cache::RedisCache;
use data_service::infrastructure::db::init_pool;
use data_service::{app_state::AppState, build_app};
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
async fn health_check() {
    let (app, _) = setup_app().await;

    let response = app
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
