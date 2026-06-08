use anyhow::Result;
use async_trait::async_trait;

pub mod runpod;
pub mod vast;

#[async_trait]
pub trait GpuProvider {
    fn name(&self) -> &str;

    async fn list_offers(&self, criteria: &SearchCriteria) -> Result<Vec<NormalizedOffer>>;

    async fn create_instance(&self, offer: &NormalizedOffer, spec: &InstanceSpec)
    -> Result<String>;

    async fn get_instance(&self, id: &str) -> Result<Option<NormalizedInstance>>;

    async fn destroy_instance(&self, id: &str) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct SearchCriteria {
    pub min_vram_gb: u32,
    pub max_price_per_hour: f64,
    pub num_gpus: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedOffer {
    pub provider: String,
    pub provider_ref: String,
    pub gpu_name: String,
    pub num_gpus: u32,
    pub vram_gb: u32,
    pub price_per_hour: f64,
    pub location: Option<String>,
    pub reliability: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct InstanceSpec {
    pub image: String,
    pub disk_gb: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstanceStatus {
    Pending,
    Running,
    Stopped,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct NormalizedInstance {
    pub id: String,
    pub provider: String,
    pub gpu_name: Option<String>,
    pub price_per_hour: f64,
    pub status: InstanceStatus,
    pub public_ip: Option<String>,
    pub ssh_host: Option<String>,
    pub ssh_port: Option<u16>,
}
