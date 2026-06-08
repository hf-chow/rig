use super::*;
use anyhow::{Ok, anyhow};
use async_trait::async_trait;
use serde::Deserialize;

struct RunPodClient {
    api_key: String,
    client: reqwest::Client,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GpuType {
    id: String,
    display_name: String,
    memory_in_gb: u32,
    lowest_price: Option<LowestPrice>,
}

struct GpuTypesData {
    gpu_types: Vec<GpuType>,
}

#[derive(Deserialize)]
struct LowestPrice {
    minimum_bid_price: Option<f64>,
}

impl RunPodClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            api_key: api_key,
            client: client,
        }
    }

    async fn graphql(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let resp = self
            .client
            .post("https://api.runpod.io/graphql")
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({"query": query, "variables": variables}))
            .send()
            .await?;
        let mut body: serde_json::Value = resp.json().await?;
        if !body["errors"].is_null() {
            anyhow::bail!("error found from response: {}", body["errors"])
        }
        Ok(body["data"].take())
    }
}
