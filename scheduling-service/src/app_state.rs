use crate::application::schedule_service::ScheduleService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub schedule_service: Arc<ScheduleService>,
}
