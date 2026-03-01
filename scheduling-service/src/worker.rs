use crate::application::schedule_service::ScheduleService;
use std::sync::Arc;

pub async fn start_worker(service: Arc<ScheduleService>) -> anyhow::Result<()> {
    let mut backoff_secs = 1u64;
    const MAX_BACKOFF: u64 = 10;
    loop {
        match service.process_next_job().await {
            Ok(()) => {
                backoff_secs = 1;
            }
            Err(e) => eprintln!("worker error: {}", e),
        }

        // No job or error: back off to avoid hammering DB when idle
        tokio::time::sleep(std::time::Duration::from_secs(backoff_secs)).await;
        if backoff_secs < MAX_BACKOFF {
            backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF);
        }
    }
}
