use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
mod common;
use common::setup_app;

#[tokio::test]
async fn health_check() {
    let (app, _) = setup_app().await;

    let request = Request::get("/health")
        .body(Body::empty())
        .expect("failed to build health request");

    let response = app
        .oneshot(request)
        .await
        .expect("health endpoint failed to respond");

    assert_eq!(response.status(), StatusCode::OK);
}
