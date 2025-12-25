use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait NaiApi: Send + Sync {
    async fn generate_image_zip(&self, payload: &Value) -> anyhow::Result<Vec<u8>>;
    async fn augment_image_zip(&self, payload: &Value) -> anyhow::Result<Vec<u8>>;
    fn zip_read_file(&self, zip_bytes: &[u8], name: &str) -> anyhow::Result<Vec<u8>>;
    async fn inquire_anlas(&self) -> anyhow::Result<i64>;
}
