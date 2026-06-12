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

#[derive(Deserialize)]
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

#[async_trait]
impl GpuProvider for RunPodClient {
    fn name(&self) -> &str {
        "runpod"
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
                        provider_ref: g.id.clone(),
                        gpu_name: g.display_name.clone(),
                        num_gpus: 1,
                        vram_gb: g.memory_in_gb,
                        price_per_hour: g.lowest_price.as_ref()?.minimum_bid_price?,
                        location: None,
                        reliability: None,
                    })
                } else {
                    None
                }
            })
            .filter(|o| {
                o.vram_gb >= criteria.min_vram_gb && o.price_per_hour <= criteria.max_price_per_hour
            })
            .collect::<Vec<_>>();
        offers.sort_by(|a, b| {
            a.price_per_hour
                .partial_cmp(&b.price_per_hour)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(offers)
    }

    async fn create_instance(
        &self,
        offer: &NormalizedOffer,
        spec: &InstanceSpec,
    ) -> Result<String> {
        let mutation = r#"{
            "input": {
                gpuTypeId: offer.provider_ref,
                cloudType: "SECURE" or "COMMUNITY",
                gpuCount: 1,
                bidPerGpu: offer.price_per_hour,
                imageName: spec.image,
                containerDiskInGb: spec.disk_gb as u32,
            }
        }"#;
        let variables = json!({"input": {
            "gpuTypeId": offer.provider_ref,
            "cloudType": "COMMUNITY",
            "gpuCount": 1,
            "bidPerGpu": offer.price_per_hour,
            "imageName": spec.image,
            "containerDiskInGb": spec.disk_gb as i64,
            "ports": "22/tcp",
        }});
        let data = self.graphql(mutation, variables).await?;
        let id = data["podRentInterruptable"]["id"]
            .as_str()
            .ok_or_else(|| anyhow!("no pod id in response"))?;
        Ok(id.to_string())
    }
}
