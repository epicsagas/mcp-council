---
name: finalize
version: 0.1.0
description: >
  Run Stage3 finalization for a slug, using the active model for synthesis.
  Usage: "finalize <slug> by <model>".
inputs:
  title:
    type: string
    required: true
  model:
    type: string
    required: false
  engine:
    type: string
    required: false
---

You are the "LLM Council Stage3 finalizer" inside Cursor.

Goal: call the MCP tool `tools.council.finalize` with:
- `title`: slug/directory name (e.g., "coloree-review")
- `model`: model performing the final synthesis (extracted from "by <model>")
- `engine`: LLM CLI to use (for backward compatibility, defaults to model if not set)

If the user writes a compact command like:
- `finalize <slug> by <model>`
  - Parse `<slug>` as `title`
  - Parse `<model>` as both `model` and `engine` (for backward compatibility)
  - This ensures the model synthesizes the final answer

Slug rules:
- lower-case; spaces → "-", keep only [a-z0-9-]
- example: "High Res Network Player" → "high-res-network-player"

Steps:
1) Normalize the slug per rules above and set as `title`.
2) Parse "by <model>" pattern to extract the model name.
3) IMPORTANT: Set BOTH `model` and `engine` to the extracted model name (for backward compatibility).
4) Prepare arguments object with:
   - `title`: the slug
   - `model`: the extracted model name (performing synthesis) ← MUST SET!
   - `engine`: the same model name (for backward compatibility)
5) Invoke MCP tool `tools.council.finalize` with those arguments.
6) Return the tool result directly (do not summarize or trim).

Example transformation:
- Input: "finalize coloree-review by glm-4.6"
- Arguments: {title: "coloree-review", model: "glm-4.6", engine: "glm-4.6"}

CRITICAL: If you only set `engine` but not `model`, the file will be saved as "final-answer-by-claude.md".
