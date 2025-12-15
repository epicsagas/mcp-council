# Quickstart: LLM Council Automation (Cursor Chat First)

This guide shows the fastest way to run the 3-step LLM Council workflow **via Cursor Chat**. The MCP server (`mcp-council` binary) runs in the background; you drive everything from chat commands.

---

## Goal

In about 3-5 minutes you will:

1. Generate Stage1 with multiple models (3 models → 3 answer files)
2. *(Optional)* Summarize large documents to reduce token costs
3. Run Stage2 Peer Review via Rust MCP server (some models auto-save, others may need manual save fallback)
4. Produce the Stage3 Final Answer

**Prereq:** ensure `~/.council` exists (home-scoped):

```
mkdir -p ~/.council
```

---

## Stage1 — Collect multi-model answers (Cursor Chat command: `/first_answer`)

### Step 1: Write your question

Create `prompt.txt` in the Cursor workspace and add your question:

```
List the key technical considerations for a high-end network audio player.
```

### Step 2: Run Stage1 (Cursor Chat, single prompt → multiple models)

```
/first_answer "High Res Network Player"
```

This command reads answers from:

- GPT-5
- Claude-4.5
- Gemini

And saves them to (default root):

```
~/.council/high-res-network-player/
```

Behind the scenes, `/first_answer` calls the MCP tool `council.first_answer` per model tab so file writing is handled by the tool (not by the model itself).

Example outputs:

```
gpt-5-answer.md
claude-answer.md
gemini-answer.md
```

Only one line of user input needed.

---

## Optional: Summarize Large Documents (Reduce Token Costs)

If your original document or prompt is very large, you can summarize it before Stage2/Stage3 to reduce token costs:

**Step 1: Generate summary prompt**

```
/summarize high-res-network-player sonnet "Very long document content..." max_length=2000
```

This generates a summary prompt and saves it to `summary-prompt.md`.

**Step 2: Save the summary**

After the model generates the summary:

```
/save_summary high-res-network-player sonnet "Summary content..."
```

This saves the summary to `summary.md`, which can be used in Stage2/Stage3 instead of the full document.

**Note**: This is optional. Use it only when dealing with very large documents to reduce token costs.

---

## Stage2 — Peer Review (Cursor Chat → MCP)

Once Stage1 is ready, pick a model tab and run Peer Review there:

**Cursor Chat command** (from `.cursor/commands/cc/peer_review.md`, run inside a model tab):

```
/peer_review high-res-network-player by sonnet
```

(`by <model>` sets `self_model` to exclude that model's own answer.)

Cursor automatically calls the Rust MCP tool:

```
tools.council.peer_review
```

The tool generates a review prompt, and the model creates the peer review content. **Some models may automatically save the review file; others may not.**

### Fallback: Manual save (if needed)

If the model didn't automatically save the review, use:

```
/save_review high-res-network-player sonnet
```

This calls `tools.council.save_review` and saves the review to:

```
peer-review-by-sonnet.md
```

**Note**: `save_review` is a fallback tool. Use it only when the model doesn't automatically save the review file.

---

## Stage3 — Final Answer (Cursor Chat → MCP) — Command: `/finalize` (`.cursor/commands/cc/finalize.md`)

**Cursor Chat command** (from `.cursor/commands/cc/finalize.md`, run inside one model tab):

```
/finalize high-res-network-player by sonnet
```

(Use `by <model>` format to specify the synthesizing model, consistent with `peer_review`.)

Cursor automatically calls:

```
tools.council.finalize
```

Output (human-facing):

```
final-answer-by-sonnet.md    # primary readable output
```

Response includes `markdown` so the final answer renders nicely.

You get a single best-quality answer that incorporates all model opinions and reviews.

---

## Result file layout

```
~/.council/high-res-network-player/
  ├── gpt-5-answer.md
  ├── sonnet-answer.md
  ├── gemini-answer.md
  ├── summary.md                    # Optional: if summarize was used
  ├── peer-review-by-sonnet.md
  ├── peer-review-by-gemini.md
  └── final-answer-by-sonnet.md
```

---

## Verify MCP server & commands

1) MCP server in PATH

```bash
which mcp-council
```

2) Cursor commands available (global or project)

```bash
ls ~/.cursor/commands | grep council
# or
ls .cursor/commands | grep council
```
