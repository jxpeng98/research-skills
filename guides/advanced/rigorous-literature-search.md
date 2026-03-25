# Rigorous Literature Search

Use this guide when you want the `research-skills` literature stack to behave more like a review-grade academic search workflow instead of a single-engine convenience search.

## Core Principle

Treat `scholarly-search`, `metadata-registry`, `citation-graph`, and `fulltext-retrieval` as separate evidence layers:

1. `scholarly-search`: discovery and candidate retrieval
2. `metadata-registry`: DOI, author, venue, year normalization
3. `citation-graph`: backward / forward snowballing
4. `fulltext-retrieval`: PDF or full-text acquisition with provenance

For rigorous work, do not depend on only one engine.

## What Is Built In Today

The current repository ships these built-in literature providers:

- `scholarly-search` → built-in Semantic Scholar API adapter
- `citation-graph` → built-in Semantic Scholar citation / reference adapter

The other layers are external-provider slots:

- `metadata-registry`
- `fulltext-retrieval`
- `screening-tracker`
- `extraction-store`

So the strictest practical baseline today is:

- built-in Semantic Scholar for discovery
- OpenAlex MCP for metadata normalization
- built-in citation graph for snowballing
- Zotero / OA resolver for full text

## Configuration Matrix

| Layer | Works with zero config | Needs API key | Needs `RESEARCH_MCP_*_CMD` | Notes |
|---|---|---|---|---|
| `scholarly-search` | yes | recommended | optional | built-in Semantic Scholar works, but can rate-limit |
| `citation-graph` | yes | no | optional | built-in graph adapter is available |
| `metadata-registry` | no | depends on provider | yes | connect OpenAlex or another metadata MCP |
| `fulltext-retrieval` | no | depends on provider | yes | connect Zotero or another full-text resolver |
| `screening-tracker` | no | depends on provider | yes | systematic review support |
| `extraction-store` | no | depends on provider | yes | systematic review support |

## Step-by-Step Setup

### Option A. Zero-Config Start

Use the built-in providers only.

### Option B. Lightweight Recommended Setup

```env
SEMANTIC_SCHOLAR_API_KEY="your-semantic-scholar-key"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
```

### Option C. Review-Grade Multi-Source Setup

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="python3 /path/to/multi_source_search_mcp.py"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
RESEARCH_MCP_CITATION_GRAPH_CMD="python3 /path/to/graph_mcp.py"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
```

### Option D. Local-Library Controlled Setup

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
```

## Strict Mode

With `--mcp-strict`, unconfigured required providers become blockers. In practice, `metadata-registry` and `fulltext-retrieval` are the first layers that usually need explicit setup.

## Recommended Search Stacks

### 1. Fast Baseline

Use this when you want better rigor without extra engineering:

```env
SEMANTIC_SCHOLAR_API_KEY="your-semantic-scholar-key"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
```

### 2. Review-Grade Stack

Use this for systematic reviews, structured related-work chapters, or any project where reproducibility matters:

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="python3 /path/to/multi_source_search_mcp.py"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
RESEARCH_MCP_CITATION_GRAPH_CMD="python3 /path/to/graph_mcp.py"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
```

### 3. Local-Library Controlled Search

Use this when your review must stay inside a curated local corpus:

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_METADATA_REGISTRY_CMD="python3 -m openalex_mcp"
```

## Recommended Query Workflow

1. Write one canonical research question plus 2-4 query variants.
2. Run each query across at least two discovery sources.
3. Normalize and deduplicate records before screening.
4. Log source, date, query string, and result count.
5. Seed a citation pass from the highest-relevance papers.
6. Only then move to screening and extraction.

Recommended artifacts:

- `search_strategy.md`
- `search_log.md`
- `bibliography.bib`
- `screening_decisions.csv`
- `fulltext_manifest.csv`

## Engine Roles

| Engine | Best use | Notes |
|---|---|---|
| Semantic Scholar | fast discovery, relevance scan, citation counts | built in; may rate-limit without an API key |
| OpenAlex | metadata normalization, entity graph, venue / author cleanup | strong companion to discovery engines |
| Crossref | DOI-focused lookup, reproducible metadata harvesting | useful as a normalization and verification layer |
| Europe PMC / PubMed | biomedical and life-science search | use for domain-specific recall |
| arXiv | CS, physics, math preprints | good for preprint-heavy domains |
| CORE | open-access full text discovery | useful for OA-heavy retrieval |
| Lens | scholarship + patent landscape | often institutionally managed; check access terms |

## Verifying the Setup

```bash
python3 -m bridges.orchestrator doctor --cwd .
printf '%s' '{"provider":"scholarly-search","task_packet":{"topic":"causal inference in management research"}}' | python3 scripts/mcp_scholarly_search.py
```

If the built-in provider returns `HTTP Error 429`, add `SEMANTIC_SCHOLAR_API_KEY` or switch `scholarly-search` to your own multi-source MCP.
