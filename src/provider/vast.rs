use super::*;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Offer {
    pub id: u64,
    pub gpu_name: Option<String>,
    pub num_gpus: u32,
    pub gpu_ram: u64,
    pub dph_total: f64,
    pub geolocation: String,
    pub reliability: f64,
    pub rentable: bool,
    pub disk_space: f64,
    pub public_ipaddr: Option<String>,
    pub search: OfferPricing,
}

#[derive(Deserialize)]
pub struct OfferResponse {
    pub offers: Vec<Offer>,
}

#[derive(Deserialize)]
pub struct OfferPricing {
    #[serde(rename = "totalHour")]
    pub total_hour: f64,
}

#[derive(Deserialize)]
pub struct Instance {
    pub id: u64,
    pub gpu_name: Option<String>,
    pub dph_total: f64,
    pub public_ipaddr: Option<String>,
    pub actual_status: Option<String>,
    pub start_date: Option<f64>,
    pub ssh_host: Option<String>,
    pub ssh_port: Option<u16>,
    pub search: Option<OfferPricing>,
}

#[derive(Deserialize)]
pub struct InstanceResponse {
    instances: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct CreateInstanceResponse {
    pub success: Option<bool>,
    pub new_contract: Option<u64>,
    pub error: Option<String>,
    pub msg: Option<String>,
}

pub struct VastClient {
    api_key: String,
    client: reqwest::Client,
    ssh_key_ids: Vec<u64>,
}

impl VastClient {
    pub fn new(api_key: String, ssh_key_ids: Vec<u64>) -> Self {
        let client = reqwest::Client::new();
        Self {
            api_key,
            client,
            ssh_key_ids,
        }
    }
}

fn map_offer(o: Offer) -> NormalizedOffer {
    NormalizedOffer {
        provider: "vastai".to_string(),
        provider_ref: o.id.to_string(),
        gpu_name: o.gpu_name.unwrap_or("unknown".to_string()),
        num_gpus: o.num_gpus,
        vram_gb: (o.gpu_ram / 1024) as u32,
        price_per_hour: o.search.total_hour,
        location: Some(o.geolocation),
        reliability: Some(o.reliability),
    }
}

fn map_instance(i: vast::Instance) -> NormalizedInstance {
    NormalizedInstance {
        id: i.id.to_string(),
        provider: "vastai".to_string(),
        gpu_name: i.gpu_name,
        price_per_hour: i.search.map(|s| s.total_hour).unwrap_or(i.dph_total),
        status: map_status(i.actual_status),
        public_ip: i.public_ipaddr,
        ssh_host: i.ssh_host,
        ssh_port: i.ssh_port,
    }
}

fn map_status(s: Option<String>) -> InstanceStatus {
    match s.as_deref() {
        Some("running") => InstanceStatus::Running,
        Some("loading" | "created") => InstanceStatus::Pending,
        Some(other) => InstanceStatus::Unknown(other.to_string()),
        None => InstanceStatus::Unknown("none".to_string()),
    }
}

#[async_trait]
impl GpuProvider for VastClient {
    fn name(&self) -> &str {
        "vastai"
    }

    async fn list_offers(&self, criteria: &SearchCriteria) -> Result<Vec<NormalizedOffer>> {
        let query = serde_json::json!({
            "gpu_ram":{"gte": criteria.min_vram_gb * 1024},
            "dph_total":{"lte": criteria.max_price_per_hour},
            "rentable": {"eq": true},
            "num_gpus":{"eq": criteria.num_gpus},
        });
        let resp = self
            .client
            .get("https://console.vast.ai/api/v0/bundles/")
            .query(&[("api_key", &self.api_key), ("q", &query.to_string())])
            .send()
            .await?;

        let offer_resp: OfferResponse = resp.json().await?;
        let mut offers = offer_resp.offers;
        offers.sort_by(|a, b| {
            a.dph_total
                .partial_cmp(&b.dph_total)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(offers.into_iter().map(map_offer).collect())
    }

    async fn get_instance(&self, id: &str) -> Result<Option<NormalizedInstance>> {
        let id: u64 = id.parse()?;
        let resp = self
            .client
            .get(format!("https://console.vast.ai/api/v0/instances/{}/", id))
            .query(&[("api_key", &self.api_key), ("owner", &"me".to_string())])
            .send()
            .await?;

        let text = resp.text().await?;
        let instance_resp: InstanceResponse = serde_json::from_str(&text)?;
        let instances: Vec<Instance> = match instance_resp.instances {
            None => vec![],
            Some(serde_json::Value::Array(arr)) => {
                serde_json::from_value(serde_json::Value::Array(arr))?
            }
            Some(obj @ serde_json::Value::Object(_)) => {
                vec![serde_json::from_value(obj)?]
            }
            _ => vec![],
        };

        Ok(instances.into_iter().find(|i| i.id == id).map(map_instance))
    }

    async fn create_instance(
        &self,
        offer: &NormalizedOffer,
        spec: &InstanceSpec,
    ) -> Result<String> {
        let offer_id: u64 = offer.provider_ref.parse()?;
        let body = serde_json::json!({
            "client_id": "me",
            "image": spec.image,
            "disk": spec.disk_gb,
            "id": offer_id,
            "ssh_key_ids": self.ssh_key_ids,
        });
        let resp = self
            .client
            .put(format!("https://console.vast.ai/api/v0/asks/{}/", offer_id))
            .query(&[("api_key", &self.api_key)])
            .json(&body)
            .send()
            .await?;
        let create_resp: CreateInstanceResponse = resp.json().await?;
        if create_resp.success != Some(true) {
            anyhow::bail!(
                "failed to create instance: {}",
                create_resp.msg.unwrap_or_default()
            )
        }
        let new_contract = create_resp
            .new_contract
            .ok_or_else(|| anyhow::anyhow!("no contract id in response"))?;
        Ok(new_contract.to_string())
    }

    async fn destroy_instance(&self, id: &str) -> Result<()> {
        let id: u64 = id.parse()?;
        let resp = self
            .client
            .delete(format!("https://console.vast.ai/api/v0/instances/{}/", id))
            .query(&[("api_key", &self.api_key)])
            .send()
            .await?;
        let delete_resp: CreateInstanceResponse = resp.json().await?;
        if delete_resp.success != Some(true) {
            if delete_resp.error.as_deref() != Some("no_such_instance") {
                anyhow::bail!(
                    "failed to destroy instance: {}",
                    delete_resp.msg.unwrap_or_default()
                )
            }
        }
        Ok(())
    }
}
