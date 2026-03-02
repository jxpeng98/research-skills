# 📚 External MCP Integration: Zotero (Local Citation Library)

The `research-skills` ecosystem comes with a built-in semantic-search provider that hits the public Semantic Scholar API. This is perfect for discovering global papers. 

However, for a highly rigorous systematic review, you may only want the AI to query your own carefully vetted **local Zotero library**. By connecting an external Zotero MCP, the AI will limit its evidence extraction exclusively to papers you have collected and annotated.

## How it works (The Fallback Chain)

By default, when Claude or Codex reaches the `scholarly-search` stage, it routes the request through our `MCPConnector`.

1. **Check Environment Variable**: It first looks for `RESEARCH_MCP_SCHOLARLY_SEARCH_CMD`.
2. **Fallback to Native**: If not set, it executes our built-in Python script (`scripts/mcp_scholarly_search.py`).

By setting the environment variable, you **override** the default API behavior and map the capability directly to your local instance.

## Step 1: Install a Zotero MCP Server

You need an MCP server that bridges the Model Context Protocol to your local Zotero database. There are several community options available. 

*(Example using typical community Zotero MCP)*:
```bash
# Ensure Node.js is installed
npm install -g @zcaceres/zotero-mcp-server
```

*Note: Follow the specific Zotero MCP's documentation to provide it with your Zotero API Key and User ID if required.*

## Step 2: Configure your Environment

Create a `.env` file in the root of your project (or set it in your universal shell config like `~/.zshrc`).

```env
# Point the scholarly-search capability to your local Zotero MCP node script
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="npx -y @zcaceres/zotero-mcp-server"
```

*The exact command matches whatever you would normally type in the terminal to start the MCP server. Our Orchestrator will spawn this command as a subprocess, pipe JSON into its stdin, and read the JSON response from stdout.*

## Step 3: Verify the Connection

Use the `doctor` command to verify that the Orchestrator successfully detects your custom command hook:

```bash
python3 -m bridges.orchestrator doctor --cwd .
```

In the **Check Details** output, look for the `scholarly-search` provider. It should now reflect your custom `npx` command instead of the default Python implementation.

## Reverting to Output Internet Search

If you want to cast a wider net again and search the whole internet via our default Semantic Scholar integration, simply unset the variable:

```bash
unset RESEARCH_MCP_SCHOLARLY_SEARCH_CMD
```
