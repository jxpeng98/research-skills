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

- `scholarly-search` → built-in Semantic Scholar API adapter with query variants, normalized rows, and baseline dedup
- `citation-graph` → built-in Semantic Scholar citation / reference adapter with local seed extraction from `search_results.csv`, `bibliography.bib`, and `notes/`
- `metadata-registry` → built-in local reference provider for identifier normalization, local record merge, and citekey generation
- `fulltext-retrieval` → built-in retrieval-manifest stub that drafts `retrieval_manifest.csv` and `screening/full_text.md` from local literature artifacts

The other layers are external-provider slots:

- `screening-tracker`
- `extraction-store`

So the strictest practical baseline today is:

- built-in Semantic Scholar for discovery
- built-in metadata-registry for local normalization, plus OpenAlex MCP for authoritative enrichment
- built-in citation graph for snowballing
- built-in fulltext planning stub, plus Zotero / OA resolver for actual full-text downloads

## Standard Literature Bundle

When literature workflows are behaving correctly, the shared bundle should converge on:

- `search_strategy.md`
- `search_log.md`
- `search_results.csv`
- `dedup_log.csv`
- `snowball_log.md`
- `bibliography.bib`
- `screening/full_text.md`
- `retrieval_manifest.csv`

## Configuration Matrix

Use this table to decide what you actually need to configure:

| Layer | Works with zero config | Needs API key | Needs `RESEARCH_MCP_*_CMD` | Notes |
|---|---|---|---|---|
| `scholarly-search` | yes, via built-in Semantic Scholar | recommended | optional | zero-config works, produces query variants + dedup-ready rows, but rate limiting is more likely without a key |
| `citation-graph` | yes, via built-in Semantic Scholar graph | no | optional | useful for snowballing even before you add external MCPs; builtin mode can resolve seeds from local artifacts |
| `metadata-registry` | yes, via built-in local reference provider | no for local mode | optional | built-in mode can merge local references from BibTeX, RIS, CSL-JSON, notes, and search results; connect OpenAlex or another metadata MCP for authoritative enrichment |
| `fulltext-retrieval` | yes, via built-in retrieval-manifest stub | no for stub mode | optional, but recommended for real downloads | builtin mode drafts retrieval status + provenance rows; connect Zotero or another resolver for actual PDF/full-text acquisition |
| `screening-tracker` | no | depends on provider | yes | mostly relevant for systematic review workflows |
| `extraction-store` | no | depends on provider | yes | mostly relevant for systematic review workflows |

## What Happens If You Configure Nothing

If you configure nothing at all:

- `scholarly-search` still attempts to use the built-in Semantic Scholar adapter
- `citation-graph` still attempts to use the built-in Semantic Scholar graph adapter
- `metadata-registry` still attempts to use the built-in local reference provider
- `fulltext-retrieval` still prepares a local retrieval manifest and screening draft
- tasks can still run unless you explicitly require strict MCP enforcement

This is enough for exploratory work, but not ideal for rigorous review-grade search.

Important: `bibliography.bib` is the canonical export in this repo, but it is not the only supported working source. The built-in metadata layer can ingest `references.json`, `references.ris`, `search_results.csv`, and `notes/` even if the user does not actively manage BibTeX.

## Step-by-Step Setup

### Option A. Zero-Config Start

Do nothing. Use the built-in providers and accept that recall, metadata quality, and rate-limit stability are limited.

The built-in `scholarly-search` baseline now still gives you:

- multiple query variants derived from topic/question/keywords
- normalized `search_results` rows
- a machine-readable `dedup_log`
- per-query `search_log` execution entries

### Option B. Recommended Lightweight Setup

Create `.env` in your project root:

```env
SEMANTIC_SCHOLAR_API_KEY="your-semantic-scholar-key"
RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD="python3 -m openalex_mcp"
```

This is the best default for most users who want stricter search without building custom infrastructure.

If you also want local full-text planning without wiring a resolver yet, the built-in `fulltext-retrieval` stub already prepares:

- `retrieval_manifest.csv` draft rows
- `screening/full_text.md` draft rows
- `not_retrieved:oa_candidate` / `not_retrieved:needs_provider` / `not_retrieved:missing_locator` status hints

### Option C. Review-Grade Multi-Source Setup

Use your own scholarly MCP for merged discovery, then keep metadata and full-text as separate layers:

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="python3 /path/to/multi_source_search_mcp.py"
RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD="python3 -m openalex_mcp"
RESEARCH_MCP_CITATION_GRAPH_CMD="python3 /path/to/graph_mcp.py"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD="npx -y @zcaceres/zotero-mcp-server"
```

Use this when you need repeatable search logs, deduped merged candidates, and stronger provenance.

### Option D. Local-Library Controlled Setup

Route both discovery and full text through your local Zotero-backed corpus:

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD="python3 -m openalex_mcp"
```

Use this when your search space must stay inside a curated local library.

## Strict Mode

If you run tasks with `--mcp-strict`, every required external provider must actually be configured. In practice this means:

- built-in `scholarly-search` and `citation-graph` can still satisfy those layers if you do not override them
- built-in `metadata-registry` can satisfy the local normalization layer without extra config
- built-in `fulltext-retrieval` can satisfy the retrieval-planning layer without extra config
- set `RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD` when you want authoritative external enrichment on top of the builtin reference mode
- set `RESEARCH_MCP_METADATA_REGISTRY_CMD` only when you want a full external override
- set `RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD` when you want resolver-backed downloads layered on top of the builtin planning stub
- set `RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD` only when you want a full external override

## Recommended Search Stacks

### 1. Fast Baseline

Use this when you want better rigor without extra engineering:

```env
SEMANTIC_SCHOLAR_API_KEY="your-semantic-scholar-key"
RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD="python3 -m openalex_mcp"
```

This gives you:

- built-in Semantic Scholar retrieval
- fewer 429 failures if you have an API key
- OpenAlex normalization for DOI, venue, and author cleanup

### 2. Review-Grade Stack

Use this for systematic reviews, structured related-work chapters, or any project where reproducibility matters:

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="python3 /path/to/multi_source_search_mcp.py"
RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD="python3 -m openalex_mcp"
RESEARCH_MCP_CITATION_GRAPH_CMD="python3 /path/to/graph_mcp.py"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD="npx -y @zcaceres/zotero-mcp-server"
```

Recommended responsibilities:

- `scholarly-search`: query multiple scholarly indexes and return a merged candidate set
- `metadata-registry`: canonical DOI and venue normalization
- `citation-graph`: structured backward / forward citation expansion
- `fulltext-retrieval`: full text plus provenance

### 3. Local-Library Controlled Search

Use this when your review must stay inside a curated local corpus:

```env
RESEARCH_MCP_SCHOLARLY_SEARCH_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD="npx -y @zcaceres/zotero-mcp-server"
RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD="python3 -m openalex_mcp"
```

This is narrower, but often more auditable.

## Recommended Query Workflow

1. Write one canonical research question plus 2-4 query variants.
2. Run each query across at least two discovery sources.
3. Normalize and deduplicate records before screening.
4. Log source, date, query string, and result count.
5. Seed a citation pass from the highest-relevance papers.
6. Only then move to screening and extraction.

Artifacts to keep under `RESEARCH/[topic]/`:

- `search_strategy.md`
- `search_log.md`
- `bibliography.bib`
- `screening_decisions.csv`
- `retrieval_manifest.csv`

## Engine Roles

Use engines by role instead of assuming one engine can do everything:

| Engine | Best use | Notes |
|---|---|---|
| Semantic Scholar | fast discovery, relevance scan, citation counts | built in; may rate-limit without an API key |
| OpenAlex | metadata normalization, entity graph, venue / author cleanup | strong companion to discovery engines |
| Crossref | DOI-focused lookup, reproducible metadata harvesting | useful as a normalization and verification layer |
| Europe PMC / PubMed | biomedical and life-science search | use for domain-specific recall |
| arXiv | CS, physics, math preprints | good for preprint-heavy domains |
| CORE | open-access full text discovery | useful for OA-heavy retrieval |
| Lens | scholarship + patent landscape | often institutionally managed; check access terms |

## Engines Not Recommended as the Primary Reproducible Layer

Google Scholar can still help with manual spot checks, but it should not be your primary automated review-grade source in this stack because query behavior and reproducibility are harder to control.

## Verifying the Setup

After configuration:

```bash
python3 -m bridges.orchestrator doctor --cwd .
```

For the built-in provider specifically, you can smoke-test it with:

```bash
printf '%s' '{"provider":"scholarly-search","task_packet":{"topic":"causal inference in management research"}}' | python3 scripts/mcp_scholarly_search.py
```

If you see `HTTP Error 429`, the provider is reachable but rate-limited. Add `SEMANTIC_SCHOLAR_API_KEY` or switch `scholarly-search` to your own multi-source MCP.
