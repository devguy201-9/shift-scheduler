use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::presentation::schedule_handler::create_schedule,
        crate::presentation::schedule_handler::get_status,
        crate::presentation::schedule_handler::get_result,
    ),
    components(
        schemas(
            crate::presentation::schedule_handler::CreateScheduleRequest,
            crate::presentation::schedule_handler::CreateScheduleResponse,
            crate::presentation::schedule_handler::ScheduleResultResponse,
            crate::presentation::schedule_handler::AssignmentResponse,
        )
    ),
    tags(
        (name = "Scheduling Service", description = "Schedule generation APIs")
    )
)]
pub struct ApiDoc;
