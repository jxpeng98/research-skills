# 🚨 Troubleshooting & Unified Error Codes Guide

This document lists all standard `ERR-RS-*` error codes you might encounter while using the Research Skills Orchestrator CLI, along with their root causes and standard solutions.

> **Pro Tip:** If you encounter any unexpected behavior, your first step should always be to run the interactive doctor: 
> `python3 -m bridges.orchestrator doctor --cwd .`

---

## Environment & Authentication (ENV)

These errors occur when the Orchestrator cannot locate a required CLI binary (like `claude` or `codex`) or the corresponding API keys in your environment.

### `[ERR-RS-ENV-001]` Missing or invalid API key.
- **Cause**: An active AI agent was requested, but its authentication key is missing from the environment.
- **Fix**: Check your `.env` file or export the keys directly on your terminal.
  - Claude: `export ANTHROPIC_API_KEY="sk-ant-..."`
  - Codex: `export OPENAI_API_KEY="sk-proj-..."`
  - Gemini: `export GOOGLE_API_KEY="AIza..."`

### `[ERR-RS-ENV-002]` Required CLI tool is not installed or not in PATH.
- **Cause**: The Node.js binary wrappers for the models are not installed.
- **Fix**: Install the underlying CLI tools globally:
  - `npm install -g @anthropic-ai/claude-code`

---

## Configuration & Standards (CFG)

These errors occur when the YAML contracts or JSON config maps contain invalid settings, or you requested a task/profile that cannot be mapped.

### `[ERR-RS-CFG-001]` Unknown or invalid agent profile specified.
- **Cause**: You passed a `--profile` (e.g., `--profile fast-writer`) that does not exist in `standards/agent-profiles.example.json`.
- **Fix**: Check your spelling. Use `task-run --help` or view the JSON file to see acceptable profiles (e.g., `default`, `academic-strict`, `bilingual-collaborator`).

### `[ERR-RS-CFG-002]` Could not read or parse standard YAML contracts.
- **Cause**: The `standards/` directory is missing critical standard files (like `research-workflow-contract.yaml`), or the YAML syntax is broken.
- **Fix**: Restore the `.yaml` files from version control. Run `python3 scripts/validate_research_standard.py --strict` to pinpoint YAML syntax errors.

### `[ERR-RS-CFG-003]` Unknown Task ID or invalid phase logic requested.
- **Cause**: You tried to run `rs task-run --task-id X99`, but `X99` does not exist in the platform mapping.
- **Fix**: Refer to `standards/research-workflow-contract.yaml` to see all valid tasks (A1-I8).

---

## Execution & Orchestration (EXE)

These errors happen during the actual generation and agent runtime phase.

### `[ERR-RS-EXE-001]` All required parallel agents failed to execute.
- **Cause**: You requested a `parallel` execution, but all agents crashed or returned safety constraint violations immediately.
- **Fix**: Check the agent logs in `.agent/logs`. This usually happens if the `<cwd>` directory is completely empty and the agent lacks context, or if API rate limits were exhausted simultaneously.

### `[ERR-RS-EXE-002]` Subprocess execution timed out.
- **Cause**: A task exceeded the hardcoded maximum wait time (usually several minutes).
- **Fix**: The scope of the work is too large for one prompt. Break down your `task.md` into smaller sub-tasks.

---

## Model Context Protocol (MCP)

These errors specifically relate to the tools the AI uses to interact with your file system, search engines, or codebase.

### `[ERR-RS-MCP-001]` Required MCP provider is not configured.
- **Cause**: A task (like `scholarly-search`) explicitly demands an MCP tool that you have not configured into the environment. 
- **Fix**: Review `.env.example`. Set the appropriate environment variable (e.g., `RESEARCH_MCP_METADATA_REGISTRY_CMD="..."`). Alternatively, run without `--mcp-strict`.

### `[ERR-RS-MCP-002]` MCP provider command failed to execute or returned an error.
- **Cause**: The MCP server you configured crashed, returned non-JSON output, or timed out.
- **Fix**: If using Zotero MCP or other community MCPs, ensure that node server is actually running and your local API keys are valid. test the MCP independently using `doctor`.
