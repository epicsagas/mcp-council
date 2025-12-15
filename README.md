# LLM Council Automation System

**Multi-Model Reasoning → Peer Review → Final Synthesis (Rust MCP Server)**

A Rust-based MCP server that enables AI model collaboration through structured peer review. The system implements a 3-stage deliberation process where multiple LLMs collaboratively answer questions with anonymized peer review to prevent bias.

> Inspired by the workflow concept of **karpathy/llm-council** (no license declared as of 2025-12-12); all code and documentation here are independently authored. Primary interface is **Cursor Chat** or **Claude Code** through MCP tools and slash commands.

---

## Overview (3 Stages)

1) **Stage1 — First Opinions**: One prompt in Cursor with multiple selected models → each model writes its answer to `~/.council/<slug>/*-answer.md`.
2) **Stage2 — Peer Review**: 
   - Run `/peer_review <slug> by <model>` to generate a review prompt (excludes the model's own response)
   - The model generates the review content
   - **Optional**: If the model doesn't automatically save the review, use `/save_review <slug> <model> <content>` as a fallback to manually save to `peer-review-by-<model>.md`
3) **Stage3 — Final Answer**: From a (single) model tab, run `/finalize <slug> by <model>` to synthesize all responses and reviews into `final-answer-by-<model>.md`.

---

## How It Fits Together (Chat Commands → MCP Server)

```
[Chat Commands (Cursor/Claude Code)]
  ├─ /first_answer "<title>"            -> Stage1 capture (multi-model answers)
  ├─ /summarize <slug> <model> <content> -> tools.council.summarize (Optional: reduce token costs)
  ├─ /save_summary <slug> <model> <content> -> tools.council.save_summary (Save summary)
  ├─ /peer_review <slug> by <model>     -> tools.council.peer_review (Stage2, self-exclusion)
  ├─ /save_review <slug> <model> <content> -> tools.council.save_review (Save peer review)
  └─ /finalize <slug> by <model>       -> tools.council.finalize (Stage3 synthesis)
                     ▼
[Rust MCP Server: mcp-council]
  Exposes tools.council.{first_answer,peer_review,save_review,finalize,summarize,save_summary}
                     ▼
[Current AI Model Context]
  Direct processing without external CLI calls
```

---

## Key Paths

```
mcp-council/          # Rust MCP server source
  ├─ src/
  ├─ Cargo.toml
  └─ QUICKSTART.md
.cursor/commands/cc/  # Chat-triggered commands (Stage1/2/3)
~/.council/{slug}/    # Outputs (answers, peer reviews, final synthesis)
```

Outputs example:

```
.council/high-res-network-player/
  ├─ gpt-5-answer.md
  ├─ claude-answer.md
  ├─ gemini-answer.md
  ├─ summary.md                    # Optional: summary for large documents
  ├─ peer-review-by-sonnet.md
  └─ final-answer-by-sonnet.md
```

---

## Chat Commands (Universal for Cursor/Claude Code)

- **Stage1 (collect answers)**
  `/first_answer "High Res Network Player"`
- **Stage2 (peer review, with self-exclusion)**
  ```
  /peer_review high-res-network-player by glm-4.6
  ```
  - Automatically excludes the specified model's own response
  - Returns structured prompt for current model to process
  - The model generates the review content
  - Some models may automatically save the review file; if not, use the fallback below
  
  **Fallback: Manual save (if needed)**
  ```
  /save_review high-res-network-player glm-4.6 "Review content..."
  ```
  - Use this only if the model didn't automatically save the review
  - Saves peer review to `peer-review-by-glm-4.6.md`
  - Stores in `~/.council/<slug>/` directory

- **Stage3 (final synthesis)**
  ```
  /finalize high-res-network-player by claude
  ```
  - Synthesizes all responses and reviews
  - Uses `by <model>` format to specify the synthesizing model

- **Optional: Summarize large documents (reduce token costs)**
  ```
  /summarize high-res-network-player sonnet "Very long document..." max_length=2000
  ```
  - Generates a summary prompt for large documents
  - After model generates summary, save it:
  ```
  /save_summary high-res-network-player sonnet "Summary content..."
  ```
  - Saves to `summary.md` for use in Stage2/Stage3 to reduce token costs

**File Structure**:

```
.council/<slug>/
├── <model>-answer.md
├── summary.md                    # Optional: for large documents
├── peer-review-by-<model>.md
└── final-answer-by-<model>.md
```

---

## Install & Wire Up

1) **Build MCP server**

```bash
cargo build --release
cp target/release/mcp-council ~/.local/bin/
chmod +x ~/.local/bin/mcp-council
```

2) **Register MCP in Cursor** (`~/.cursor/mcp.json`)

```json
{
  "servers": {
    "llm-council": {
      "command": "mcp-council",
      "args": []
    }
  }
}
```

3) **Install chat commands**

```bash
# Ensure council root exists (home-scoped)
mkdir -p ~/.council

# For Cursor
mkdir -p ~/.cursor/commands/cc
cp mcp-council/commands/cc/* ~/.cursor/commands/cc/

# For Claude Code (per-project)
mkdir -p .cursor/commands/cc
cp mcp-council/commands/cc/* .cursor/commands/cc/

# Or globally for Claude Code
mkdir -p ~/.claude/commands/cc
cp mcp-council/commands/cc/* ~/.claude/commands/cc/
```

For a full walkthrough, see [QUICKSTART.md](QUICKSTART.md).

---

## Key Features

- **Self-Exclusion**: Each model automatically excludes its own response from peer review
- **Default Location**: Uses `~/.council/{slug}` for storage by default
- **Universal Compatibility**: Works with both Cursor and Claude Code
- **No External Dependencies**: Direct processing within current AI context
- **Anonymized Review**: Models evaluate responses without knowing which model wrote them
- **Token Cost Optimization**: Optional `summarize` tool to reduce token costs for large documents in Stage2/Stage3

## Technical Notes

- **MCP Protocol**: JSON-RPC 2.0 compliant server
- **Async Rust**: Non-blocking I/O operations
- **Error Handling**: Comprehensive error propagation and context
- **File Discovery**: Intelligent `.council/` directory search up to 10 parent levels
- **Model Support**: Extensible for any LLM with proper naming conventions

## License

**[Apache-2.0](LICENSE)**

---

## Acknowledgements

- Model Context Protocol (MCP)
- Anthropic Claude
- Google Gemini
- OpenAI GPT Models
- Cursor IDE Integration
- Claude Code Integration
