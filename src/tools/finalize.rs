use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn find_council_dir() -> Result<PathBuf> {
    let home = env::var("HOME").context("HOME not set")?;
    let council = PathBuf::from(home).join(".council");
    if !council.exists() {
        return Err(anyhow::anyhow!(
            "Council directory not found: {}",
            council.display()
        ));
    }
    Ok(council)
}

pub async fn handle_finalize(params: Value) -> Result<Value> {
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
    
    // Keep engine for backward compatibility (use model if engine not explicitly set)
    let engine = params["engine"]
        .as_str()
        .unwrap_or(model);

    let council_base = find_council_dir()?;
    let base_dir = council_base.join(title);
    
    // Debug logging
    eprintln!("DEBUG: finalize called with params: title={}, model={}, council_base={}, base_dir={}",
        title, model, council_base.display(), base_dir.display());
    
    if !base_dir.exists() {
        return Err(anyhow::anyhow!(
            "Directory not found: {} (council base: {})",
            base_dir.display(),
            council_base.display()
        ));
    }

    // Load Stage1 answers (markdown preferred, JSON for backward compatibility)
    let answer_files: Vec<PathBuf> = fs::read_dir(&base_dir)
        .context(format!("Failed to read directory: {}", base_dir.display()))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let file_name = path.file_name()?.to_string_lossy();
            
            if file_name.contains("-answer.md") || file_name.ends_with("answer.md")
                || file_name.contains("-answer.json") || file_name.ends_with("answer.json") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    if answer_files.is_empty() {
        return Err(anyhow::anyhow!(
            "No Stage1 answer files found in {}",
            base_dir.display()
        ));
    }

    let mut stage1_results = Vec::new();
    for file_path in &answer_files {
        let parsed = read_stage1_answer(file_path)
            .context(format!("Failed to parse answer file: {}", file_path.display()))?;
        stage1_results.push(parsed);
    }

    // Load Stage2 reviews (markdown preferred, JSON for backward compatibility)
    let review_files: Vec<PathBuf> = fs::read_dir(&base_dir)
        .context(format!("Failed to read directory: {}", base_dir.display()))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let file_name = path.file_name()?.to_string_lossy();
            
            if file_name.contains("peer-review") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    let mut stage2_results = Vec::new();
    for file_path in &review_files {
        let parsed = read_stage2_review(file_path)
            .context(format!("Failed to parse review file: {}", file_path.display()))?;
        stage2_results.push(parsed);
    }

    if stage2_results.is_empty() {
        return Err(anyhow::anyhow!(
            "No Stage2 review files found. Please run peer_review first."
        ));
    }

    // Extract user query
    let user_query = extract_user_query(&base_dir)?;

    // Build Stage1 text
    let stage1_text = stage1_results
        .iter()
        .enumerate()
        .map(|(idx, result)| {
            let default_model = format!("Model {}", idx + 1);
            let model = result
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or(&default_model);
            let response = format_response_content(result);
            format!("Model: {}\nResponse: {}", model, response)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    // Build Stage2 text
    let stage2_text = stage2_results
        .iter()
        .enumerate()
        .map(|(idx, result)| {
            let default_reviewer = format!("Reviewer {}", idx + 1);
            // Try "model" first (current format), fallback to "engine" (for backward compatibility)
            let model = result
                .get("model")
                .or_else(|| result.get("engine"))
                .and_then(|v| v.as_str())
                .unwrap_or(&default_reviewer);
            let review = result
                .get("review")
                .or_else(|| result.get("content"))
                .and_then(|v| v.as_str())
                .unwrap_or("No review content");
            format!("Model: {}\nRanking: {}", model, review)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    // Build chairman prompt
    let chairman_prompt = format!(
        r#"You are the Chairman of an LLM Council. Multiple AI models have provided responses to a user's question, and then ranked each other's responses.

Original Question: {}

STAGE 1 - Individual Responses:
{}

STAGE 2 - Peer Rankings:
{}

Your task as Chairman is to synthesize all of this information into a single, comprehensive, accurate answer to the user's original question. Consider:
- The individual responses and their insights
- The peer rankings and what they reveal about response quality
- Any patterns of agreement or disagreement

Provide a clear, well-reasoned final answer that represents the council's collective wisdom:"#,
        user_query, stage1_text, stage2_text
    );

    // Return the data and prompt for the current model to process directly
    Ok(json!({
        "success": true,
        "action": "synthesize_final_answer",
        "data": {
            "title": title,
            "user_query": user_query,
            "stage1_results": stage1_results,
            "stage2_results": stage2_results,
            "model": model,
            "engine": engine,
            "chairman_prompt": chairman_prompt
        },
        "output_file": format!("final-answer-by-{}.md", model_for_file),
        "output_dir": base_dir.display().to_string(),
        "instruction": "As Chairman of the LLM Council, please synthesize all provided information into a comprehensive final answer to the user's question. When you're done, I'll save it to the specified file."
    }))
}


fn extract_user_query(base_dir: &Path) -> Result<String> {
    // Try to find the original query in various possible locations
    let possible_files = [
        "query.txt",
        "user_query.txt",
        "question.txt",
        "input.txt",
    ];

    for file_name in &possible_files {
        let file_path = base_dir.join(file_name);
        if file_path.exists() {
            return Ok(fs::read_to_string(&file_path)?
                .trim()
                .to_string());
        }
    }

    // Try to extract from answer files (both JSON and Markdown)
    let answer_files: Vec<PathBuf> = fs::read_dir(base_dir)
        .context("Failed to read directory")?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let file_name = path.file_name()?.to_string_lossy();
            if file_name.contains("-answer.md") || file_name.ends_with("answer.md")
                || file_name.contains("-answer.json") || file_name.ends_with("answer.json") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    if let Some(first_file) = answer_files.first() {
        let content = fs::read_to_string(first_file)?;
        
        // Try JSON format first
        if let Ok(json_data) = serde_json::from_str::<Value>(&content) {
            if let Some(query) = json_data.get("query").or(json_data.get("user_query")) {
                if let Some(query_str) = query.as_str() {
                    return Ok(query_str.to_string());
                }
            }
        }
        
        // Try Markdown format: look for "- prompt: {prompt}" pattern
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("- prompt:") || line.starts_with("-prompt:") {
                let prompt = line
                    .trim_start_matches("- prompt:")
                    .trim_start_matches("-prompt:")
                    .trim();
                if !prompt.is_empty() {
                    return Ok(prompt.to_string());
                }
            }
        }
    }

    Ok("Unknown query".to_string())
}

fn read_stage1_answer(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path.display()))?;

    let model_from_name = path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.replace("-answer", ""))
        .unwrap_or_else(|| "unknown-model".to_string());

    if let Ok(json_data) = serde_json::from_str::<Value>(&content) {
        let model = json_data.get("model")
            .and_then(|v| v.as_str())
            .unwrap_or(&model_from_name)
            .to_string();
        let response = format_response_content(&json_data);
        return Ok(json!({
            "model": model,
            "response": response,
            "raw": json_data
        }));
    }

    Ok(json!({
        "model": model_from_name,
        "response": content,
        "raw": content
    }))
}

fn read_stage2_review(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path.display()))?;

    let model_from_name = path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.replace("peer-review-by-", ""))
        .unwrap_or_else(|| "unknown-model".to_string());

    if let Ok(json_data) = serde_json::from_str::<Value>(&content) {
        // Try "model" first (current format), fallback to "engine" (for backward compatibility)
        let model = json_data.get("model")
            .or_else(|| json_data.get("engine"))
            .and_then(|v| v.as_str())
            .unwrap_or(&model_from_name)
            .to_string();
        let review = json_data.get("review")
            .or_else(|| json_data.get("content"))
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| format_response_content(&json_data));
        return Ok(json!({
            "model": model,
            "review": review,
            "raw": json_data
        }));
    }

    // Treat as markdown/plain text - try to extract model from markdown metadata
    let mut extracted_model = model_from_name.clone();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("- model:") || line.starts_with("-model:") {
            let model = line
                .trim_start_matches("- model:")
                .trim_start_matches("-model:")
                .trim();
            if !model.is_empty() {
                extracted_model = model.to_string();
                break;
            }
        }
    }

    Ok(json!({
        "model": extracted_model,
        "review": content,
        "raw": content
    }))
}

fn format_response_content(content: &Value) -> String {
    // Try to extract the actual response text from various possible JSON structures
    if let Some(text) = content.get("response").and_then(|v| v.as_str()) {
        return text.to_string();
    }
    if let Some(text) = content.get("content").and_then(|v| v.as_str()) {
        return text.to_string();
    }
    if let Some(text) = content.as_str() {
        return text.to_string();
    }
    
    // Fallback: pretty print the JSON
    serde_json::to_string_pretty(content).unwrap_or_else(|_| "Invalid content".to_string())
}


