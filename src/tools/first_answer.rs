use anyhow::{Context, Result};
use chrono::Utc;
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

fn sanitize_model(model: &str) -> String {
    let lowered = model.to_lowercase();
    let sanitized: String = lowered
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
        "unknown-model".to_string()
    } else {
        sanitized
    }
}

pub async fn handle_first_answer(params: Value) -> Result<Value> {
    let title = params["title"]
        .as_str()
        .context("Missing required parameter: title")?;
    let model_raw = params["model"]
        .as_str()
        .unwrap_or("unknown-model")
        .trim();
    let model = if model_raw.is_empty() {
        "unknown-model"
    } else {
        model_raw
    };
    let prompt = params["prompt"]
        .as_str()
        .context("Missing required parameter: prompt")?;
    let content = params["content"]
        .as_str()
        .context("Missing required parameter: content")?;

    // Debug logging
    eprintln!(
        "DEBUG: first_answer called with params: title={}, model={}, prompt_len={}, content_len={}",
        title,
        model,
        prompt.len(),
        content.len()
    );

    let council_base = find_council_dir()?;
    let base_dir = council_base.join(title);

    // Ensure directory exists
    fs::create_dir_all(&base_dir).context(format!(
        "Failed to create/find council directory: {}",
        base_dir.display()
    ))?;

    let model_for_file = sanitize_model(model);
    let mut file_name = format!("{}-answer.md", model_for_file);
    let mut file_path = base_dir.join(&file_name);

    if file_path.exists() {
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
        file_name = format!("{}-answer-{}.md", model_for_file, timestamp);
        file_path = base_dir.join(&file_name);
    }

    let markdown = format!(
        "# {model} answer\n- model: {model}\n- prompt: {prompt}\n- created_at: {created_at}\n\n{content}\n",
        model = model,
        prompt = prompt,
        created_at = Utc::now().to_rfc3339(),
        content = content
    );

    fs::write(&file_path, &markdown).context(format!(
        "Failed to write Stage1 answer file: {} (searched from: {})",
        file_path.display(),
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .display()
    ))?;

    Ok(json!({
        "success": true,
        "file_saved": file_path.to_string_lossy(),
        "summary": format!("Stage1 answer saved to {}", file_path.display())
    }))
}


