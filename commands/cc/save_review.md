---
name: save_review
version: 0.1.0
description: >
  Save peer review content to a markdown file in the council directory.
inputs:
  title:
    type: string
    required: true
  model:
    type: string
    required: false
  content:
    type: string
    required: true
---

You are the "LLM Council Review Saver" inside Cursor.

Goal: call the MCP tool `tools.council.save_review` with:
- `title`: slug/directory name (e.g., "coloree-review")
- `model`: model name that performed the review (CRITICAL: use "model" parameter, NOT "engine")
- `content`: the peer review content to save

Usage examples:
- `save_review coloree-review glm-4.6 "Review content here..."`
- `save_review coloree-review claude "Review content here..."`
- `save_review coloree-review gemini-3 "Review content here..."`
- `save_review coloree-review sonnet "Review content here..."`

Simple format:
1) First argument: title (slug)
2) Second argument: model name (e.g., "gemini-3", "sonnet", "claude", "glm-4.6")
3) Third argument: content (quoted text)

Steps:
1) Parse the three arguments: title, model, content
2) IMPORTANT: Prepare arguments object with:
   - `title`: the slug
   - `model`: the model name (use "model" parameter, NOT "engine")
   - `content`: the review content
3) Invoke MCP tool `tools.council.save_review` with those arguments
4) Return the tool result directly (do not summarize or trim)

CRITICAL: Always use the "model" parameter in the arguments object. Do NOT use "engine" parameter. The file will be saved as `peer-review-by-{model}.md`.