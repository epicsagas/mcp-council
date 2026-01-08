mod mcp;
mod tools;
mod cli_runner;

use anyhow::Result;
use mcp::McpServer;
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
            install_commands(".claude", &subfolder)
        }
        Some("--init-cursor") => {
            let subfolder = prompt_subfolder();
            eprintln!();
            install_commands(".cursor", &subfolder)
        }
        Some("--init-claude") => {
            let subfolder = prompt_subfolder();
            eprintln!();
            install_commands(".claude", &subfolder)
        }
        _ => {
            let mut server = McpServer::new();
            server.run().await
        }
    }
}

