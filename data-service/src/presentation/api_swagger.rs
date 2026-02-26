use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::presentation::staff_handler::create_staff,
        crate::presentation::staff_handler::get_staff,
        crate::presentation::staff_handler::update_staff,
        crate::presentation::staff_handler::delete_staff,
        crate::presentation::staff_handler::batch_create_staff,
        crate::presentation::group_handler::create_group,
        crate::presentation::group_handler::resolved_members,
        crate::presentation::group_handler::add_member,
        crate::presentation::group_handler::remove_member,
        crate::presentation::group_handler::update_group,
        crate::presentation::group_handler::delete_group,
        crate::presentation::group_handler::batch_create_group,
    ),
    components(
        schemas(
            crate::presentation::staff_handler::CreateStaffRequest,
            crate::presentation::staff_handler::UpdateStaffRequest,
            crate::presentation::staff_handler::BatchCreateStaffRequest,
            crate::presentation::staff_handler::StaffResponse,
            crate::presentation::group_handler::CreateGroupRequest,
            crate::presentation::group_handler::UpdateGroupRequest,
            crate::presentation::group_handler::BatchCreateGroupRequest,
        )
    ),
    tags(
        (name = "Data Service", description = "Staff & Group APIs")
    )
)]
pub struct ApiDoc;
