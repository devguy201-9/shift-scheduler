use data_service::infrastructure::cache::RedisCache;
use data_service::infrastructure::db::init_pool;
use data_service::{app_state::AppState, build_app};

async fn setup_app() -> axum::Router {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").unwrap();
    let redis_url = std::env::var("REDIS_URL").unwrap();

    let pool = init_pool(&db_url).await.unwrap();
    let redis = RedisCache::new(&redis_url).unwrap();

    let state = AppState::new(pool, redis);

    build_app(state)
}
