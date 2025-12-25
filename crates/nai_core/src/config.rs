use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub token: String,
    pub proxy: Option<String>,
    pub bind: String,
    pub output_dir: PathBuf,
    pub custom_path_template: String,
    pub format_input: bool,
    /// Base cooldown seconds between generation calls (job pacing).
    /// 0 disables cooldown.
    pub cool_time: u64,
    /// Random jitter seconds added to cooldown.
    /// Effective sleep range is [abs(cool_time - cool_jitter), cool_time + cool_jitter].
    pub cool_jitter: f64,
    /// Optional directory to serve static frontend assets (index.html, etc.).
    pub static_dir: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing NovelAI token: set env var token/TOKEN in .env")]
    MissingToken,
    #[error("invalid port in env var port/PORT: {0}")]
    InvalidPort(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        // Try to load .env from common locations.
        // - Running from repo root: .env
        // - Running from rust/backend: ../.env or ../../.env
        let _ = dotenvy::from_path(".env");
        let _ = dotenvy::from_path("../.env");
        let _ = dotenvy::from_path("../../.env");

        let token = std::env::var("token")
            .or_else(|_| std::env::var("TOKEN"))
            .map_err(|_| ConfigError::MissingToken)?;

        let proxy = std::env::var("proxy")
            .or_else(|_| std::env::var("PROXY"))
            .ok();

        let port_str = std::env::var("port")
            .or_else(|_| std::env::var("PORT"))
            .unwrap_or_else(|_| "11451".to_string());
        let port: u16 = port_str
            .parse()
            .map_err(|_| ConfigError::InvalidPort(port_str.clone()))?;

        let bind = std::env::var("bind")
            .or_else(|_| std::env::var("BIND"))
            .unwrap_or_else(|_| format!("127.0.0.1:{port}"));

        let output_dir = std::env::var("output_dir")
            .or_else(|_| std::env::var("OUTPUT_DIR"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("outputs"));

        let custom_path_template = std::env::var("custom_path")
            .or_else(|_| std::env::var("CUSTOM_PATH"))
            // 文件名：编号在前，随机字符在后，方便排序。
            // 示例：text2image/2025-12-24/00001_a1b2c3_123456789.png
            .unwrap_or_else(|_| "<类型>/<日期>/<编号>_<随机字符>_<种子>".to_string());

        let format_input = std::env::var("format_input")
            .or_else(|_| std::env::var("FORMAT_INPUT"))
            .map(|v| matches!(v.as_str(), "1" | "true" | "True" | "TRUE"))
            .unwrap_or(true);

        let cool_time: u64 = std::env::var("cool_time")
            .or_else(|_| std::env::var("COOL_TIME"))
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);

        let cool_jitter: f64 = std::env::var("cool_jitter")
            .or_else(|_| std::env::var("COOL_JITTER"))
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0);

        let static_dir = std::env::var("static_dir")
            .or_else(|_| std::env::var("STATIC_DIR"))
            .ok()
            .map(PathBuf::from);

        Ok(Self {
            token,
            proxy,
            bind,
            output_dir,
            custom_path_template,
            format_input,
            cool_time,
            cool_jitter,
            static_dir,
        })
    }
}
