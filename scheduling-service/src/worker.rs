use crate::application::schedule_service::ScheduleService;
use std::sync::Arc;

pub async fn start_worker(service: Arc<ScheduleService>) -> anyhow::Result<()> {
    loop {
        if let Err(e) = service.process_next_job().await {
            eprintln!("Worker error: {}", e);
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
