use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use chrono::Local;
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::warn;

use crate::{config::AppConfig, dto::OutputItem, util};

#[derive(Debug, Error)]
pub enum OutputError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid output path")]
    InvalidPath,
}

#[derive(Clone)]
pub struct OutputStore {
    output_dir: PathBuf,
    template: String,
    counters_path: PathBuf,
    /// Per-leaf directory "next index" cursor.
    /// Key is a normalized relative dir path using '/'. Empty string means outputs root.
    counters: Arc<Mutex<BTreeMap<String, usize>>>,
}

impl OutputStore {
    pub fn new(cfg: &AppConfig) -> Result<Self, OutputError> {
        let output_dir = cfg.output_dir.clone();
        std::fs::create_dir_all(&output_dir)?;

        let counters_path = output_dir.join("output_counters.json");
        let mut counters: BTreeMap<String, usize> =
            load_counters_sync(&counters_path).unwrap_or_default();

        // 启动时扫描一下游标位置：根据已有文件名推断每个目录的 next index。
        // 仅支持新命名：00001_xxxxxx_seed.png（编号在前）
        let changed = scan_existing_outputs_sync(&output_dir, &mut counters)?;
        if changed {
            save_counters_sync(&counters_path, &counters)?;
        }

        Ok(Self {
            output_dir,
            template: cfg.custom_path_template.clone(),
            counters_path,
            counters: Arc::new(Mutex::new(counters)),
        })
    }

    pub fn root(&self) -> &Path {
        &self.output_dir
    }

    /// Save image and return a *relative path* under the outputs root.
    pub async fn save_png(
        &self,
        kind: &str,
        seed: u64,
        png_bytes: &[u8],
    ) -> Result<String, OutputError> {
        let date = Local::now().date_naive().to_string();
        let random = util::random_str(6);

        let rel_template = self
            .template
            .replace("<类型>", kind)
            .replace("<日期>", &date)
            .replace("<种子>", &seed.to_string())
            .replace("<随机字符>", &random);

        // We must not use raw "<编号>" in any on-disk directory name (Windows forbids '<' and '>').
        // Compute leaf directory using a safe placeholder index, then compute the real index.
        let mut rel_for_dir = rel_template.replace("<编号>", "00000");
        if !rel_for_dir.ends_with(".png") {
            rel_for_dir.push_str(".png");
        }
        let rel_for_dir = sanitize_rel_path(&rel_for_dir);

        let leaf_dir = Path::new(&rel_for_dir)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();

        let abs_leaf = self.output_dir.join(&leaf_dir);
        tokio::fs::create_dir_all(&abs_leaf).await?;

        // Use a persistent per-directory cursor so deletions won't cause index reuse.
        let leaf_key = normalize_rel_dir_key(&leaf_dir);
        let next_index = {
            let mut map = self.counters.lock().await;
            let next = map.get(&leaf_key).copied().unwrap_or(0);
            map.insert(leaf_key.clone(), next + 1);
            next
        };
        let idx = format!("{:0>5}", next_index);
        self.save_counters().await?;

        let mut rel = rel_template.replace("<编号>", &idx);
        if !rel.ends_with(".png") {
            rel.push_str(".png");
        }

        let rel_sanitized = sanitize_rel_path(&rel);
        if rel_sanitized != rel {
            warn!(
                original = %rel,
                sanitized = %rel_sanitized,
                template = %self.template,
                "output path contained invalid characters; sanitized"
            );
        }
        rel = rel_sanitized;

        let path = self.output_dir.join(&rel);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&path, png_bytes).await?;
        // Return relative path for browser usage.
        Ok(rel)
    }

    /// List png files as relative paths under outputs root.
    pub async fn list_pngs(&self, limit: usize) -> Result<Vec<String>, OutputError> {
        let mut out = Vec::new();
        if !self.output_dir.exists() {
            return Ok(out);
        }
        walk_pngs(&self.output_dir, &self.output_dir, &mut out, limit)?;
        Ok(out)
    }

    pub async fn list_items_paginated(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<OutputItem>, bool, usize), OutputError> {
        let root = self.output_dir.clone();
        let items = tokio::task::spawn_blocking(move || -> Result<Vec<OutputItem>, OutputError> {
            let mut rels = Vec::new();
            if !root.exists() {
                return Ok(Vec::new());
            }
            walk_pngs(&root, &root, &mut rels, usize::MAX)?;
            let mut items: Vec<OutputItem> =
                rels.into_iter().map(|p| output_item_from_rel(&p)).collect();

            items.sort_by(|a, b| {
                let a_idx = parse_output_index(&a.filename).unwrap_or(0);
                let b_idx = parse_output_index(&b.filename).unwrap_or(0);
                a.op_type
                    .cmp(&b.op_type)
                    .then_with(|| b.date.cmp(&a.date))
                    .then_with(|| b_idx.cmp(&a_idx))
                    .then_with(|| b.filename.cmp(&a.filename))
            });
            Ok(items)
        })
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))??;
        let total = items.len();
        let sliced: Vec<OutputItem> = items.into_iter().skip(offset).take(limit).collect();
        let next_offset = offset + sliced.len();
        let has_more = next_offset < total;
        Ok((sliced, has_more, next_offset))
    }

    pub async fn delete_rel_files(&self, rel_paths: &[String]) -> Result<usize, OutputError> {
        let mut deleted = 0usize;
        for rel in rel_paths {
            let rel_norm = normalize_rel_path(rel);
            if !is_safe_rel_path(&rel_norm) {
                continue;
            }
            let abs = self.output_dir.join(&rel_norm);
            match tokio::fs::remove_file(&abs).await {
                Ok(()) => {
                    deleted += 1;
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    // ignore
                }
                Err(e) => return Err(OutputError::Io(e)),
            }
            // Best-effort: clean empty parent dirs up to outputs root.
            let _ = cleanup_empty_parents(&self.output_dir, abs.parent()).await;
        }
        Ok(deleted)
    }

    async fn save_counters(&self) -> Result<(), OutputError> {
        let map = self.counters.lock().await;
        let bytes = serde_json::to_vec_pretty(&*map)?;
        tokio::fs::write(&self.counters_path, bytes).await?;
        Ok(())
    }
}

fn is_date_component(s: &str) -> bool {
    // YYYY-MM-DD
    if s.len() != 10 {
        return false;
    }
    let b = s.as_bytes();
    b[0..4].iter().all(|c| c.is_ascii_digit())
        && b[4] == b'-'
        && b[5..7].iter().all(|c| c.is_ascii_digit())
        && b[7] == b'-'
        && b[8..10].iter().all(|c| c.is_ascii_digit())
}

fn output_item_from_rel(rel: &str) -> OutputItem {
    let rel_norm = normalize_rel_path(rel);
    let parts: Vec<&str> = rel_norm.split('/').filter(|s| !s.is_empty()).collect();
    let filename = parts.last().copied().unwrap_or("").to_string();

    // Default: <op>/<date>/<file>
    let mut op_type = parts.get(0).copied().unwrap_or("").to_string();
    let mut date = parts.get(1).copied().unwrap_or("").to_string();

    // Director: prefer grouping as director/<type> when possible.
    if parts.first().copied() == Some("director") {
        // director/<date>/<file>
        if parts.get(1).copied().is_some_and(is_date_component) {
            op_type = "director".to_string();
            date = parts.get(1).copied().unwrap_or("").to_string();
        // director/<type>/<date>/<file>
        } else if parts.get(2).copied().is_some_and(is_date_component) {
            op_type = format!("director/{}", parts.get(1).copied().unwrap_or(""));
            date = parts.get(2).copied().unwrap_or("").to_string();
        // director/<type>/<idx>/<date>/<file>
        } else if parts.get(3).copied().is_some_and(is_date_component) {
            op_type = format!("director/{}", parts.get(1).copied().unwrap_or(""));
            date = parts.get(3).copied().unwrap_or("").to_string();
        }
    }

    OutputItem {
        path: rel_norm,
        op_type,
        date,
        filename,
    }
}

fn sanitize_component(mut s: String) -> String {
    // Strip ASCII control characters.
    s.retain(|c| !c.is_control());

    // Windows forbidden characters: <>:"/\\|?*
    const FORBIDDEN: [char; 9] = ['<', '>', ':', '"', '|', '?', '*', '\\', '/'];
    s = s
        .chars()
        .map(|c| if FORBIDDEN.contains(&c) { '_' } else { c })
        .collect();

    // Avoid trailing dots/spaces on Windows.
    while s.ends_with(' ') || s.ends_with('.') {
        s.pop();
    }

    if s.is_empty() {
        return "_".to_string();
    }

    // Avoid reserved device names on Windows.
    let upper = s.to_ascii_uppercase();
    let is_reserved = matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (upper.starts_with("COM") && upper[3..].parse::<u8>().ok().is_some())
        || (upper.starts_with("LPT") && upper[3..].parse::<u8>().ok().is_some());
    if is_reserved {
        s = format!("_{s}");
    }

    // Avoid path traversal-ish segments.
    if s == "." || s == ".." {
        return "_".to_string();
    }

    s
}

fn sanitize_rel_path(rel: &str) -> String {
    // Normalize separators and sanitize each component.
    // Keep output rel paths using '/' so the web preview URL stays consistent.
    let mut parts: Vec<String> = Vec::new();
    for raw in rel.split(['/', '\\']) {
        if raw.is_empty() {
            continue;
        }
        parts.push(sanitize_component(raw.to_string()));
    }
    parts.join("/")
}

fn normalize_rel_path(rel: &str) -> String {
    rel.replace('\\', "/")
}

fn normalize_rel_dir_key(dir: &Path) -> String {
    dir.to_string_lossy()
        .replace('\\', "/")
        .trim_matches('/')
        .to_string()
}

fn is_safe_rel_path(rel: &str) -> bool {
    let p = Path::new(rel);
    for c in p.components() {
        match c {
            Component::Normal(_) => {}
            _ => return false,
        }
    }
    true
}

async fn cleanup_empty_parents(root: &Path, start: Option<&Path>) -> Result<(), std::io::Error> {
    let Some(mut cur) = start.map(|p| p.to_path_buf()) else {
        return Ok(());
    };
    loop {
        if cur == root {
            break;
        }
        // Stop if outside root.
        if !cur.starts_with(root) {
            break;
        }
        match tokio::fs::read_dir(&cur).await {
            Ok(mut rd) => {
                if rd.next_entry().await?.is_some() {
                    break;
                }
            }
            Err(_) => break,
        }
        // empty
        let _ = tokio::fs::remove_dir(&cur).await;
        if let Some(p) = cur.parent() {
            cur = p.to_path_buf();
        } else {
            break;
        }
    }
    Ok(())
}

fn load_counters_sync(path: &Path) -> Result<BTreeMap<String, usize>, OutputError> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let bytes = std::fs::read(path)?;
    let map: BTreeMap<String, usize> = serde_json::from_slice(&bytes)?;
    Ok(map)
}

fn save_counters_sync(path: &Path, map: &BTreeMap<String, usize>) -> Result<(), OutputError> {
    let bytes = serde_json::to_vec_pretty(map)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

fn scan_existing_outputs_sync(
    root: &Path,
    counters: &mut BTreeMap<String, usize>,
) -> Result<bool, OutputError> {
    if !root.exists() {
        return Ok(false);
    }
    let mut changed = false;
    scan_dir_sync(root, root, counters, &mut changed)?;
    Ok(changed)
}

fn scan_dir_sync(
    root: &Path,
    dir: &Path,
    counters: &mut BTreeMap<String, usize>,
    changed: &mut bool,
) -> Result<(), OutputError> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            scan_dir_sync(root, &p, counters, changed)?;
            continue;
        }
        if p.extension().and_then(|s| s.to_str()) != Some("png") {
            continue;
        }
        let Ok(rel) = p.strip_prefix(root) else {
            continue;
        };
        let Some(file_name) = p.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(idx) = parse_output_index(file_name) else {
            continue;
        };
        let dir_key = rel
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        let next = idx + 1;
        let cur = counters.get(&dir_key).copied().unwrap_or(0);
        if next > cur {
            counters.insert(dir_key, next);
            *changed = true;
        }
    }
    Ok(())
}

fn parse_output_index(file_name: &str) -> Option<usize> {
    // 新默认：00001_xxxxxx_seed.png（编号在前，5 位十进制）
    if file_name.len() >= 5 {
        let head = &file_name[..5];
        if head.chars().all(|c| c.is_ascii_digit()) {
            return head.parse::<usize>().ok();
        }
    }

    None
}

fn walk_pngs(
    root: &Path,
    dir: &Path,
    out: &mut Vec<String>,
    limit: usize,
) -> Result<(), OutputError> {
    if out.len() >= limit {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            walk_pngs(root, &p, out, limit)?;
            if out.len() >= limit {
                return Ok(());
            }
        } else if p.extension().and_then(|s| s.to_str()) == Some("png") {
            if let Ok(rel) = p.strip_prefix(root) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
            if out.len() >= limit {
                return Ok(());
            }
        }
    }
    Ok(())
}
