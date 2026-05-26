use anyhow::Result;
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
}

impl VastClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::new();
        Self { api_key, client }
    }

    pub async fn list_offers(&self, min_vram_gb: u32, max_price: f64) -> Result<Vec<Offer>> {
        let query = serde_json::json!({
            "gpu_ram":{"gte": min_vram_gb * 1024},
            "dph_total":{"lte": max_price},
            "rentable": {"eq": true},
            "num_gpus":{"eq":1}
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
        Ok(offers)
    }

    pub async fn get_instance(&self, id: u64) -> Result<Option<Instance>> {
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

        Ok(instances.into_iter().find(|i| i.id == id))
    }

    pub async fn create_instance(
        &self,
        offer_id: u64,
        image: &str,
        disk_gb: f64,
        ssh_key_ids: &[u64],
    ) -> Result<u64> {
        let body = serde_json::json!({
            "client_id": "me",
            "image": image,
            "disk": disk_gb,
            "id": offer_id,
            "ssh_key_ids": ssh_key_ids,
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
        Ok(new_contract)
    }

    pub async fn destroy_instance(&self, id: u64) -> Result<()> {
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
