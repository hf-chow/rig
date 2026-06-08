use super::*;
use async_trait::async_trait;
use serde::Deserialize;

struct RunPodClient {
    api_key: String,
    client: reqwest::Client,
}

impl RunPodClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            api_key: api_key,
            client: client,
        }
    }
}
