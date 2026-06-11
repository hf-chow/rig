use super::*;
use anyhow::{Ok, anyhow};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

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

    async fn list_offers(&self, criteria: SearchCriteria) -> Result<Vec<NormalizedOffer>> {
        let query = r#"
        query{
            gpuTypes {
                id
                displayName
                memoryInGb
                lowestPrice(input: { gpuCount: 1}) { minimumBidPrice }
            }
        }"#;
        let data = self.graphql(query, json!({})).await?;
        let parsed: GpuTypesData = serde_json::from_value(data)?;

        let mut offers = parsed
            .gpu_types
            .iter()
            .filter_map(|g| {
                if !g.lowest_price.is_none() {
                    Some(NormalizedOffer {
                        provider: "runpod".to_string(),
                        gpu_name: g.display_name,
                        num_gpus: 1,
                        vram_gb: g.memory_in_gb,
                        price_per_hour: g.lowest_price.as_ref()?.minimum_bid_price?,
                        location: None,
                        provider_ref: g.id.clone(),
                        reliability: None,
                    })
                } else {
                    None
                }
            })
            .filter(|o| {
                if o.vram_gb >= criteria.min_vram_gb
                    && o.price_per_hour <= criteria.max_price_per_hour
                {
                    Some(o)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        offers.sort_by(|a, b| a.price_per_hour.cmp(&b.price_per_hour));
        Ok(offers)
    }
}
