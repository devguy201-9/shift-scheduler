use data_service::app_state::AppState;
use data_service::infrastructure::cache::RedisCache;
use data_service::infrastructure::db::init_pool;
use data_service::{build_app, load_env};
use sqlx::migrate;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    _ = load_env();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let pool = init_pool(&database_url)
        .await
        .expect("Failed to connect to database");

    migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let redis = RedisCache::new(&redis_url).expect("Failed to create redis client");

    let state = AppState::new(pool, redis);

    let app = build_app(state);

    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind TCP listener");

    axum::serve(listener, app).await.expect("Server failed");
}
