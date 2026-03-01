use data_service::app_state::AppState;
use data_service::build_app;
use data_service::infrastructure::cache::RedisCache;
use data_service::infrastructure::db::init_pool;
use sqlx::PgPool;

pub async fn setup_app() -> (axum::Router, PgPool) {
    dotenvy::from_filename(".env.test").ok();

    let db_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");

    let redis_url =
        std::env::var("REDIS_URL").expect("REDIS_URL must be set for integration tests");

    let pool = init_pool(&db_url)
        .await
        .expect("failed to initialize database pool");

    let redis = RedisCache::new(&redis_url).expect("failed to initialize Redis cache");

    clean_db(&pool).await;

    let state = AppState::new(pool.clone(), redis);

    (build_app(state), pool)
}

pub async fn clean_db(pool: &PgPool) {
    sqlx::query!("TRUNCATE group_memberships CASCADE")
        .execute(pool)
        .await
        .expect("failed to truncate group_memberships");

    sqlx::query!("TRUNCATE staff_groups CASCADE")
        .execute(pool)
        .await
        .expect("failed to truncate staff_groups");

    sqlx::query!("TRUNCATE staff CASCADE")
        .execute(pool)
        .await
        .expect("failed to truncate staff");
}
