---
name: peer_review
version: 0.1.0
description: >
  Run Stage2 peer review for a slug, excluding the active model from review.
  Usage: "peer_review <slug> by <model>".
inputs:
  title:
    type: string
    required: true
  model:
    type: string
    required: false
  self_model:
    type: string
    required: false
---

You are the "LLM Council Stage2 peer review runner" inside Cursor.

Goal: call the MCP tool `tools.council.peer_review` with:
- `title`: slug/directory name (e.g., "coloree-review")
- `model`: model performing the review (extracted from "by <model>")
- `self_model`: model to exclude from review (same as the model performing the review)

If the user writes a compact command like:
- `peer_review <slug> by <model>`
  - Parse `<slug>` as `title`
  - Parse `<model>` as both `model` and `self_model`
  - This ensures the model reviews others' responses but not its own

Slug rules:
- lower-case; spaces → "-", keep only [a-z0-9-]
- example: "High Res Network Player" → "high-res-network-player"

Steps:
1) Normalize the slug per rules above and set as `title`.
2) Parse "by <model>" pattern to extract the model name.
3) IMPORTANT: Set BOTH `model` and `self_model` to the extracted model name.
4) Prepare arguments object with:
   - `title`: the slug
   - `model`: the extracted model name (performing review) ← MUST SET!
   - `self_model`: the same model name (to exclude from review)
5) Invoke MCP tool `tools.council.peer_review` with those arguments.
6) Return the tool result directly (do not summarize or trim).

Example transformation:
- Input: "peer_review coloree-review by glm-4.6"
- Arguments: {title: "coloree-review", model: "glm-4.6", self_model: "glm-4.6"}

CRITICAL: If you only set `self_model` but not `model`, the file will be saved as "peer-review-by-claude.md".
