use crate::app_state::AppState;
use crate::config::AppConfig;
use crate::presentation::schedule_handler::{create_schedule, get_result, get_status};
use axum::Router;
use axum::routing::{get, post};
use std::fs::File;

pub mod app_state;
pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod worker;

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .nest("/api/v1", api_v1_routes())
        .with_state(state)
}

fn api_v1_routes() -> Router<AppState> {
    Router::new()
        .route("/schedules", post(create_schedule))
        .route("/schedules/:id/status", get(get_status))
        .route("/schedules/:id/result", get(get_result))
}

pub fn load_config() -> AppConfig {
    let file = File::open("config.yaml").expect("config.yaml not found");
    serde_yaml::from_reader(file).expect("Invalid config.yaml")
}
