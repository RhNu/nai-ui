use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BaseGenerateRequest {
    pub model: String,
    pub positive: String,
    pub negative: String,
    /// Number of images to generate for a single submission.
    /// If omitted, defaults to 1.
    pub quantity: Option<u32>,
    pub width: u32,
    pub height: u32,
    pub steps: u32,
    pub scale: f32,
    pub sampler: String,
    pub noise_schedule: Option<String>,
    pub cfg_rescale: Option<f32>,
    pub seed: i64, // -1 => random
    pub add_quality_tags: Option<bool>,
    pub undesired_content_preset: Option<String>,
    pub sm: Option<bool>,
    pub sm_dyn: Option<bool>,
    pub use_coords: Option<bool>,
    pub legacy_uc: Option<bool>,
    pub character_prompts: Option<Vec<CharacterPrompt>>,
    pub reference_image_multiple: Option<Vec<String>>,
    pub reference_information_extracted_multiple: Option<Vec<i32>>,
    pub reference_strength_multiple: Option<Vec<f32>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CharacterPrompt {
    pub prompt: String,
    pub uc: String,
    pub center: Center,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Center {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Img2ImgRequest {
    #[serde(flatten)]
    pub base: BaseGenerateRequest,
    pub image_base64: String,
    pub strength: f32,
    pub noise: f32,
    pub extra_noise_seed: Option<i64>,
    pub color_correct: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InpaintRequest {
    #[serde(flatten)]
    pub base: BaseGenerateRequest,
    pub image_base64: String,
    pub mask_base64: String,
    pub strength: f32,
    pub noise: f32,
    pub extra_noise_seed: Option<i64>,
    pub color_correct: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CharacterRequest {
    #[serde(flatten)]
    pub base: BaseGenerateRequest,
    pub character_reference_image_base64: String,
    pub style_aware: bool,
    pub fidelity: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct GenerateResponse {
    pub seed: u64,
    pub output_path: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct JobSubmitResponse {
    pub job_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct DirectorRequest {
    pub width: u32,
    pub height: u32,
    pub image_base64: String,
}

#[derive(Debug, Deserialize)]
pub struct DirectorPromptRequest {
    #[serde(flatten)]
    pub base: DirectorRequest,
    pub prompt: String,
    pub defry: i32,
}

#[derive(Debug, Serialize)]
pub struct DirectorResponse {
    pub output_paths: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct OutputItem {
    /// Relative path under outputs root (uses '/').
    pub path: String,
    /// Category key for UI grouping. For director: "director/<type>".
    pub op_type: String,
    /// YYYY-MM-DD parsed from path if possible.
    pub date: String,
    /// File name (including extension).
    pub filename: String,
}

#[derive(Debug, Serialize)]
pub struct OutputsListResponse {
    pub items: Vec<OutputItem>,
}

#[derive(Debug, Deserialize)]
pub struct OutputsDeleteRequest {
    pub items: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct OutputsDeleteResponse {
    pub deleted: usize,
}

/// Internal: used for job payload storage.
pub type RawJson = Value;
