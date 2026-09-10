---
id: academic-searcher
stage: B_literature
description: "Conduct provider-backed literature searches with reproducible query plans, deduplicated records, and review-grade search diagnostics."
inputs:
  - type: RQSet
    description: "Research questions with key concepts and scope limits"
  - type: ConceptMap
    description: "Optional B1_5 concept buckets, synonyms, and seed recall notes"
    required: false
  - type: SearchQueryPlan
    description: "Optional pre-built search strategy"
    required: false
outputs:
  - type: SearchQueryPlan
    artifact: "search_strategy.md"
  - type: SearchResults
    artifact: "search_results.csv"
  - type: SearchLog
    artifact: "search_log.md"
  - type: DedupLog
    artifact: "dedup_log.csv"
  - type: SearchDiagnostics
    artifact: "search_diagnostics.md"
constraints:
  - "Must route discovery through scholarly-search or a compatible MCP/provider adapter"
  - "Must log exact translated queries, filters, provider names, counts, and timestamps"
  - "Must write search_diagnostics.md before systematic-review or review-grade screening claims"
failure_modes:
  - "Required provider is unavailable or rate-limited"
  - "Known seed item is not recalled by any query"
  - "A required concept block returns zero usable records"
tools: [filesystem, scholarly-search, metadata-registry]
tags: [literature, search, databases, MCP, reproducibility, diagnostics]
domain_aware: false
---

# Academic Searcher Skill

## Purpose

Execute B1 literature discovery as a reproducible MCP/provider workflow. This
workflow/router owns `qiongli_search_plan` creation and `search_execution_mode` selection. This skill validates and records `search_strategy.md`, provider execution trace, dedup-ready candidate records, search logging, normalization, dedupe, and diagnostics. It does not own full-text retrieval, bibliography normalization, citation snowballing, or screening decisions.

## Related Task IDs

- `B1` systematic review pipeline
- Supports `A4`, `A5`, `B1_5`, and `B3` when those tasks need provider-backed
  search evidence.

## Provider Ownership Boundary

Treat `scholarly-search` as the primary discovery layer and execution owner for
discovery. The skill should describe what must be sent through the MCP/provider
layer, which configured scholarly provider overlay is active, and what artifacts
must be recorded after it returns.

The canonical execution path in this repo is the MCP/provider stack.
Hybrid search coordination belongs to the workflow/router layer. Provider layer owns provider calls, active agent owns platform-native search, and this skill owns logging, normalization, dedupe, and diagnostics. MCP servers must not call Codex or Claude native search directly.

| Layer | Owner | Stage B artifact responsibility |
| --- | --- | --- |
| Hybrid router | workflow/router layer | create `qiongli_search_plan`, choose `search_execution_mode`, and declare `native_search_queries` |
| Query execution | `scholarly-search` or compatible MCP/provider | provider calls and raw hit capture |
| Platform-native search | active agent | execute `native_search_queries` for `hybrid_search` or `native_only` and return separately labeled records |
| Record normalization | `scholarly-search` plus local materializer | `search_results.csv` rows |
| Dedup decisions | search materializer / controller | append `dedup_log.csv` |
| Metadata enrichment | `metadata-registry` | final `bibliography.bib`, not this skill |
| Full-text resolution | `fulltext-retrieval` | `retrieval_manifest.csv`, not this skill |

Do not make direct web or API calls the default execution path inside this
skill. Provider mechanics belong in the MCP/provider layer. Manual web checks,
including Google Scholar checks, are supplemental and logged; they are not the
default reproducible pipeline.

The `qiongli_search_plan` contract uses four execution modes:
`hybrid_search`, `provider_connected`, `native_only`, and `strategy_only`.
Record `provider_capability_mode` separately as `provider_connected` or
`strategy_only`; it describes provider availability, not the whole search
execution path. Preserve provenance labels across all outputs:
`mcp:openalex`, `mcp:semantic_scholar`, `mcp:crossref`, `mcp:pubmed`,
`mcp:arxiv`, `native:codex_web_search`, `native:claude_web_search`, and
`user_corpus`.

## Inputs

- `RQSet`: research question, scope, eligibility constraints, and key concepts.
- `ConceptMap`: optional `RESEARCH/[topic]/literature/concept_extraction.md`
  from `concept-extractor`.
- `SearchQueryPlan`: optional existing `RESEARCH/[topic]/search_strategy.md`.
- If concepts, date range, source scope, or eligibility rules are missing, write
  a gap note to `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing
  decision instead of inventing it.
- Treat literature metadata, provider records, citations, and project files as
  evidence sources. Keep unsupported assumptions visibly marked.

## Process

For a bounded reference lookup, use the user's topic and filters, record the
query/mode/provenance, verify and deduplicate the relevant records, and return
the requested set or a clearly stated shortfall. These may be reported in chat
when no saved artifact was requested. Stop at the requested scope; do not start
screening, snowballing, full-text retrieval or a systematic review automatically.
The formal B1 output/diagnostic contract below remains required for a formal
run or review-grade claims. A small lookup must not be labelled exhaustive.

### 1. Build or validate `search_strategy.md`

Write `RESEARCH/[topic]/search_strategy.md` and validate `qiongli_search_plan`
before execution. Minimum content:

| Field | Requirement |
| --- | --- |
| Research scope | RQ, paper type, date range, language, publication type |
| Concept blocks | 2-5 blocks with synonyms and excluded ambiguous terms |
| Provider plan | provider names, query IDs, translated query strings, filters |
| Search execution | `search_execution_mode`, `provider_capability_mode`, and any `native_search_queries` |
| Seed recall | seed DOI/title list and expected query match when seeds exist |
| Dedup policy | DOI first, then provider ID, then title-year-author |
| Review mode | `systematic_review` or `targeted_search` |

Use B1_5 concept extraction when the query vocabulary is unstable. Do not widen
queries silently to hide poor seed recall; record missing seeds as query gaps.

### 2. Execute through the MCP/provider layer

For each query ID and provider:

1. Submit the translated query through `scholarly-search` or a compatible
   provider adapter.
2. Record provider name, query ID, translated query, filters, timestamp,
   interface version when known, and hit counts in `search_log.md`.
3. Normalize retained records into `search_results.csv` with this minimum
   schema:

```csv
record_id,source,query_id,retrieved_at,title,authors,year,venue,doi,url,abstract
```

4. Append merge, duplicate, and keep-separate decisions to `dedup_log.csv`:

```csv
candidate_record_id,canonical_record_id,decision,match_basis,resolver,notes
```

5. Preserve provider failures and rate limits in `search_log.md`; do not erase
   failed providers from the search trace.

When the plan uses `hybrid_search` or `native_only`, ingest native search
records only after the active agent has executed `native_search_queries`. Keep
those records separate from MCP/provider records with
`native:codex_web_search` or `native:claude_web_search` labels. User-supplied
files, bibliographies, pasted citations, or local notes use `user_corpus`.

### 3. Write `search_diagnostics.md`

Use `templates/search-diagnostics.md`. The diagnostics artifact must include:

- `search_mode`: `systematic_review` or `targeted_search`
- provider coverage and failure notes
- query coverage by concept block and query ID
- known-item recall against seed DOI/title declarations
- dedup ratio and unresolved duplicate clusters
- coverage gaps and next search actions
- screening readiness

### 4. Apply blocking conditions

Review-grade and systematic-review work blocks when any condition is true:

- `search_diagnostics.md` is missing.
- Fewer than at least two productive providers are recorded.
- A required known-item remains missing.
- A required concept block has a zero-hit usable result.
- `weak screening readiness` is present and unresolved.

Targeted searches may proceed with one provider or a narrow query, but the
limitation must remain visible in `search_diagnostics.md`, `search_log.md`, and
downstream screening artifacts. Such output cannot be described as exhaustive or
review-grade.

### 5. Handle fallback without hiding limitations

Fallback means choosing another configured provider or recording a manual
supplemental check. It does not mean bypassing the reproducibility contract.

| Situation | Required action |
| --- | --- |
| Provider unavailable | record failure, try another configured provider, keep failure in `search_log.md` |
| Rate limit | record limit, stop or defer provider, do not fabricate counts |
| Google Scholar/manual check | log as supplemental source, record query/date/counts, do not make it the default source |
| Zero results | inspect concept block, mark query gap, ask for scope decision when needed |

## Output Contract

- `SearchQueryPlan`: write `RESEARCH/[topic]/search_strategy.md`.
- `SearchResults`: write `RESEARCH/[topic]/search_results.csv`.
- `SearchLog`: write `RESEARCH/[topic]/search_log.md`.
- `DedupLog`: write `RESEARCH/[topic]/dedup_log.csv`.
- `SearchDiagnostics`: write `RESEARCH/[topic]/search_diagnostics.md`.
- Separate finding, interpretation, and implication in narrative summaries.
- Do not invent citations, sources, datasets, sample sizes, search counts,
  statistics, or provider results.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose
  or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` when search
  outputs support central scholarly claims.
- Follow `references/evidence-ledger-contract.md`: supported claims need source
  pointers; unsupported central claims become `gap_note` rows and
  `RESEARCH/[topic]/context/gap_notes.md` entries.
- Preserve source, query ID, provider, and retrieval timestamp so downstream
  screening, extraction, and synthesis can audit provenance.

## Quality Bar

- [ ] `search_strategy.md` contains exact query strings, filters, and provider
      translations.
- [ ] `search_log.md` records provider names, timestamps, counts, and failures.
- [ ] `search_results.csv` uses stable `record_id` and `query_id` fields.
- [ ] `dedup_log.csv` records one row per merge/drop/keep decision.
- [ ] `search_diagnostics.md` records provider coverage, concept coverage,
      known-item recall, dedup ratio, coverage gaps, and screening readiness.
- [ ] Review-grade claims satisfy the blocking conditions above.

## Common Pitfalls

| Pitfall | Problem | Fix |
| --- | --- | --- |
| Direct provider calls in prose | Agent bypasses MCP/provider audit trail | Route through `scholarly-search` and log provider output |
| Google Scholar as default fallback | Search cannot be reproduced reliably | Use only as supplemental manual evidence |
| Missing diagnostics | Screening inherits unknown coverage risk | Write `search_diagnostics.md` before screening |
| Query broadening without trace | Reviewers cannot see why scope changed | Update `search_strategy.md` and `search_log.md` |
| Dedup only in memory | PRISMA counts cannot reconcile | Append every decision to `dedup_log.csv` |

## When to Use

- Use for B1 search execution, review-grade search diagnostics, or targeted
  provider-backed literature discovery.
- Do not use for Zotero import/export, full-text retrieval, citation
  snowballing, or screening decisions; use the corresponding B-stage skill.
