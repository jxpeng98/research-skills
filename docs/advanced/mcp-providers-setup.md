# 🔌 Optional MCP Providers Setup Guide

After running `rsk upgrade`, you may see warnings like:

```
⚠  MCP screening-tracker: RESEARCH_MCP_SCREENING_TRACKER_CMD not configured
⚠  MCP extraction-store: RESEARCH_MCP_EXTRACTION_STORE_CMD not configured
...
```

**These ⚠ are informational only — they do not affect the framework's core functionality.** Some MCPs in this repo are built-in reference providers, while others are optional external capability slots. This document explains what each one does, where to find implementations, and how to connect them.

If your goal is a review-grade or highly reproducible academic literature search stack, read [Rigorous Literature Search](/advanced/rigorous-literature-search) after this page. This page explains provider wiring; the other guide explains which search layers to combine.

---

## How It Works

Each MCP maps to an environment variable (`RESEARCH_MCP_<NAME>_CMD`).  
When executing a task, the system spawns a subprocess from this command, pipes in a JSON payload, and reads the JSON response from stdout.

There are three execution patterns in this repo:

1. **Full external override**
   Set `RESEARCH_MCP_<NAME>_CMD` when you want one external command to fully replace the builtin/provider slot.
2. **Builtin baseline + external overlay**
   For selected literature MCPs, keep the builtin provider and layer external enrichment or resolution on top:
   - `RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD`
   - `RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD`
3. **Builtin or auto-discovered stub fallback**
   If no environment variable is set, the runtime will use `scripts/mcp_<name>.py` when present. If neither an env var nor a builtin/stub script exists, you get a ⚠ warning and the task continues with reduced assistance.

## MCP Capability Matrix

Use this table first. It tells you whether each MCP already has a meaningful builtin baseline, when an external provider is worth wiring, and when a stub is enough.

| MCP | Builtin baseline in repo | Recommended external pattern | Configure external when | Stub is enough when | Primary env knobs |
|---|---|---|---|---|---|
| `metadata-registry` | Yes. Local normalization, merge, citekey generation, local artifact ingest. | Usually **overlay**, not replace. | You want authoritative enrichment from OpenAlex/Crossref on top of local reference state. | Rarely needed; builtin is already useful. | `RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD`, `RESEARCH_MCP_METADATA_REGISTRY_CMD` |
| `fulltext-retrieval` | Yes, but planning-only. Drafts manifest + full-text tracking. | Usually **overlay resolver**, not replace. | You want actual PDF/full-text resolution rather than only manifest planning. | Acceptable only if planning/audit is enough for now. | `RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD`, `RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD` |
| `screening-tracker` | No builtin provider. | Direct external MCP or local stub. | You need PRISMA decision persistence, dual screening, or blinded reviewer workflows. | Fine for solo/non-review work or when screening happens outside the repo. | `RESEARCH_MCP_SCREENING_TRACKER_CMD` |
| `extraction-store` | No builtin provider. | Direct external MCP or local stub. | You need structured extraction storage shared across B/E tasks or reviewers. | Fine when extraction lives in markdown/CSV artifacts only. | `RESEARCH_MCP_EXTRACTION_STORE_CMD` |
| `stats-engine` | No builtin provider. | Direct external MCP. | You need real model execution, meta-analysis, Bayesian computation, or numeric diagnostics. | Only if you are drafting plans/specs without executing models yet. | `RESEARCH_MCP_STATS_ENGINE_CMD` |
| `code-runtime` | No builtin provider. | Direct external MCP. | You need sandboxed Python/R execution instead of planning-only code generation. | Only if the task is limited to design/specification or offline execution outside this framework. | `RESEARCH_MCP_CODE_RUNTIME_CMD` |
| `reporting-guidelines` | No builtin MCP, but strong skill-level fallback via `reporting-checker`. | Direct external MCP or local checklist stub. | You want externalized guideline lookup, richer checklist coverage, or centralized audit outside the skill cards. | Usually yes, because core guideline logic already exists in repo skills. | `RESEARCH_MCP_REPORTING_GUIDELINES_CMD` |
| `submission-kit` | No builtin MCP, but strong skill-level fallback via `submission-packager`. | Direct external MCP. | You need downstream journal-system or Overleaf integration, not just file generation. | Usually yes, if local artifact generation is enough. | `RESEARCH_MCP_SUBMISSION_KIT_CMD` |

## Quick Decision Rules

- Choose **builtin only** when the repo already produces the artifacts you need and you mainly care about contract completeness.
- Choose **builtin + overlay** for `metadata-registry` and `fulltext-retrieval` when local artifact ownership should stay in-repo but authority should come from an external resolver or enrichment source.
- Choose **full external override** only when your external MCP owns the slot better than the builtin implementation and you are comfortable replacing the default behavior.
- Choose a **thin local stub** when you only want to silence warnings or satisfy orchestration contracts for a capability you are not actively using yet.

---

## Provider Reference

### 1. `metadata-registry` — Bibliographic Metadata Normalization

**Purpose:** Enrich and normalize paper DOI, journal, year, and author metadata.  
**Used in:** Stage B (literature processing), task C1, etc.

This provider now has a built-in local reference implementation for identifier normalization, local record merge, and citekey generation. It can ingest `bibliography.bib`, `references.json`, `references.ris`, `search_results.csv`, and `notes/*.md`. For authoritative enrichment, prefer wiring an overlay command such as OpenAlex on top of the builtin provider.

Current enrichment merge policy is source-aware rather than last-write-wins:
- `OpenAlex` is preferred for title, author list, venue, publisher, and OA link enrichment
- `Crossref` is preferred for volume / issue / page metadata and DOI landing-page normalization
- locally established `doi` and `citekey` values remain sticky unless they are missing

**Recommended tools:**

| Tool | Type | Link |
|------|------|------|
| Built-in local reference provider | Included in this repo | `scripts/mcp_metadata_registry.py` |
| OpenAlex MCP | Open-source Python | [github.com/b-vitamins/openalex-mcp](https://github.com/b-vitamins/openalex-mcp) |

```bash
# Example: keep builtin metadata-registry, add OpenAlex enrichment overlay
pip install openalex-mcp
export RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD="python3 -m openalex_mcp"
```

Set `RESEARCH_MCP_METADATA_REGISTRY_CMD` only if you want to replace the builtin metadata-registry completely.

---

### 2. `fulltext-retrieval` — Full-Text Paper Acquisition

**Purpose:** Resolve and retrieve full PDF text, track version provenance.  
**Used in:** B1 (systematic review), B2 (full-text extraction).

This provider now has a built-in retrieval-planning stub. The builtin mode does not download PDFs, but it can draft `retrieval_manifest.csv` and `screening/full_text.md` from local literature artifacts, preserve existing manifest rows, verify referenced local files, and flag OA/manual follow-up candidates.

Resolver handoff should follow a layered contract:
- keep the builtin stub as the planning baseline
- use `RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD` when you want an external resolver to update manifest rows with actual retrieval outcomes
- use `RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD` only when you want to replace the builtin provider entirely

Current resolver merge policy is source-aware rather than last-write-wins:
- resolver-backed `retrieved_*` statuses outrank builtin `not_retrieved:*` planning states
- resolver `fulltext_path`, `license`, and `version_label` values replace builtin placeholders when the resolver has equal or higher priority
- builtin planning notes are preserved and resolver notes are appended for auditability

**Recommended tools:**

| Tool | Type | Link |
|------|------|------|
| Built-in retrieval-planning stub | Included in this repo | `scripts/mcp_fulltext_retrieval.py` |
| Zotero MCP Server | Node.js, connects to local Zotero library | [github.com/zcaceres/zotero-mcp](https://github.com/zcaceres/zotero-mcp) |
| Unpaywall API wrapper | Retrieves open-access full text | Custom script via `api.unpaywall.org` |

```bash
# Optional: keep builtin planning and layer Zotero resolution on top
npm install -g @zcaceres/zotero-mcp-server
export RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD="npx -y @zcaceres/zotero-mcp-server"
```

Set `RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD` only when you want to replace the builtin planning stub with a fully external provider.

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
# RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD="python3 -m openalex_mcp"
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
