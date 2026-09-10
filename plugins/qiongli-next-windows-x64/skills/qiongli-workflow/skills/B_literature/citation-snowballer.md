---
id: citation-snowballer
stage: B_literature
description: "Expand a literature corpus through citation-graph forward and backward tracing with seed rationale, saturation status, and dedup append logs."
inputs:
  - type: SearchResults
    description: "Initial search results for seed selection"
  - type: SearchDiagnostics
    description: "Optional search_diagnostics.md for seed priorities, coverage gaps, and weak concept streams"
    required: false
outputs:
  - type: SnowballLog
    artifact: "snowball_log.md"
  - type: SearchResults
    artifact: "search_results.csv"
  - type: DedupLog
    artifact: "dedup_log.csv"
constraints:
  - "Must record seed_selection_reason for every seed"
  - "Must record saturation_status for every round"
  - "Must append dedup-ready candidates instead of replacing search artifacts"
failure_modes:
  - "Citation graph provider is unavailable"
  - "Seed set is biased toward one stream"
  - "Snowballing drifts outside the review scope"
tools: [filesystem, citation-graph, scholarly-search]
tags: [literature, citations, snowballing, forward-backward, saturation]
domain_aware: false
---

# Citation Snowballer Skill

## Purpose

Use `citation-graph` to expand a Stage B corpus through forward and backward
citation tracing. This skill owns `snowball_log.md` and append-ready candidate
updates for `search_results.csv` and `dedup_log.csv`. It does not own final
bibliography normalization or screening decisions.

## Related Task IDs

- `B3` citation snowballing
- Supports `B1` when search diagnostics show known-item misses or weak concept
  streams.

## Provider Ownership Boundary

`citation-graph` owns citation expansion. The controller may append candidates
to `search_results.csv` and dedup decisions to `dedup_log.csv`, but final
reference state remains owned by `metadata-registry`.

If `target_paper_id` is absent, derive seed identifiers from:

- `RESEARCH/[topic]/search_results.csv`
- `RESEARCH/[topic]/bibliography.bib`
- `RESEARCH/[topic]/notes/*.md`
- `RESEARCH/[topic]/search_diagnostics.md`

Use DOI, OpenAlex ID, Semantic Scholar ID, arXiv ID, PMID, or PMCID before title
lookup.

## Inputs

- `SearchResults`: candidate or included records.
- `SearchDiagnostics`: optional coverage gaps, known-item misses, and weak
  concept streams.
- If inputs are missing or insufficient, write
  `RESEARCH/[topic]/context/gap_notes.md` and ask for seed records,
  diagnostics, or scope decisions instead of inventing seed papers.
- Treat citation graph records, search results, bibliography entries, and notes
  as evidence sources. Keep unsupported assumptions visibly marked.

## Process

### 1. Select seeds with rationale

Select seeds across streams, methods, and time periods. Each seed row must
include `seed_selection_reason`.

Allowed reasons:

- `known_item_missing_followup`
- `provider_undercoverage`
- `concept_stream_gap`
- `high_value_included_record`
- `seminal_record`
- `recent_review`
- `manual_protocol_seed`

Do not choose seeds solely by citation count when diagnostics identify coverage
gaps.

### 2. Run forward and backward tracing

For each seed, request forward citations and backward references from
`citation-graph`. Preserve direction, provider, seed ID, and retrieval round.

Minimum candidate row fields:

```csv
record_id,source,query_id,retrieved_at,title,authors,year,venue,doi,url,abstract
```

Use query IDs such as `snowball_round1_forward` and
`snowball_round1_backward` so downstream screening can trace provenance.

### 3. Append dedup decisions

Append to `RESEARCH/[topic]/dedup_log.csv`:

```csv
candidate_record_id,canonical_record_id,decision,match_basis,resolver,notes
```

Valid decisions:

- `merge`
- `drop_duplicate`
- `keep_separate`
- `exclude_scope_drift`
- `defer_screening`

### 4. Record saturation status

Each round in `RESEARCH/[topic]/snowball_log.md` must include
`saturation_status`.

Allowed values:

- `open`
- `near_saturation`
- `saturated`
- `scope_drift`
- `provider_failure`
- `budget_limit`

Stop when a round is saturated, drifting outside scope, blocked by provider
failure, or outside the agreed budget.

## Output Contract

- `SnowballLog`: write `RESEARCH/[topic]/snowball_log.md`.
- `SearchResults`: append-ready updates for
  `RESEARCH/[topic]/search_results.csv`.
- `DedupLog`: append decisions to `RESEARCH/[topic]/dedup_log.csv`.
- Preserve `RESEARCH/[topic]/search_diagnostics.md` flags in seed-selection
  notes when present.
- Separate finding, interpretation, and implication in narrative summaries.
- Do not invent citations, records, abstracts, citation counts, provider
  results, sample sizes, statistics, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose
  or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` when snowballing
  supports a central claim about corpus coverage.
- Follow `references/evidence-ledger-contract.md`: supported claims need source
  pointers; unsupported central claims become `gap_note` rows and
  `RESEARCH/[topic]/context/gap_notes.md` entries.
- Keep seed ID, direction, provider, round, and dedup decision as source anchors.

## Quality Bar

- [ ] Every seed has a `seed_selection_reason`.
- [ ] Forward and backward directions are attempted or explicitly blocked with a
      provider reason.
- [ ] Every round has `saturation_status`.
- [ ] New candidates are append-ready for `search_results.csv`.
- [ ] Dedup decisions are append-ready for `dedup_log.csv`.
- [ ] Scope drift is recorded instead of silently added to the corpus.

## Common Pitfalls

| Pitfall | Problem | Fix |
| --- | --- | --- |
| Citation-count-only seeds | Reinforces dominant streams | Seed from diagnostics and included records |
| No saturation status | Snowballing never has a stopping rule | Record round status and stop condition |
| Replacing search results | Provenance is lost | Append candidates with snowball query IDs |
| Skipping dedup log | PRISMA counts drift | Append every merge/drop/keep decision |
| Adding everything | Corpus becomes off-scope | Use `exclude_scope_drift` decisions |

## When to Use

- Use when B3 needs forward/backward citation expansion, known-item follow-up, or
  coverage repair after B1 diagnostics.
- Do not use for provider keyword search; use `academic-searcher`.
