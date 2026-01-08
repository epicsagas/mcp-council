mod mcp;
mod tools;
mod cli_runner;

use anyhow::Result;
use mcp::McpServer;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

// Embed command files at compile time
const CMD_FINALIZE: &str = include_str!("../commands/cc/finalize.md");
const CMD_FIRST_ANSWER: &str = include_str!("../commands/cc/first_answer.md");
const CMD_PEER_REVIEW: &str = include_str!("../commands/cc/peer_review.md");
const CMD_SAVE_REVIEW: &str = include_str!("../commands/cc/save_review.md");
const CMD_SAVE_SUMMARY: &str = include_str!("../commands/cc/save_summary.md");
const CMD_SUMMARIZE: &str = include_str!("../commands/cc/summarize.md");

fn print_help() {
    eprintln!("mcp-council - MCP server for multi-LLM peer review workflow");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  mcp-council                Run MCP server (default)");
    eprintln!("  mcp-council --init         Install to both Cursor and Claude Code (interactive)");
    eprintln!("  mcp-council --init-cursor  Install to ~/.cursor/commands/<folder>/");
    eprintln!("  mcp-council --init-claude  Install to ~/.claude/commands/<folder>/");
    eprintln!("  mcp-council --help         Show this help message");
    eprintln!();
}

fn prompt_subfolder() -> String {
    eprint!("Enter subfolder name (leave empty for default 'cc'): ");
    io::stderr().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input.is_empty() {
        "cc".to_string()
    } else {
        input.to_string()
    }
}

fn install_commands_to(cmd_dir: PathBuf) -> Result<()> {
    fs::create_dir_all(&cmd_dir)?;

    let commands = [
        ("finalize.md", CMD_FINALIZE),
        ("first_answer.md", CMD_FIRST_ANSWER),
        ("peer_review.md", CMD_PEER_REVIEW),
        ("save_review.md", CMD_SAVE_REVIEW),
        ("save_summary.md", CMD_SAVE_SUMMARY),
        ("summarize.md", CMD_SUMMARIZE),
    ];

    for (name, content) in commands {
        let path = cmd_dir.join(name);
        fs::write(&path, content)?;
        eprintln!("  Installed: {}", path.display());
    }

    // Ensure council directory exists
    let home = env::var("HOME").expect("HOME environment variable not set");
    let council_dir = PathBuf::from(&home).join(".council");
    fs::create_dir_all(&council_dir)?;

    eprintln!();
    eprintln!("Commands installed successfully!");
    eprintln!("Council directory: ~/.council/");

    Ok(())
}

fn merge_mcp_config(config_path: &PathBuf) -> Result<bool> {
    let mcp_council_config = json!({
        "command": "mcp-council",
        "args": []
    });

    let mut config: Value = if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    // Ensure mcpServers exists
    if !config.get("mcpServers").is_some() {
        config["mcpServers"] = json!({});
    }

    // Check if mcp-council already exists
    let already_exists = config["mcpServers"].get("mcp-council").is_some();

    // Add or update mcp-council
    config["mcpServers"]["mcp-council"] = mcp_council_config;

    // Write back with pretty formatting
    let formatted = serde_json::to_string_pretty(&config)?;
    fs::write(config_path, formatted)?;

    Ok(already_exists)
}

fn setup_mcp_config(target: &str) -> Result<()> {
    let home = env::var("HOME").expect("HOME environment variable not set");

    match target {
        "cursor" => {
            let config_path = PathBuf::from(&home).join(".cursor").join("mcp.json");
            let already_exists = merge_mcp_config(&config_path)?;
            if already_exists {
                eprintln!("  Updated: {}", config_path.display());
            } else {
                eprintln!("  Added: {}", config_path.display());
            }
        }
        "claude" => {
            let config_path = PathBuf::from(&home).join(".claude.json");
            let already_exists = merge_mcp_config(&config_path)?;
            if already_exists {
                eprintln!("  Updated: {}", config_path.display());
            } else {
                eprintln!("  Added: {}", config_path.display());
            }
        }
        "both" => {
            let cursor_path = PathBuf::from(&home).join(".cursor").join("mcp.json");
            let claude_path = PathBuf::from(&home).join(".claude.json");

            // Ensure .cursor directory exists
            fs::create_dir_all(PathBuf::from(&home).join(".cursor"))?;

            let cursor_exists = merge_mcp_config(&cursor_path)?;
            let claude_exists = merge_mcp_config(&claude_path)?;

            eprintln!();
            eprintln!("[MCP Config]");
            if cursor_exists {
                eprintln!("  Updated: {}", cursor_path.display());
            } else {
                eprintln!("  Added: {}", cursor_path.display());
            }
            if claude_exists {
                eprintln!("  Updated: {}", claude_path.display());
            } else {
                eprintln!("  Added: {}", claude_path.display());
            }
        }
        _ => {}
    }

    Ok(())
}

fn install_commands(base_dir: &str, subfolder: &str) -> Result<()> {
    let home = env::var("HOME").expect("HOME environment variable not set");
    let cmd_dir = PathBuf::from(&home).join(base_dir).join("commands").join(subfolder);
    install_commands_to(cmd_dir)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--help" | "-h") => {
            print_help();
            Ok(())
        }
        Some("--init") => {
            let subfolder = prompt_subfolder();
            eprintln!();
            eprintln!("[Cursor]");
            install_commands(".cursor", &subfolder)?;
            eprintln!("\n[Claude Code]");
            install_commands(".claude", &subfolder)?;
            setup_mcp_config("both")?;
            eprintln!();
            eprintln!("✅ Installation complete! Restart Cursor/Claude Code to activate.");
            Ok(())
        }
        Some("--init-cursor") => {
            let subfolder = prompt_subfolder();
            eprintln!();
            install_commands(".cursor", &subfolder)?;
            setup_mcp_config("cursor")?;
            eprintln!();
            eprintln!("✅ Installation complete! Restart Cursor to activate.");
            Ok(())
        }
        Some("--init-claude") => {
            let subfolder = prompt_subfolder();
            eprintln!();
            install_commands(".claude", &subfolder)?;
            setup_mcp_config("claude")?;
            eprintln!();
            eprintln!("✅ Installation complete! Restart Claude Code to activate.");
            Ok(())
        }
        _ => {
            let mut server = McpServer::new();
            server.run().await
        }
    }
}

