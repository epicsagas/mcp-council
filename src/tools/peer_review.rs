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

pub async fn handle_peer_review(params: Value) -> Result<Value> {
    let title = params["title"]
        .as_str()
        .context("Missing required parameter: title")?;
    // Try to get model from various sources in priority order:
    // 1. Explicit model parameter
    // 2. self_model (when model is not explicitly set but self_model is)
    // 3. engine parameter (for backward compatibility)
    // 4. Default to "claude"
    let model_raw = params["model"]
        .as_str()
        .or_else(|| params.get("self_model").and_then(|v| v.as_str()))
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
    let self_model = params.get("self_model").and_then(|v| v.as_str());

    // Debug logging
    eprintln!("DEBUG: peer_review called with params: title={}, model={}, self_model={}",
        title, model, self_model.unwrap_or("None"));

    let council_base = find_council_dir()?;
    let base_dir = council_base.join(title);
    
    // Debug logging
    eprintln!("DEBUG: peer_review - council_base={}, base_dir={}",
        council_base.display(), base_dir.display());
    
    if !base_dir.exists() {
        return Err(anyhow::anyhow!(
            "Directory not found: {} (council base: {})",
            base_dir.display(),
            council_base.display()
        ));
    }

    // Find all Stage1 answer files (markdown preferred, JSON for backward compatibility)
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

    // Load and parse all answer files, optionally excluding self_model
    let mut answers = Vec::new();
    let mut labels = Vec::new();
    for file_path in answer_files.iter() {
        let content_value = read_stage1_answer(file_path)
            .context(format!("Failed to parse answer file: {}", file_path.display()))?;

        let model_name = content_value
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown-model");

        if let Some(self_model_name) = self_model {
            if model_name.eq_ignore_ascii_case(self_model_name) {
                eprintln!(
                    "INFO: Skipping self_model '{}' from peer review",
                    self_model_name
                );
                continue;
            }
        }

        answers.push(json!({
            "file": file_path.file_name().unwrap().to_string_lossy(),
            "content": content_value
        }));
    }

    if answers.is_empty() {
        return Err(anyhow::anyhow!(
            "No Stage1 answers available after applying self_model exclusion"
        ));
    }

    // Re-label responses after exclusion to keep labels consecutive
    for (idx, answer) in answers.iter_mut().enumerate() {
        let label = format!("Response {}", char::from(b'A' + idx as u8));
        labels.push(label.clone());
        answer["label"] = json!(label);
    }

    // Build review prompt
    let user_query = extract_user_query(&base_dir)?;
    
    let responses_text = answers
        .iter()
        .map(|a| {
            format!(
                "{}:\n{}",
                a["label"].as_str().unwrap(),
                format_response_content(&a["content"])
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let _ranking_prompt = format!(
        r#"You are evaluating different responses to the following question:

Question: {}

Here are the responses from different models (anonymized):

{}

Your task:
1. First, evaluate each response individually. For each response, explain what it does well and what it does poorly.
2. Then, at the very end of your response, provide a final ranking.

IMPORTANT: Your final ranking MUST be formatted EXACTLY as follows:
- Start with the line "FINAL RANKING:" (all caps, with colon)
- Then list the responses from best to worst as a numbered list
- Each line should be: number, period, space, then ONLY the response label (e.g., "1. Response A")
- Do not add any other text or explanations in the ranking section

Example of the correct format for your ENTIRE response:

Response A provides good detail on X but misses Y...
Response B is accurate but lacks depth on Z...
Response C offers the most comprehensive answer...

FINAL RANKING:
1. Response C
2. Response A
3. Response B

Now provide your evaluation and ranking:"#,
        user_query, responses_text
    );

    // Create a prompt template that the current model should use
    let review_request_prompt = format!(
        r#"Please perform a peer review of the following responses and provide your evaluation in this exact format:

## User Question
{}

## Responses to Review
{}

## Instructions
1. Evaluate each response individually
2. Provide "FINAL RANKING:" section with numbered list from best to worst
3. Use exact format: "1. Response A", "2. Response B", etc.

Note: Your own response (if present) has been excluded from this review.

After you complete your review, the system will save it as: peer-review-by-{}.md"#,
        user_query, responses_text, model
    );

    Ok(json!({
        "success": true,
        "action": "perform_peer_review_and_save",
        "review_request": review_request_prompt,
        "output_file": format!("peer-review-by-{}.md", model_for_file),
        "output_dir": base_dir.display().to_string(),
        "instruction": "Please provide your peer review evaluation. When you're done, I'll save it to the specified file."
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

    // Treat as markdown/plain text
    Ok(json!({
        "model": model_from_name,
        "response": content,
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


