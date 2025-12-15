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

pub async fn handle_summarize(params: Value) -> Result<Value> {
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
    let content = params["content"]
        .as_str()
        .context("Missing required parameter: content")?;
    let max_length = params["max_length"]
        .as_u64()
        .unwrap_or(2000); // Default: 2000 characters

    // Debug logging
    eprintln!(
        "DEBUG: summarize called with params: title={}, model={}, content_len={}, max_length={}",
        title,
        model,
        content.len(),
        max_length
    );

    let council_base = find_council_dir()?;
    let base_dir = council_base.join(title);

    // Ensure directory exists
    fs::create_dir_all(&base_dir).context(format!(
        "Failed to create/find council directory: {}",
        base_dir.display()
    ))?;

    // Build summary prompt
    let summary_prompt = format!(
        r#"Please summarize the following content concisely. The summary should be comprehensive but concise, capturing all key points and important details. Target length: approximately {} characters.

Original Content:
{}

Provide a clear, well-structured summary that preserves all essential information:"#,
        max_length, content
    );

    // Save summary prompt to file (for reference)
    let summary_prompt_path = base_dir.join("summary-prompt.md");
    let prompt_markdown = format!(
        "# Summary Request\n- title: {}\n- model: {}\n- created_at: {}\n- original_length: {}\n- target_length: {}\n\n## Summary Prompt\n\n{}",
        title,
        model,
        Utc::now().to_rfc3339(),
        content.len(),
        max_length,
        summary_prompt
    );
    fs::write(&summary_prompt_path, &prompt_markdown).context(format!(
        "Failed to write summary prompt file: {}",
        summary_prompt_path.display()
    ))?;

    Ok(json!({
        "success": true,
        "action": "generate_summary",
        "summary_prompt": summary_prompt,
        "output_file": "summary.md",
        "output_dir": base_dir.display().to_string(),
        "prompt_file": summary_prompt_path.to_string_lossy(),
        "instruction": "Please generate a concise summary of the provided content. When you're done, I'll save it to summary.md in the council directory."
    }))
}

