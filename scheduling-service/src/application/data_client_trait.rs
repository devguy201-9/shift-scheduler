use axum::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait DataClient: Send + Sync {
    async fn get_group_members(&self, group_id: Uuid) -> anyhow::Result<Vec<Uuid>>;
}
