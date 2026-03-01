use crate::app_state::AppState;
use crate::application::error::AppError;
use crate::domain::staff::Staff;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use shared::types::StaffStatus;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct CreateStaffRequest {
    pub name: String,
    pub email: String,
    pub position: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateStaffRequest {
    pub name: String,
    pub email: String,
    pub position: String,
    pub status: String,
}

#[derive(Deserialize, ToSchema)]
pub struct BatchCreateStaffRequest {
    pub items: Vec<CreateStaffRequest>,
}

#[derive(Serialize, ToSchema)]
pub struct StaffResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub position: String,
    pub status: String,
}
impl From<Staff> for StaffResponse {
    fn from(staff: Staff) -> Self {
        Self {
            id: staff.id.to_string(),
            name: staff.name,
            email: staff.email,
            position: staff.position,
            status: staff.status.to_string(),
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/staff",
    request_body = CreateStaffRequest,
    responses(
        (status = 201, description = "Staff created"),
        (status = 400, description = "Validation error")
    )
)]
pub async fn create_staff(
    State(state): State<AppState>,
    Json(payload): Json<CreateStaffRequest>,
) -> Result<impl IntoResponse, AppError> {
    let staff = Staff {
        id: Uuid::new_v4(),
        name: payload.name,
        email: payload.email,
        position: payload.position,
        status: StaffStatus::Active,
    };

    state.staff_service.create_staff(staff).await?;

    Ok((StatusCode::CREATED, "Created"))
}

#[utoipa::path(
    get,
    path = "/api/v1/staff/{id}",
    params(
        ("id" = Uuid, Path, description = "Staff ID")
    ),
    responses(
        (status = 200, description = "Staff found", body = StaffResponse),
        (status = 404, description = "Not found")
    ),
    tag = "Data Service"
)]
pub async fn get_staff(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<StaffResponse>, AppError> {
    let staff = state
        .staff_service
        .get_staff(id)
        .await?
        .ok_or(AppError::NotFound("Staff not found".into()))?;

    Ok(Json(staff.into()))
}

#[utoipa::path(
    put,
    path = "/api/v1/staff/{id}",
    params(
        ("id" = Uuid, Path, description = "Staff ID")
    ),
    request_body = UpdateStaffRequest,
    responses(
        (status = 204, description = "Updated"),
        (status = 400, description = "Validation error")
    ),
    tag = "Data Service"
)]
pub async fn update_staff(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStaffRequest>,
) -> Result<StatusCode, AppError> {
    let status = match payload.status.as_str() {
        "ACTIVE" => StaffStatus::Active,
        "INACTIVE" => StaffStatus::Inactive,
        _ => return Err(AppError::Validation("Invalid status".into())),
    };

    let staff = Staff {
        id,
        name: payload.name,
        email: payload.email,
        position: payload.position,
        status,
    };

    state.staff_service.update_staff(staff).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/staff/{id}",
    params(
        ("id" = Uuid, Path, description = "Staff ID")
    ),
    responses(
        (status = 204, description = "Deleted")
    ),
    tag = "Data Service"
)]
pub async fn delete_staff(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.staff_service.delete_staff(id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/staff/batch",
    request_body = BatchCreateStaffRequest,
    responses(
        (status = 201, description = "Batch staff created"),
        (status = 400, description = "Validation error")
    ),
    tag = "Data Service"
)]
pub async fn batch_create_staff(
    State(state): State<AppState>,
    Json(payload): Json<BatchCreateStaffRequest>,
) -> Result<StatusCode, AppError> {
    let staff_list = payload
        .items
        .into_iter()
        .map(|item| Staff {
            id: Uuid::new_v4(),
            name: item.name,
            email: item.email,
            position: item.position,
            status: StaffStatus::Active,
        })
        .collect();

    state.staff_service.batch_create(staff_list).await?;

    Ok(StatusCode::CREATED)
}
