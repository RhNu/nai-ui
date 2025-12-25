use std::collections::HashMap;

use async_recursion::async_recursion;
use regex::Regex;

use nai_core::{config::AppConfig, prompt};

use crate::prompt_snippet_store::PromptSnippetStore;

const MAX_DEPTH: usize = 8;
const MAX_TOTAL_EXPANSIONS: usize = 64;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SnippetExpansionResult {
    pub positive: String,
    pub negative: String,
    pub warnings: Vec<String>,
}

pub async fn expand_prompts_pair(
    cfg: &AppConfig,
    store: &PromptSnippetStore,
    positive: &str,
    negative: &str,
) -> anyhow::Result<SnippetExpansionResult> {
    let mut cache = HashMap::new();
    let mut warnings = Vec::new();
    let mut total = 0usize;

    let pos = expand_text(cfg, store, positive, &mut cache, &mut warnings, &mut total).await?;
    let neg = expand_text(cfg, store, negative, &mut cache, &mut warnings, &mut total).await?;

    Ok(SnippetExpansionResult {
        positive: pos,
        negative: neg,
        warnings,
    })
}

async fn expand_text(
    cfg: &AppConfig,
    store: &PromptSnippetStore,
    text: &str,
    cache: &mut HashMap<String, String>,
    warnings: &mut Vec<String>,
    total: &mut usize,
) -> anyhow::Result<String> {
    let token_re = Regex::new(r"<\s*snippet:([^>]+?)\s*>").expect("valid snippet regex");

    let mut out = String::new();
    let mut last_idx = 0;

    for cap in token_re.captures_iter(text) {
        let m = cap.get(0).expect("full match");
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();

        out.push_str(&text[last_idx..m.start()]);
        last_idx = m.end();

        if name.is_empty() {
            warnings.push("空 snippet 名称已忽略".to_string());
            continue;
        }

        if *total >= MAX_TOTAL_EXPANSIONS {
            warnings.push(format!(
                "已达到展开上限({MAX_TOTAL_EXPANSIONS})，跳过 {name}"
            ));
            continue;
        }
        *total += 1;

        let expanded =
            resolve_snippet(cfg, store, name, cache, warnings, total, &mut Vec::new(), 0).await?;

        if expanded.is_empty() {
            warnings.push(format!("片段 {name} 无法展开，已移除"));
        }
        out.push_str(&expanded);
    }

    out.push_str(&text[last_idx..]);
    Ok(prompt::format_str(cfg, &out))
}

#[async_recursion]
async fn resolve_snippet(
    cfg: &AppConfig,
    store: &PromptSnippetStore,
    name: &str,
    cache: &mut HashMap<String, String>,
    warnings: &mut Vec<String>,
    total: &mut usize,
    stack: &mut Vec<String>,
    depth: usize,
) -> anyhow::Result<String> {
    if depth >= MAX_DEPTH {
        warnings.push(format!("递归深度超过 {MAX_DEPTH}，跳过 {name}"));
        return Ok(String::new());
    }

    if stack.contains(&name.to_string()) {
        let mut chain = stack.clone();
        chain.push(name.to_string());
        let desc = chain.join(" -> ");
        warnings.push(format!("检测到循环引用：{desc}"));
        return Ok(String::new());
    }

    if let Some(cached) = cache.get(name) {
        return Ok(cached.clone());
    }

    let Some(snippet) = store.get(name).await? else {
        warnings.push(format!("片段不存在：{name}"));
        return Ok(String::new());
    };

    let mut new_stack = stack.clone();
    new_stack.push(name.to_string());

    let mut local_total = *total;
    let expanded_body =
        expand_text(cfg, store, &snippet.body, cache, warnings, &mut local_total).await?;
    *total = local_total;

    cache.insert(name.to_string(), expanded_body.clone());
    Ok(expanded_body)
}
