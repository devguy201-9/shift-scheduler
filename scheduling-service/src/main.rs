use scheduling_service::app_state::AppState;
use scheduling_service::application::schedule_service::ScheduleService;
use scheduling_service::infrastructure::db::init_pool;
use scheduling_service::infrastructure::http_data_client::HttpDataClient;
use scheduling_service::infrastructure::schedule_repository::ScheduleRepositoryPg;
use scheduling_service::worker::start_worker;
use scheduling_service::{build_app, load_config, load_env};
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    _ = load_env();
    let config = load_config();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let data_service_url = std::env::var("DATA_SERVICE_URL").expect("DATA_SERVICE_URL must be set");
    let pool = init_pool(&database_url)
        .await
        .expect("Failed to connect to database");

    let repo = Arc::new(ScheduleRepositoryPg::new(pool));

    let data_client = Arc::new(HttpDataClient::new(data_service_url));

    let schedule_service = Arc::new(ScheduleService::new(
        repo.clone(),
        data_client.clone(),
        config.rules.clone(),
    ));

    // Start worker
    let worker_service = schedule_service.clone();
    tokio::spawn(async move {
        if let Err(e) = start_worker(worker_service).await {
            eprintln!("Worker crashed: {}", e);
        }
    });

    let state = AppState { schedule_service };

    let app = build_app(state);

    let listener = TcpListener::bind("0.0.0.0:8081")
        .await
        .expect("Failed to bind port");

    axum::serve(listener, app).await.expect("Server failed");
}
