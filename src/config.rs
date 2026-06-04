use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub image: Option<String>,
    pub disk_gb: Option<f64>,
    pub max_price_per_hour: Option<f64>,
    pub min_vram_gb: Option<u32>,
    pub providers: Providers,
}

#[derive(Deserialize, Serialize)]
pub struct Providers {
    pub vastai: Option<VastConfig>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct VastConfig {
    pub api_key: Option<String>,
    pub ssh_key_id: Option<Vec<u64>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            image: Some("pytorch/pytorch:2.3.0-cuda12.1-cudnn8-runtime".to_string()),
            disk_gb: Some(10.0),
            max_price_per_hour: Some(0.05),
            min_vram_gb: Some(8),
            providers: Providers {
                vastai: Some(VastConfig {
                    api_key: None,
                    ssh_key_id: None,
                }),
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read config at {}", path.display()))?;

        toml::from_str(&contents)
            .with_context(|| format!("failed to parse config at {}", path.display()))
    }
}

impl VastConfig {
    pub fn resolve_token(&self) -> Result<String> {
        if let Ok(token) = std::env::var("VASTAI_API_KEY") {
            return Ok(token);
        }
        self.api_key.clone().ok_or_else(|| anyhow::anyhow!("no Vast.ai API key found. Set VASTAI_API_KEY or add token to ~/.config/rig/config.toml"))
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rig")
        .join("config.toml")
}
