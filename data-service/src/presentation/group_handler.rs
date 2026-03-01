use crate::app_state::AppState;
use crate::application::error::AppError;
use crate::domain::group::StaffGroup;
use crate::presentation::staff_handler::StaffResponse;
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct CreateGroupRequest {
    pub name: String,
    pub parent_group_id: Option<Uuid>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateGroupRequest {
    pub name: String,
    pub parent_group_id: Option<Uuid>,
}

#[derive(Deserialize, ToSchema)]
pub struct BatchCreateGroupRequest {
    pub items: Vec<CreateGroupRequest>,
}

#[utoipa::path(
    post,
    path = "/api/v1/groups",
    request_body = CreateGroupRequest,
    responses(
        (status = 201, description = "Group created"),
        (status = 400, description = "Validation error")
    ),
    tag = "Data Service"
)]
pub async fn create_group(
    State(state): State<AppState>,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<StatusCode, AppError> {
    let group = StaffGroup {
        id: Uuid::new_v4(),
        name: payload.name,
        parent_group_id: payload.parent_group_id,
    };

    state.group_service.create_group(group).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    get,
    path = "/api/v1/groups/{id}/resolved-members",
    params(
        ("id" = Uuid, Path, description = "Group ID")
    ),
    responses(
        (status = 200, description = "Resolved members", body = [StaffResponse])
    ),
    tag = "Data Service"
)]
pub async fn resolved_members(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<Vec<StaffResponse>>, AppError> {
    let members = state.group_service.resolve_members(group_id).await?;
    Ok(Json(members.into_iter().map(StaffResponse::from).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/groups/{group_id}/members/{staff_id}",
    params(
        ("group_id" = Uuid, Path, description = "Group ID"),
        ("staff_id" = Uuid, Path, description = "Staff ID")
    ),
    responses(
        (status = 204, description = "Member added")
    ),
    tag = "Data Service"
)]
pub async fn add_member(
    State(state): State<AppState>,
    Path((group_id, staff_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    state.group_service.add_member(group_id, staff_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/groups/{group_id}/members/{staff_id}",
    params(
        ("group_id" = Uuid, Path, description = "Group ID"),
        ("staff_id" = Uuid, Path, description = "Staff ID")
    ),
    responses(
        (status = 204, description = "Member removed")
    ),
    tag = "Data Service"
)]
pub async fn remove_member(
    State(state): State<AppState>,
    Path((group_id, staff_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    state
        .group_service
        .remove_member(group_id, staff_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
#[utoipa::path(
    put,
    path = "/api/v1/groups/{id}",
    params(
        ("id" = Uuid, Path, description = "Group ID")
    ),
    request_body = UpdateGroupRequest,
    responses(
        (status = 204, description = "Group updated"),
        (status = 400, description = "Validation error")
    ),
    tag = "Data Service"
)]

pub async fn update_group(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateGroupRequest>,
) -> Result<StatusCode, AppError> {
    let group = StaffGroup {
        id,
        name: payload.name,
        parent_group_id: payload.parent_group_id,
    };

    state.group_service.update_group(group).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/groups/{id}",
    params(
        ("id" = Uuid, Path, description = "Group ID")
    ),
    responses(
        (status = 204, description = "Group deleted")
    ),
    tag = "Data Service"
)]
pub async fn delete_group(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.group_service.delete_group(id).await?;

    Ok(StatusCode::NO_CONTENT)
}
#[utoipa::path(
    post,
    path = "/api/v1/groups/batch",
    request_body = BatchCreateGroupRequest,
    responses(
        (status = 201, description = "Groups created")
    ),
    tag = "Data Service"
)]
pub async fn batch_create_group(
    State(state): State<AppState>,
    Json(payload): Json<BatchCreateGroupRequest>,
) -> Result<StatusCode, AppError> {
    let groups = payload
        .items
        .into_iter()
        .map(|item| StaffGroup {
            id: Uuid::new_v4(),
            name: item.name,
            parent_group_id: item.parent_group_id,
        })
        .collect();

    state.group_service.batch_create(groups).await?;

    Ok(StatusCode::CREATED)
}
