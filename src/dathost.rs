use std::{env, time::Duration};

use bytes::Bytes;
use reqwest::{Client, Result};

use crate::models::{DatHostServer, ServerId};

#[derive(Clone)]
pub struct DathostClient(Client);

impl DathostClient {
    pub fn new() -> Result<Self> {
        let mut headers = http::HeaderMap::with_capacity(1);
        headers.insert(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_str(&{
                let username = env::var("DATHOST_USER").expect("DATHOST_USER must be set");
                let password = env::var("DATHOST_PASSWORD").ok();
                format!(
                    "Basic {}",
                    base64::encode(format!(
                        "{username}:{password}",
                        password = password.as_deref().unwrap_or("")
                    ))
                )
            })
            .unwrap(),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(60 * 10))
            .build()?;
        Ok(Self(client))
    }

    pub async fn get_server_info(&self, server_id: &ServerId) -> Result<DatHostServer> {
        self.0
            .get(&format!(
                "https://dathost.net/api/0.1/game-servers/{server_id}"
            ))
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_server_map(&self, server_id: &ServerId) -> Result<String> {
        Ok(self
            .get_server_info(server_id)
            .await?
            .csgo_settings
            .mapgroup_start_map)
    }

    pub async fn get_file(&self, server_id: &ServerId, path: &str) -> Result<Bytes> {
        self.0
            .get(&format!(
                "https://dathost.net/api/0.1/game-servers/{server_id}/files/{path}"
            ))
            .send()
            .await?
            .bytes()
            .await
    }
}
