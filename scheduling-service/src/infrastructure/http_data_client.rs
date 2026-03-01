use crate::application::data_client_trait::DataClient;
use axum::async_trait;
use reqwest::Client;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct StaffResponse {
    id: Uuid,
}

pub struct HttpDataClient {
    client: Client,
    base_url: String,
}

impl HttpDataClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }
}

#[async_trait]
impl DataClient for HttpDataClient {
    async fn get_group_members(&self, group_id: Uuid) -> anyhow::Result<Vec<Uuid>> {
        let url = format!(
            "{}/api/v1/groups/{}/resolved-members",
            self.base_url, group_id
        );

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Data service returned non-success status"));
        }

        let staff: Vec<StaffResponse> = response.json().await?;

        Ok(staff.into_iter().map(|s| s.id).collect())
    }
}
