use crate::app_state::AppState;
use crate::application::api_error::ScheduleApiError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Datelike;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct CreateScheduleRequest {
    pub staff_group_id: Uuid,
    pub period_begin_date: NaiveDate,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateScheduleResponse {
    pub schedule_id: Uuid,
    pub status: String,
}

#[derive(Serialize, ToSchema)]
pub struct ScheduleResultResponse {
    pub schedule_id: Uuid,
    pub period_begin_date: NaiveDate,
    pub staff_group_id: Uuid,
    pub assignments: Vec<AssignmentResponse>,
}

#[derive(Serialize, ToSchema)]
pub struct AssignmentResponse {
    pub staff_id: Uuid,
    pub date: NaiveDate,
    pub shift: String,
}

// POST /api/v1/schedules
#[utoipa::path(
    post,
    path = "/api/v1/schedules",
    request_body = CreateScheduleRequest,
    responses(
        (status = 202, description = "Schedule job created", body = CreateScheduleResponse),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Scheduling Service"
)]
#[axum::debug_handler]
pub async fn create_schedule(
    State(state): State<AppState>,
    Json(req): Json<CreateScheduleRequest>,
) -> Result<impl IntoResponse, ScheduleApiError> {
    //must be Monday
    if req.period_begin_date.weekday().number_from_monday() != 1 {
        return Err(ScheduleApiError::Validation(
            "period_begin_date must be a Monday".into(),
        ));
    }

    let job_id = state
        .schedule_service
        .create_job(req.staff_group_id, req.period_begin_date)
        .await
        .map_err(|_| ScheduleApiError::Internal)?;

    Ok((
        StatusCode::ACCEPTED,
        Json(CreateScheduleResponse {
            schedule_id: job_id,
            status: "PENDING".into(),
        }),
    ))
}

// GET /api/v1/schedules/:id/status
#[utoipa::path(
    get,
    path = "/api/v1/schedules/{id}/status",
    params(
        ("id" = Uuid, Path, description = "Schedule ID")
    ),
    responses(
        (status = 200, description = "Current job status", body = String),
        (status = 404, description = "Schedule not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Scheduling Service"
)]
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

// GET /api/v1/schedules/:id/result
#[utoipa::path(
    get,
    path = "/api/v1/schedules/{id}/result",
    params(
        ("id" = Uuid, Path, description = "Schedule ID")
    ),
    responses(
        (status = 200, description = "Schedule result", body = ScheduleResultResponse),
        (status = 404, description = "Schedule not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Scheduling Service"
)]
pub async fn get_result(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ScheduleResultResponse>, ScheduleApiError> {
    let job = state
        .schedule_service
        .get_job(id)
        .await
        .map_err(|_| ScheduleApiError::Internal)?
        .ok_or(ScheduleApiError::NotFound)?;

    let assignments = state
        .schedule_service
        .get_result(id)
        .await
        .map_err(|_| ScheduleApiError::Internal)?;

    let mapped = assignments
        .into_iter()
        .map(|a| AssignmentResponse {
            staff_id: a.staff_id,
            date: a.date,
            shift: a.shift.to_string(),
        })
        .collect();

    Ok(Json(ScheduleResultResponse {
        schedule_id: job.id,
        period_begin_date: job.period_begin_date,
        staff_group_id: job.staff_group_id,
        assignments: mapped,
    }))
}
