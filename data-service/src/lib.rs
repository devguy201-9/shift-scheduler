use crate::app_state::AppState;
use crate::presentation::api_swagger::ApiDoc;
use crate::presentation::{group_handler::*, staff_handler::*};
use axum::Router;
use axum::routing::{delete, get, post, put};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod app_state;
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

pub fn build_app(state: AppState) -> Router {
    Router::new()
        // Health
        .route("/health", get(|| async { "OK" }))
        // API v1
        .nest("/api/v1", api_v1_routes())
        // Swagger
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}

fn api_v1_routes() -> Router<AppState> {
    Router::new()
        // Staff CRUD
        .route("/staff", post(create_staff))
        .route("/staff/:id", get(get_staff))
        .route("/staff/:id", put(update_staff))
        .route("/staff/:id", delete(delete_staff))
        .route("/staff/batch", post(batch_create_staff))
        // Group
        .route("/groups/:id/resolved-members", get(resolved_members))
        .route("/groups/:group_id/members/:staff_id", post(add_member))
        .route("/groups/:group_id/members/:staff_id", delete(remove_member))
        .route("/groups", post(create_group))
        .route("/groups/:id", put(update_group))
        .route("/groups/:id", delete(delete_group))
        .route("/groups/batch", post(batch_create_group))
}
