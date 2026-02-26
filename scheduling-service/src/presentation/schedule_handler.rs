use crate::app_state::AppState;
use crate::application::api_error::ScheduleApiError;
use crate::domain::schedule::ShiftAssignment;
use axum::{
    Json,
    extract::{Path, State},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateScheduleRequest {
    pub staff_group_id: Uuid,
    pub period_begin_date: NaiveDate,
}

#[derive(Serialize)]
pub struct CreateScheduleResponse {
    pub schedule_id: Uuid,
    pub status: String,
}

// POST /api/v1/schedules
pub async fn create_schedule(
    State(state): State<AppState>,
    Json(req): Json<CreateScheduleRequest>,
) -> Result<Json<CreateScheduleResponse>, ScheduleApiError> {
    let job_id = state
        .schedule_service
        .create_job(req.staff_group_id, req.period_begin_date)
        .await
        .map_err(|_| ScheduleApiError::Internal)?;

    Ok(Json(CreateScheduleResponse {
        schedule_id: job_id,
        status: "PENDING".into(),
    }))
}

// GET /status
pub async fn get_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<String>, ScheduleApiError> {
    let status = state
        .schedule_service
        .get_status(id)
        .await
        .map_err(|_| ScheduleApiError::Internal)?;

    match status {
        Some(s) => Ok(Json(s.as_str().to_string())),
        None => Err(ScheduleApiError::NotFound),
    }
}

// GET /result
pub async fn get_result(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ShiftAssignment>>, ScheduleApiError> {
    let result = state
        .schedule_service
        .get_result(id)
        .await
        .map_err(|_| ScheduleApiError::Internal)?;

    Ok(Json(result))
}
