use std::io::{Cursor, Read};

use async_trait::async_trait;
use reqwest::{Client, header};
use serde_json::Value;
use thiserror::Error;
use tracing::warn;
use zip::ZipArchive;

use nai_core::nai::NaiApi;

#[derive(Debug, Error)]
pub enum NaiError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unexpected response status {status}: {body}")]
    BadStatus { status: u16, body: String },
    #[error("missing file in zip: {0}")]
    MissingZipEntry(String),
}

#[derive(Clone)]
pub struct NaiClient {
    client: Client,
    token: String,
}

impl NaiClient {
    pub fn new(token: String, proxy: Option<String>) -> Result<Self, NaiError> {
        let token = token.trim().trim_matches('"');
        let token = token
            .strip_prefix("Bearer ")
            .or_else(|| token.strip_prefix("bearer "))
            .unwrap_or(token)
            .to_string();

        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::ORIGIN,
            header::HeaderValue::from_static("https://novelai.net"),
        );
        headers.insert(
            header::REFERER,
            header::HeaderValue::from_static("https://novelai.net/"),
        );

        let mut builder = Client::builder().default_headers(headers);
        if let Some(proxy) = proxy {
            builder = builder.proxy(reqwest::Proxy::all(proxy)?);
        }
        let client = builder.build()?;
        Ok(Self { client, token })
    }

    async fn post_zip(&self, url: &str, payload: &Value) -> Result<Vec<u8>, NaiError> {
        let rep = self
            .client
            .post(url)
            .bearer_auth(&self.token)
            .json(payload)
            .send()
            .await?;

        let status = rep.status();
        let body = rep.bytes().await?;
        if !status.is_success() {
            return Err(NaiError::BadStatus {
                status: status.as_u16(),
                body: String::from_utf8_lossy(&body).to_string(),
            });
        }
        Ok(body.to_vec())
    }

    fn zip_read_file_impl(zip_bytes: &[u8], name: &str) -> Result<Vec<u8>, NaiError> {
        let mut archive = ZipArchive::new(Cursor::new(zip_bytes))?;
        let mut file = archive
            .by_name(name)
            .map_err(|_| NaiError::MissingZipEntry(name.to_string()))?;
        let mut buf = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

#[async_trait]
impl NaiApi for NaiClient {
    async fn generate_image_zip(&self, payload: &Value) -> anyhow::Result<Vec<u8>> {
        Ok(self
            .post_zip("https://image.novelai.net/ai/generate-image", payload)
            .await?)
    }

    async fn augment_image_zip(&self, payload: &Value) -> anyhow::Result<Vec<u8>> {
        Ok(self
            .post_zip("https://image.novelai.net/ai/augment-image", payload)
            .await?)
    }

    fn zip_read_file(&self, zip_bytes: &[u8], name: &str) -> anyhow::Result<Vec<u8>> {
        Ok(Self::zip_read_file_impl(zip_bytes, name)?)
    }

    async fn inquire_anlas(&self) -> anyhow::Result<i64> {
        let rep = self
            .client
            .get("https://api.novelai.net/user/subscription")
            .bearer_auth(&self.token)
            .send()
            .await?;

        let status = rep.status();
        let body = rep.bytes().await?;
        if !status.is_success() {
            return Err(NaiError::BadStatus {
                status: status.as_u16(),
                body: String::from_utf8_lossy(&body).to_string(),
            }
            .into());
        }

        let v: serde_json::Value =
            serde_json::from_slice(&body).map_err(|e| NaiError::BadStatus {
                status: 200,
                body: format!("failed to parse json: {e}"),
            })?;

        v["trainingStepsLeft"]["fixedTrainingStepsLeft"]
            .as_i64()
            .ok_or_else(|| {
                warn!(
                    "missing trainingStepsLeft.fixedTrainingStepsLeft in response: {}",
                    v
                );
                NaiError::BadStatus {
                    status: 200,
                    body: "missing trainingStepsLeft.fixedTrainingStepsLeft".to_string(),
                }
                .into()
            })
    }
}
