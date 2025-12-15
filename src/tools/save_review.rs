use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::PathBuf;

fn find_council_dir() -> Result<PathBuf> {
    let home = env::var("HOME").context("HOME not set")?;
    let council = PathBuf::from(home).join(".council");
    if !council.exists() {
        fs::create_dir_all(&council)
            .context(format!("Failed to create council directory: {}", council.display()))?;
    }
    Ok(council)
}

pub async fn handle_save_review(params: Value) -> Result<Value> {
    let title = params["title"]
        .as_str()
        .context("Missing required parameter: title")?;
    // Try to get model from various sources in priority order:
    // 1. Explicit model parameter
    // 2. engine parameter (for backward compatibility)
    // 3. Default to "claude"
    let model_raw = params["model"]
        .as_str()
        .or_else(|| params["engine"].as_str())
        .unwrap_or("claude");
    let model_trimmed = model_raw.trim();
    let model = if model_trimmed.is_empty() { "claude" } else { model_trimmed };

    // Debug logging
    eprintln!("DEBUG: save_review called with params: title={}, model={}, model_raw={:?}, engine={:?}",
        title, model, params.get("model"), params.get("engine"));

    let model_for_file: String = {
        let sanitized: String = model
            .chars()
            .map(|c: char| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        let cleaned = sanitized.trim_matches('-');
        if cleaned.is_empty() {
            "claude".to_string()
        } else {
            sanitized
        }
    };

    let review_content = params["content"]
        .as_str()
        .context("Missing required parameter: content")?;

    let council_base = find_council_dir()?;
    let base_dir = council_base.join(title);

    if !base_dir.exists() {
        return Err(anyhow::anyhow!(
            "Council directory not found: {}",
            base_dir.display()
        ));
    }

    // Build markdown content
    let markdown = format!(
        "# Peer Review\n- title: {}\n- model: {}\n\n## Review Content\n\n{}",
        title, model, review_content
    );

    // Save markdown file
    let review_md_path = base_dir.join(format!("peer-review-by-{}.md", model_for_file));
    fs::write(&review_md_path, &markdown)
        .context(format!("Failed to write review file: {} (searched from: {})",
            review_md_path.display(),
            env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).display()))?;

    Ok(json!({
        "success": true,
        "file_saved": review_md_path.to_string_lossy(),
        "summary": format!("Peer review saved to {}", review_md_path.display())
    }))
}