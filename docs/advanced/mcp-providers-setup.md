# 🔌 Optional MCP Providers Setup Guide

After running `rsk upgrade`, you may see warnings like:

```
⚠  MCP metadata-registry: RESEARCH_MCP_METADATA_REGISTRY_CMD not configured
⚠  MCP fulltext-retrieval: RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD not configured
...
```

**These ⚠ are informational only — they do not affect the framework's core functionality.** These MCPs (Model Context Protocol tools) are optional external capability slots. This document explains what each one does, where to find implementations, and how to connect them.

---

## How It Works

Each MCP maps to an environment variable (`RESEARCH_MCP_<NAME>_CMD`).  
When executing a task, the system spawns a subprocess from this command, pipes in a JSON payload, and reads the JSON response from stdout.

**Resolution priority (highest to lowest):**
1. Environment variable → uses your specified external command
2. Built-in Python script (`scripts/mcp_<name>.py`) → auto-discovered, no config needed
3. Neither → shows ⚠ warning; task still runs without external tool assistance

---

## Provider Reference

### 1. `metadata-registry` — Bibliographic Metadata Normalization

**Purpose:** Enrich and normalize paper DOI, journal, year, and author metadata.  
**Used in:** Stage B (literature processing), task C1, etc.

**Recommended tools:**

| Tool | Type | Link |
|------|------|------|
| OpenAlex MCP | Open-source Python | [github.com/b-vitamins/openalex-mcp](https://github.com/b-vitamins/openalex-mcp) |
| Semantic Scholar (built-in) | Already built-in, no config needed | — |

```bash
# Example: connect OpenAlex MCP
pip install openalex-mcp
export RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
```

---

### 2. `fulltext-retrieval` — Full-Text Paper Acquisition

**Purpose:** Resolve and retrieve full PDF text, track version provenance.  
**Used in:** B1 (systematic review), B2 (full-text extraction).

**Recommended tools:**

| Tool | Type | Link |
|------|------|------|
| Zotero MCP Server | Node.js, connects to local Zotero library | [github.com/zcaceres/zotero-mcp](https://github.com/zcaceres/zotero-mcp) |
| Unpaywall API wrapper | Retrieves open-access full text | Custom script via `api.unpaywall.org` |

```bash
# Connect Zotero MCP (requires Zotero desktop app running)
npm install -g @zcaceres/zotero-mcp-server
export RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
```

> **See also:** Detailed Zotero setup in [`mcp-zotero-integration.md`](./mcp-zotero-integration.md).

---

### 3. `screening-tracker` — Literature Screening State Tracker

**Purpose:** Track inclusion/exclusion decisions for each record in a PRISMA screening workflow.  
**Used in:** B1 systematic review tasks.

**Recommended tools:**

| Tool | Type | Link |
|------|------|------|
| Rayyan MCP (experimental) | Connects to Rayyan platform API | See [rayyan.ai](https://www.rayyan.ai) developer docs |
| Custom SQLite tracker | Local file | See "Building a Stub" section below |

Rayyan is the most popular systematic review screening platform (supports multi-user blinded screening). If no MCP exists yet, use a lightweight stub script as a placeholder (see below).

---

### 4. `extraction-store` — Structured Data Extraction Store

**Purpose:** Store structured study attributes, effect sizes, and outcome data extracted from papers.  
**Used in:** B2/B6/E series (data extraction and analysis).

**Recommended tools:**

| Tool | Type | Link |
|------|------|------|
| Covidence MCP (planned) | Systematic review management platform | [covidence.org](https://www.covidence.org) |
| Local CSV/SQLite stub | Lightweight implementation | See "Building a Stub" section below |

No mature open-source extraction-store MCP currently exists. Use a stub (see below) to silently bypass this module.

---

### 5. `stats-engine` — Statistical Analysis Engine

**Purpose:** Run meta-analyses, mixed-effects models, Bayesian inference, and other statistical computations.  
**Used in:** C3, E3_5, I series (statistics and code) tasks.

**Recommended tools:**

| Tool | Type | Link |
|------|------|------|
| R MCP Server | Uses `metafor`/`brms` R packages | [github.com/holtzy/R-MCP](https://github.com/holtzy/R-MCP) (reference) |
| Python statistics MCP | Uses `statsmodels`/`pymc` | See "Building a Stub" section below |

```bash
# Connect an R-based stats engine
export RESEARCH_MCP_STATS_ENGINE_CMD="Rscript /path/to/stats_mcp.R"

# Or a Python-based one
export RESEARCH_MCP_STATS_ENGINE_CMD="python3 /path/to/stats_mcp.py"
```

---

### 6. `code-runtime` — Code Execution Runtime

**Purpose:** Safely execute generated research code (Python/R) in a sandbox and return results.  
**Used in:** I series (research code) tasks.

**Recommended tools:**

| Tool | Type | Link |
|------|------|------|
| Jupyter MCP Server | Executes via Jupyter HTTP kernel | [github.com/datalayer/jupyter-mcp-server](https://github.com/datalayer/jupyter-mcp-server) |
| Modal.com MCP | Serverless cloud code execution | [modal.com/docs](https://modal.com/docs) |

```bash
pip install jupyter-mcp-server
export RESEARCH_MCP_CODE_RUNTIME_CMD="python3 -m jupyter_mcp_server"
```

---

### 7. `reporting-guidelines` — Reporting Guideline Lookup

**Purpose:** Query and validate completeness of CONSORT, PRISMA, STROBE, CHEERS and other reporting standards.  
**Used in:** G/H series (compliance and submission) tasks.

**Recommended tools:**

| Tool | Type | Link |
|------|------|------|
| EQUATOR MCP (planned) | Connects to EQUATOR Network database | [equator-network.org](https://www.equator-network.org) |
| Local guideline YAML stub | Embeds checklist items locally | See "Building a Stub" section below |

No official EQUATOR MCP is published yet. The `skills/G_compliance/reporting-checker.md` skill card already embeds core guideline checklists, so **skipping this MCP does not affect compliance checking functionality**.

---

### 8. `submission-kit` — Submission Package Manager

**Purpose:** Manage journal submission materials: cover letters, CRediT author statements, supplementary file inventories.  
**Used in:** H1 (submission preparation) tasks.

**Recommended tools:**

| Tool | Type | Link |
|------|------|------|
| Open Journal Systems MCP | Connects to OJS API | See [pkp.sfu.ca/ojs](https://pkp.sfu.ca/ojs/) |
| Overleaf MCP | Connects to Overleaf projects | Experimental, see [overleaf.com/devs](https://www.overleaf.com/devs) |

For most users this MCP has low priority — the `submission-packager` skill card already generates complete submission packages.

---

## Configuration

### Option A: `.env` File (Recommended — project-scoped)

Create `.env` in your project root (copy from `.env.example`):

```bash
# Uncomment and fill in as needed

# RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
# RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
# RESEARCH_MCP_SCREENING_TRACKER_CMD="python3 /path/to/screening_stub.py"
# RESEARCH_MCP_EXTRACTION_STORE_CMD="python3 /path/to/extraction_stub.py"
# RESEARCH_MCP_STATS_ENGINE_CMD="python3 /path/to/stats_mcp.py"
# RESEARCH_MCP_CODE_RUNTIME_CMD="python3 -m jupyter_mcp_server"
# RESEARCH_MCP_REPORTING_GUIDELINES_CMD="python3 /path/to/reporting_stub.py"
# RESEARCH_MCP_SUBMISSION_KIT_CMD="python3 /path/to/submission_stub.py"
```

### Option B: Shell Environment Variables (Global)

```bash
# Add to ~/.zshrc or ~/.bashrc
export RESEARCH_MCP_STATS_ENGINE_CMD="python3 /path/to/stats_mcp.py"
```

---

## Building a Stub (Simplest way to silence ⚠ warnings)

If you don't need a provider's capabilities right now but want to silence the warning, drop a stub script into `scripts/`. The system **auto-discovers** any file named `scripts/mcp_<name>.py` (replacing hyphens with underscores) — no environment variable needed.

```python
# scripts/mcp_screening_tracker.py  (example stub for screening-tracker)
import json, sys

payload = json.loads(sys.stdin.read())
print(json.dumps({
    "status": "ok",
    "summary": "screening-tracker stub: no external tracker configured.",
    "provenance": [],
    "data": {}
}))
```

**Stub filename reference:**

| MCP Name | Stub filename |
|---|---|
| `screening-tracker` | `scripts/mcp_screening_tracker.py` |
| `extraction-store` | `scripts/mcp_extraction_store.py` |
| `stats-engine` | `scripts/mcp_stats_engine.py` |
| `code-runtime` | `scripts/mcp_code_runtime.py` |
| `reporting-guidelines` | `scripts/mcp_reporting_guidelines.py` |
| `submission-kit` | `scripts/mcp_submission_kit.py` |
| `metadata-registry` | `scripts/mcp_metadata_registry.py` |
| `fulltext-retrieval` | `scripts/mcp_fulltext_retrieval.py` |

---

## Verifying Your Setup

After configuration, run the doctor command:

```bash
python3 -m bridges.orchestrator doctor --cwd .
```

`[OK]` means the provider is successfully connected. `[WARNING]` means it is still unconfigured (but the framework will continue running without it).
