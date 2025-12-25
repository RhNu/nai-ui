use rand::Rng;
use serde_json::{Value, json};

use crate::{
    config::AppConfig,
    dto::{
        BaseGenerateRequest, CharacterRequest, DirectorResponse, GenerateResponse, Img2ImgRequest,
        InpaintRequest,
    },
    nai::NaiApi,
    outputs::OutputStore,
    prompt,
};

fn output_url(rel_path: &str) -> String {
    // Browser-friendly URL; front-end should prefix with backend base url.
    format!("/outputs/{}", rel_path.replace('\\', "/"))
}

fn normalize_seed(seed: i64) -> u64 {
    if seed == -1 {
        let mut rng = rand::rng();
        rng.random_range(1_000_000_000u64..=9_999_999_999u64)
    } else {
        seed.max(0) as u64
    }
}

fn uc_preset_id(model: &str, preset: &str) -> i32 {
    match model {
        "nai-diffusion-4-5-full" => match preset {
            "Heavy" => 0,
            "Light" => 1,
            "Furry Focus" => 2,
            "Human Focus" => 3,
            "None" => 4,
            _ => 4,
        },
        "nai-diffusion-3" | "nai-diffusion-4-5-curated" => match preset {
            "Heavy" => 0,
            "Light" => 1,
            "Human Focus" => 2,
            "None" => 3,
            _ => 3,
        },
        "nai-diffusion-furry-3" | "nai-diffusion-4-curated-preview" | "nai-diffusion-4-full" => {
            match preset {
                "Heavy" => 0,
                "Light" => 1,
                "None" => 2,
                _ => 2,
            }
        }
        _ => 0,
    }
}

fn quality_tags(model: &str) -> &'static str {
    match model {
        "nai-diffusion-4-5-full" => ", very aesthetic, masterpiece, no text",
        "nai-diffusion-4-5-curated" => {
            ", very aesthetic, masterpiece, no text, -0.8::feet::, rating:general"
        }
        "nai-diffusion-4-full" => ", no text, best quality, very aesthetic, absurdres",
        "nai-diffusion-4-curated-preview" => {
            ", rating:general, best quality, very aesthetic, absurdres"
        }
        "nai-diffusion-3" => ", best quality, amazing quality, very aesthetic, absurdres",
        "nai-diffusion-furry-3" => ", {best quality}, {amazing quality}",
        _ => "",
    }
}

fn skip_cfg_above_sigma(model: &str) -> f64 {
    match model {
        "nai-diffusion-4-5-full" => 58.0,
        "nai-diffusion-4-5-curated" => 36.158_893_609_242_725,
        "nai-diffusion-4-full" => 19.0,
        "nai-diffusion-3" => 19.343_056_794_463_642,
        "nai-diffusion-furry-3" | "nai-diffusion-4-curated-preview" => 11.845_154_803_027_79,
        _ => 0.0,
    }
}

async fn preprocess_prompts(
    cfg: &AppConfig,
    _outputs: &OutputStore,
    positive: &str,
    negative: &str,
) -> anyhow::Result<(String, String)> {
    let pos = prompt::format_str(cfg, positive);
    let neg = prompt::format_str(cfg, negative);
    Ok((pos, neg))
}

fn build_text2image_payload(
    req: &BaseGenerateRequest,
    seed: u64,
    positive: &str,
    negative: &str,
    uc_preset: i32,
    cfg_rescale: f32,
    use_coords: bool,
    legacy_uc: bool,
) -> Value {
    let model = req.model.as_str();
    let mut v = json!({
        "input": positive,
        "model": model,
        "action": "generate",
        "parameters": {
            "params_version": 3,
            "width": req.width,
            "height": req.height,
            "scale": req.scale,
            "sampler": req.sampler,
            "steps": req.steps,
            "n_samples": 1,
            "ucPreset": uc_preset,
            "qualityToggle": req.add_quality_tags.unwrap_or(false),
            "autoSmea": false,
            "dynamic_thresholding": false,
            "controlnet_strength": 1,
            "legacy": false,
            "add_original_image": true,
            "cfg_rescale": cfg_rescale,
            "legacy_v3_extend": false,
            "skip_cfg_above_sigma": skip_cfg_above_sigma(model),
            "seed": seed,
            "negative_prompt": negative,
        },
        "use_new_shared_trial": true,
    });

    // noise_schedule rules
    if let Some(ns) = &req.noise_schedule {
        if (model == "nai-diffusion-3" || model == "nai-diffusion-furry-3")
            && req.sampler == "ddim_v3"
        {
            // omit
        } else {
            v["parameters"]["noise_schedule"] = json!(ns);
        }
    }

    // v3/furry3 have sm/sm_dyn and no v4_prompt
    if model == "nai-diffusion-3" || model == "nai-diffusion-furry-3" {
        v["parameters"]["sm"] = json!(req.sm.unwrap_or(false));
        v["parameters"]["sm_dyn"] = json!(req.sm_dyn.unwrap_or(false));
        v["parameters"]["characterPrompts"] = json!([]);
    } else {
        let enabled_char_prompts = req
            .character_prompts
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter(|c| c.enabled)
            .collect::<Vec<_>>();

        let v4_char_positive = enabled_char_prompts
            .iter()
            .map(|c| {
                json!({
                    "char_caption": c.prompt,
                    "centers": [{"x": c.center.x, "y": c.center.y}]
                })
            })
            .collect::<Vec<_>>();
        let v4_char_negative = enabled_char_prompts
            .iter()
            .map(|c| {
                json!({
                    "char_caption": c.uc,
                    "centers": [{"x": c.center.x, "y": c.center.y}]
                })
            })
            .collect::<Vec<_>>();

        v["parameters"]["use_coords"] = json!(use_coords);
        v["parameters"]["legacy_uc"] = json!(legacy_uc);
        v["parameters"]["normalize_reference_strength_multiple"] = json!(null);
        v["parameters"]["inpaintImg2ImgStrength"] = json!(1);
        v["parameters"]["characterPrompts"] = json!(enabled_char_prompts);
        v["parameters"]["v4_prompt"] = json!({
            "caption": {"base_caption": positive, "char_captions": v4_char_positive},
            "use_coords": use_coords,
            "use_order": true
        });
        v["parameters"]["v4_negative_prompt"] = json!({
            "caption": {"base_caption": negative, "char_captions": v4_char_negative},
            "legacy_uc": legacy_uc
        });
        v["parameters"]["stream"] = json!("msgpack");
    }

    if req.sampler == "k_euler_ancestral" {
        v["parameters"]["deliberate_euler_ancestral_bug"] = json!(false);
        v["parameters"]["prefer_brownian"] = json!(true);
    }

    // vibe transfer pass-through
    if let Some(imgs) = &req.reference_image_multiple {
        v["parameters"]["reference_image_multiple"] = json!(imgs);
    }
    if let Some(strengths) = &req.reference_strength_multiple {
        v["parameters"]["reference_strength_multiple"] = json!(strengths);
    }
    if let Some(info) = &req.reference_information_extracted_multiple {
        v["parameters"]["reference_information_extracted_multiple"] = json!(info);
    }

    v
}

fn apply_img2img(
    json_data: &mut Value,
    strength: f32,
    noise: f32,
    image_base64: &str,
    extra_noise_seed: u64,
    color_correct: bool,
) {
    json_data["action"] = json!("img2img");
    json_data["parameters"]["color_correct"] = json!(color_correct);
    json_data["parameters"]["strength"] = json!(strength);
    json_data["parameters"]["noise"] = json!(noise);
    json_data["parameters"]["image"] = json!(image_base64);
    json_data["parameters"]["extra_noise_seed"] = json!(extra_noise_seed);
}

fn apply_inpaint(
    json_data: &mut Value,
    model: &str,
    strength: f32,
    noise: f32,
    image_base64: &str,
    mask_base64: &str,
    extra_noise_seed: u64,
    color_correct: bool,
) {
    apply_img2img(
        json_data,
        strength,
        noise,
        image_base64,
        extra_noise_seed,
        color_correct,
    );

    let inpaint_model = match model {
        "nai-diffusion-4-5-full" => "nai-diffusion-4-5-full-inpainting",
        "nai-diffusion-4-5-curated" => "nai-diffusion-4-5-curated-inpainting",
        "nai-diffusion-4-full" => "nai-diffusion-4-full-inpainting",
        "nai-diffusion-4-curated-preview" => "nai-diffusion-4-curated-inpainting",
        "nai-diffusion-3" => "nai-diffusion-3-inpainting",
        "nai-diffusion-furry-3" => "nai-diffusion-furry-3-inpainting",
        _ => model,
    };
    json_data["model"] = json!(inpaint_model);
    json_data["action"] = json!("infill");
    json_data["parameters"]["mask"] = json!(mask_base64);
    json_data["parameters"]["add_original_image"] = json!(false);
}

fn apply_character_reference(
    json_data: &mut Value,
    character_reference_image_base64: &str,
    style_aware: bool,
    fidelity: f32,
) {
    json_data["parameters"]["director_reference_images"] =
        json!([character_reference_image_base64]);
    json_data["parameters"]["director_reference_descriptions"] = json!([
        {
            "caption": {
                "base_caption": if style_aware {"character&style"} else {"character"},
                "char_captions": []
            },
            "legacy_uc": false
        }
    ]);
    json_data["parameters"]["director_reference_information_extracted"] = json!([1]);
    json_data["parameters"]["director_reference_strength_values"] = json!([1]);
    json_data["parameters"]["director_reference_secondary_strength_values"] =
        json!([1.0 - fidelity]);
}

pub async fn generate_t2i(
    cfg: &AppConfig,
    outputs: &OutputStore,
    nai: &dyn NaiApi,
    req: BaseGenerateRequest,
) -> anyhow::Result<GenerateResponse> {
    let seed = normalize_seed(req.seed);
    let add_quality_tags = req.add_quality_tags.unwrap_or(false);
    let (pos, neg) = preprocess_prompts(cfg, outputs, &req.positive, &req.negative).await?;
    let pos = if add_quality_tags {
        format!("{}{}", pos, quality_tags(&req.model))
    } else {
        pos
    };

    let uc_preset = uc_preset_id(
        &req.model,
        req.undesired_content_preset.as_deref().unwrap_or("None"),
    );
    let cfg_rescale = req.cfg_rescale.unwrap_or(0.0);
    let use_coords = req.use_coords.unwrap_or(true);
    let legacy_uc = req.legacy_uc.unwrap_or(false);

    let json_data = build_text2image_payload(
        &req,
        seed,
        &pos,
        &neg,
        uc_preset,
        cfg_rescale,
        use_coords,
        legacy_uc,
    );
    let zip_bytes = nai.generate_image_zip(&json_data).await?;
    let png = nai.zip_read_file(&zip_bytes, "image_0.png")?;
    let output_path = outputs.save_png("text2image", seed, &png).await?;
    let url = output_url(&output_path);
    Ok(GenerateResponse {
        seed,
        output_path,
        url,
    })
}

pub async fn generate_i2i(
    cfg: &AppConfig,
    outputs: &OutputStore,
    nai: &dyn NaiApi,
    req: Img2ImgRequest,
) -> anyhow::Result<GenerateResponse> {
    let seed = normalize_seed(req.base.seed);
    let (pos, neg) =
        preprocess_prompts(cfg, outputs, &req.base.positive, &req.base.negative).await?;
    let uc_preset = uc_preset_id(
        &req.base.model,
        req.base
            .undesired_content_preset
            .as_deref()
            .unwrap_or("None"),
    );
    let cfg_rescale = req.base.cfg_rescale.unwrap_or(0.0);
    let use_coords = req.base.use_coords.unwrap_or(true);
    let legacy_uc = req.base.legacy_uc.unwrap_or(false);

    let mut json_data = build_text2image_payload(
        &req.base,
        seed,
        &pos,
        &neg,
        uc_preset,
        cfg_rescale,
        use_coords,
        legacy_uc,
    );
    apply_img2img(
        &mut json_data,
        req.strength,
        req.noise,
        &req.image_base64,
        req.extra_noise_seed.unwrap_or(seed as i64) as u64,
        req.color_correct.unwrap_or(false),
    );

    let zip_bytes = nai.generate_image_zip(&json_data).await?;
    let png = nai.zip_read_file(&zip_bytes, "image_0.png")?;
    let output_path = outputs.save_png("image2image", seed, &png).await?;
    let url = output_url(&output_path);
    Ok(GenerateResponse {
        seed,
        output_path,
        url,
    })
}

pub async fn generate_inpaint(
    cfg: &AppConfig,
    outputs: &OutputStore,
    nai: &dyn NaiApi,
    req: InpaintRequest,
) -> anyhow::Result<GenerateResponse> {
    let seed = normalize_seed(req.base.seed);
    let (pos, neg) =
        preprocess_prompts(cfg, outputs, &req.base.positive, &req.base.negative).await?;
    let uc_preset = uc_preset_id(
        &req.base.model,
        req.base
            .undesired_content_preset
            .as_deref()
            .unwrap_or("None"),
    );
    let cfg_rescale = req.base.cfg_rescale.unwrap_or(0.0);
    let use_coords = req.base.use_coords.unwrap_or(true);
    let legacy_uc = req.base.legacy_uc.unwrap_or(false);

    let mut json_data = build_text2image_payload(
        &req.base,
        seed,
        &pos,
        &neg,
        uc_preset,
        cfg_rescale,
        use_coords,
        legacy_uc,
    );
    apply_inpaint(
        &mut json_data,
        &req.base.model,
        req.strength,
        req.noise,
        &req.image_base64,
        &req.mask_base64,
        req.extra_noise_seed.unwrap_or(seed as i64) as u64,
        req.color_correct.unwrap_or(false),
    );

    let zip_bytes = nai.generate_image_zip(&json_data).await?;
    let png = nai.zip_read_file(&zip_bytes, "image_0.png")?;
    let output_path = outputs.save_png("inpaint", seed, &png).await?;
    let url = output_url(&output_path);
    Ok(GenerateResponse {
        seed,
        output_path,
        url,
    })
}

pub async fn generate_character(
    cfg: &AppConfig,
    outputs: &OutputStore,
    nai: &dyn NaiApi,
    req: CharacterRequest,
) -> anyhow::Result<GenerateResponse> {
    let seed = normalize_seed(req.base.seed);
    let (pos, neg) =
        preprocess_prompts(cfg, outputs, &req.base.positive, &req.base.negative).await?;
    let uc_preset = uc_preset_id(
        &req.base.model,
        req.base
            .undesired_content_preset
            .as_deref()
            .unwrap_or("None"),
    );
    let cfg_rescale = req.base.cfg_rescale.unwrap_or(0.0);
    let use_coords = req.base.use_coords.unwrap_or(true);
    let legacy_uc = req.base.legacy_uc.unwrap_or(false);

    if req.base.model != "nai-diffusion-4-5-full" && req.base.model != "nai-diffusion-4-5-curated" {
        anyhow::bail!("character currently supported only for nai-diffusion-4-5-full/curated");
    }

    let mut json_data = build_text2image_payload(
        &req.base,
        seed,
        &pos,
        &neg,
        uc_preset,
        cfg_rescale,
        use_coords,
        legacy_uc,
    );
    apply_character_reference(
        &mut json_data,
        &req.character_reference_image_base64,
        req.style_aware,
        req.fidelity,
    );

    let zip_bytes = nai.generate_image_zip(&json_data).await?;
    let png = nai.zip_read_file(&zip_bytes, "image_0.png")?;
    let output_path = outputs.save_png("character", seed, &png).await?;
    let url = output_url(&output_path);
    Ok(GenerateResponse {
        seed,
        output_path,
        url,
    })
}

pub async fn director_call(
    outputs: &OutputStore,
    nai: &dyn NaiApi,
    payload: Value,
    is_bg_removal: bool,
) -> anyhow::Result<DirectorResponse> {
    let zip_bytes = nai.augment_image_zip(&payload).await?;

    let mut paths = Vec::new();
    if is_bg_removal {
        for (idx, name) in ["image_0.png", "image_1.png", "image_2.png"]
            .into_iter()
            .enumerate()
        {
            if let Ok(png) = nai.zip_read_file(&zip_bytes, name) {
                let path = outputs
                    .save_png(
                        &format!("director/remove_bg/{idx}"),
                        rand::random::<u64>(),
                        &png,
                    )
                    .await?;
                paths.push(path);
            }
        }
    } else {
        let png = nai.zip_read_file(&zip_bytes, "image_0.png")?;
        let path = outputs
            .save_png("director", rand::random::<u64>(), &png)
            .await?;
        paths.push(path);
    }

    Ok(DirectorResponse {
        output_paths: paths,
    })
}
