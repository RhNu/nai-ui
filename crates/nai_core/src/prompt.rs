use std::collections::BTreeMap;
use std::path::Path;

use rand::Rng;
use regex::Regex;

use crate::config::AppConfig;

pub fn format_str(cfg: &AppConfig, text: &str) -> String {
    if !cfg.format_input {
        return text.to_string();
    }

    // Mimic utils.format_str: normalize commas/spaces per line, preserve newlines.
    let comma_re = Regex::new(r"[,\s]*,[,\s]*").expect("valid regex");
    let spaces_re = Regex::new(r" +").expect("valid regex");

    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        if line == "\n" {
            out.push('\n');
            continue;
        }
        let (content, has_nl) = line
            .strip_suffix('\n')
            .map(|c| (c, true))
            .unwrap_or((line, false));
        let mut s = comma_re.replace_all(content, ", ").to_string();
        s = spaces_re.replace_all(&s, " ").to_string();
        let s = s.trim();
        out.push_str(s);
        if has_nl {
            out.push('\n');
        }
    }
    out
}

/// Implements wildcard syntax: <类别:随机|顺序|文件名>
/// 顺序模式会把计数持久化到 output_dir/temp_wildcards.json
pub async fn replace_wildcards(
    cfg: &AppConfig,
    wildcards_root: &Path,
    output_dir: &Path,
    text: &str,
) -> anyhow::Result<String> {
    let pattern = Regex::new(r"<([^:]+):([^>]+)>").expect("valid regex");
    let mut out = text.to_string();

    let mut sequence_state = load_sequence_state(output_dir).await.unwrap_or_default();
    let mut changed = false;

    let caps: Vec<(String, String)> = pattern
        .captures_iter(text)
        .filter_map(|c| Some((c.get(1)?.as_str().to_string(), c.get(2)?.as_str().to_string())))
        .collect();

    for (category, selector) in caps {
        let category_dir = wildcards_root.join(&category);
        if !category_dir.is_dir() {
            continue;
        }

        let mut rng = rand::rng();

        let replacement = if selector == "随机" {
            let mut entries: Vec<_> = std::fs::read_dir(&category_dir)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("txt"))
                .collect();
            entries.sort_by_key(|e| e.file_name());
            if entries.is_empty() {
                continue;
            }
            let idx = rng.random_range(0..entries.len());
            std::fs::read_to_string(entries[idx].path())?
        } else if selector == "顺序" {
            let mut entries: Vec<_> = std::fs::read_dir(&category_dir)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("txt"))
                .collect();
            entries.sort_by_key(|e| e.file_name());
            if entries.is_empty() {
                continue;
            }
            let next = sequence_state
                .get(&category)
                .copied()
                .unwrap_or(usize::MAX)
                .wrapping_add(1);
            let idx = if next >= entries.len() { 0 } else { next };
            sequence_state.insert(category.clone(), idx);
            changed = true;
            std::fs::read_to_string(entries[idx].path())?
        } else {
            let path = category_dir.join(format!("{selector}.txt"));
            if !path.is_file() {
                continue;
            }
            std::fs::read_to_string(path)?
        };

        out = out.replace(&format!("<{category}:{selector}>"), replacement.trim());
    }

    if changed {
        save_sequence_state(output_dir, &sequence_state).await?;
    }

    Ok(format_str(cfg, &out))
}

async fn load_sequence_state(output_dir: &Path) -> anyhow::Result<BTreeMap<String, usize>> {
    let path = output_dir.join("temp_wildcards.json");
    let bytes = tokio::fs::read(&path).await?;
    let map = serde_json::from_slice(&bytes)?;
    Ok(map)
}

async fn save_sequence_state(output_dir: &Path, map: &BTreeMap<String, usize>) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(output_dir).await?;
    let path = output_dir.join("temp_wildcards.json");
    let bytes = serde_json::to_vec_pretty(map)?;
    tokio::fs::write(path, bytes).await?;
    Ok(())
}
