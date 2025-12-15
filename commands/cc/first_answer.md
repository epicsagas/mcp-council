---
name: first_answer
version: 0.2.0
description: >
  Save Stage1 model answers from the current conversation into
  .council/{slug}/{model}-answer.md (Markdown per model).
inputs:
  title:
    type: string
    required: true
  model:
    type: string
    required: false
---

You are the "LLM Council Stage1 saver" running inside Cursor.

Goal: Save the current model's answer (or answers from the conversation) as a Markdown file under `.council/{slug}/` **by calling the MCP tool** `council.first_answer`. Do NOT write files yourself; delegate file I/O to the tool.

**CRITICAL**: Even if there are no previous assistant responses in the conversation, you MUST still save the current model's answer if:
1. The user has asked a question in this conversation, OR
2. The user has provided context/files to analyze, OR  
3. You are generating a response right now

The purpose is to capture the current model's Stage1 answer, not to wait for multiple models' answers.

Rules:
1) Slug:
   - lower-case; spaces → "-", keep only [a-z0-9-]
   - example: "High Res Network Player" → "high-res-network-player"
2) Directory: `.council/{slug}` (the MCP tool will create it if missing).
3) Determine `model` from the current tab name (e.g., Sonnet → `sonnet`, GPT-5.1 → `gpt-5-1`, gemini-2.5 → `gemini-2-5`). If unsure, ask the user.
4) Pick the latest user question as `prompt`. If none, use the command context (e.g., the file or topic mentioned in the council command).
5) Generate (or reuse) the full answer text (`content`). Do NOT truncate or summarize.
6) Call the MCP tool:
   ```
   tools.council.first_answer
   - title: {slug from the command input}
   - model: {tab/model name, sanitized to lowercase/hyphen}
   - prompt: {latest user question or context}
   - content: {the full answer you just produced}
   ```
7) The tool handles filenames:
   - Writes `{model}-answer.md`; if it already exists, writes `{model}-answer-YYYYMMDD-HHMMSS.md`.
   - Stores under `.council/{slug}/`.
8) After the tool succeeds, respond with a short summary listing written files and their paths.
