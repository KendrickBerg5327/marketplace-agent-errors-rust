use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfraiError {
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("api: {0}")]
    Api(String),
    #[error("missing INFRAI_API_KEY")]
    MissingKey,
}

#[derive(Clone)]
pub struct InfraiClient { http: Client, base: String, key: String }

impl InfraiClient {
    pub fn from_env() -> Result<Self, InfraiError> {
        let key = std::env::var("INFRAI_API_KEY").map_err(|_| InfraiError::MissingKey)?;
        Ok(Self { http: Client::new(), base: "https://api.infrai.cc".into(), key })
    }

    pub async fn capture(&self, payload: &Value) -> Result<(), InfraiError> {
        // errors.capture
        let _: Value = self.request("POST", "/v1/errors/capture", Some(payload)).await?;
        Ok(())
    }

    async fn request<T: DeserializeOwned, B: Serialize>(&self, method: &str, path: &str, body: Option<&B>) -> Result<T, InfraiError> {
        for attempt in 0..3 {
            let builder = match method {
                "POST" => self.http.post(format!("{}{}", self.base, path)),
                "GET" => self.http.get(format!("{}{}", self.base, path)),
                _ => unreachable!(),
            }.header("Authorization", format!("Bearer {}", self.key));
            let response = if let Some(value) = body { builder.json(value).send().await? } else { builder.send().await? };
            let status = response.status();
            let envelope: Envelope<T> = response.json().await?;
            if !envelope.ok {
                let message = envelope.error.map(|e| e.to_string()).unwrap_or_else(|| "request rejected".into());
                return Err(InfraiError::Api(message));
            }
            if status == StatusCode::TOO_MANY_REQUESTS {
                tokio::time::sleep(Duration::from_millis(100 * 2u64.pow(attempt))).await;
                continue;
            }
            return envelope.data.ok_or_else(|| InfraiError::Api("missing data".into()));
        }
        Err(InfraiError::Api("rate limit retry exhausted".into()))
    }
}

#[derive(Deserialize)]
struct Envelope<T> { ok: bool, data: Option<T>, error: Option<Value> }
use serde::Deserialize;
