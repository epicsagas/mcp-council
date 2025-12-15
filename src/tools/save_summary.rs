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

pub async fn handle_save_summary(params: Value) -> Result<Value> {
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

    let summary_content = params["content"]
        .as_str()
        .context("Missing required parameter: content")?;

    // Debug logging
    eprintln!("DEBUG: save_summary called with params: title={}, model={}, content_len={}",
        title, model, summary_content.len());

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
        "# Summary\n- title: {}\n- model: {}\n\n## Summary Content\n\n{}",
        title, model, summary_content
    );

    // Save markdown file
    let summary_md_path = base_dir.join("summary.md");
    fs::write(&summary_md_path, &markdown)
        .context(format!("Failed to write summary file: {} (searched from: {})",
            summary_md_path.display(),
            env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).display()))?;

    Ok(json!({
        "success": true,
        "file_saved": summary_md_path.to_string_lossy(),
        "summary": format!("Summary saved to {}", summary_md_path.display())
    }))
}

