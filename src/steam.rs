use std::{env, time::Duration};

use crate::models::{DeleteGsltRequest, QueryLoginTokenRequest, SteamApiRootResponse};
use reqwest::{Client, Response, Result};

#[derive(Clone)]
pub struct SteamWebClient(Client);

impl SteamWebClient {
    pub fn new() -> Result<Self> {
        let mut headers = http::HeaderMap::with_capacity(1);
        headers.insert(
            "x-webapi-key",
            http::HeaderValue::from_str(
                env::var("STEAM_API_KEY")
                    .expect("STEAM_API_KEY must be set")
                    .as_str(),
            )
            .unwrap(),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(60))
            .build()?;
        Ok(Self(client))
    }
    pub async fn query_login_token(&self, gslt: String) -> Result<u64> {
        let json = serde_json::to_string(&QueryLoginTokenRequest { login_token: gslt }).unwrap();
        let resp = self
            .0
            .get("https://api.steampowered.com/IGameServersService/QueryLoginToken/v1/")
            .query(&[("input_json", &&json)])
            .header("Content-Length", 0)
            .send()
            .await?
            .json::<SteamApiRootResponse>()
            .await?;
        let steamid_str = resp.response.steamid.parse::<u64>().unwrap();
        Ok(steamid_str)
    }
    pub async fn delete_gslt(&self, steamid: u64) -> Result<Response> {
        let json = serde_json::to_string(&DeleteGsltRequest { steamid }).unwrap();
        self.0
            .post("https://api.steampowered.com/IGameServersService/DeleteAccount/v1/")
            .query(&[("input_json", &&json)])
            .header("Content-Length", 0)
            .send()
            .await
    }
}
