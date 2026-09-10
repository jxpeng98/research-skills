---
id: paper-screener
stage: B_literature
description: "Apply two-stage literature screening with diagnostic-aware PRISMA decision logs."
inputs:
  - type: SearchResults
    description: "Deduplicated or dedup-ready search results to screen"
  - type: RQSet
    description: "Inclusion and exclusion criteria from the research question or protocol"
  - type: SearchDiagnostics
    description: "Optional search_diagnostics.md with search readiness flags and coverage gaps"
    required: false
outputs:
  - type: ScreeningDecisionLog
    artifact: "screening/title_abstract.md"
  - type: FullTextScreening
    artifact: "screening/full_text.md"
  - type: PRISMAFlowData
    artifact: "screening/prisma_flow.md"
constraints:
  - "Must preserve search provenance and diagnostic flags in every screening row"
  - "Must document one specific exclusion reason for every excluded record"
  - "Must not claim review-grade screening when search diagnostics are unresolved"
failure_modes:
  - "Ambiguous eligibility criteria cause inconsistent decisions"
  - "Search diagnostics show unresolved coverage gaps"
  - "Full text is unavailable for a borderline record"
tools: [filesystem, screening-tracker]
tags: [literature, screening, PRISMA, inclusion-exclusion, diagnostics]
domain_aware: false
---

# Paper Screener Skill

## Purpose

Screen candidate literature for B1 using a two-stage, PRISMA-compatible decision
log. This skill owns screening decisions and PRISMA flow counts. It does not own
search execution, full-text retrieval, or extraction.

## Related Task IDs

- `B1` systematic review pipeline
- Supports `B2` when targeted reading needs explicit include/exclude decisions.

## Inputs

- `SearchResults`: `RESEARCH/[topic]/search_results.csv`.
- `RQSet` or protocol criteria: inclusion and exclusion rules.
- `SearchDiagnostics`: `RESEARCH/[topic]/search_diagnostics.md` when produced
  by `academic-searcher`.
- If inputs are missing or insufficient, write
  `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing criteria,
  diagnostics, or candidate records instead of inventing criteria.
- Treat titles, abstracts, full-text evidence, metadata, and diagnostic flags as
  evidence sources. Keep unsupported assumptions visibly marked.

## Process

### 1. Check search readiness before screening

Inspect `search_diagnostics.md` when present.

| Diagnostic state | Screening behavior |
| --- | --- |
| no unresolved flags | proceed with review-grade screening |
| `known_item_missing` | triage may continue; review-grade claim blocks |
| `query_too_narrow` | triage may continue; copy limitation into screening notes |
| `provider_undercoverage` | triage may continue; do not claim exhaustive coverage |
| `weak screening readiness` | block review-grade screening until resolved or protocol-visible limitation is written |

If diagnostics are missing for a systematic-review or review-grade task, block
and request `search_diagnostics.md`. For targeted reading, continue only with a
visible limitation.

### 2. Title/abstract screening

Write `RESEARCH/[topic]/screening/title_abstract.md`.

Each row must preserve:

```markdown
| record_id | query_id | source | title | year | relevance_reason | diagnostic_flags | decision | exclusion_reason | notes |
|---|---|---|---|---|---|---|---|---|---|
```

Allowed decisions:

- `include`
- `exclude`
- `uncertain`

Every `exclude` row requires one specific exclusion reason tied to the protocol.
Do not use vague reasons such as "not relevant" when a criterion-specific reason
is available.

### 3. Full-text screening

Write `RESEARCH/[topic]/screening/full_text.md` after retrieval attempts.

```markdown
| record_id | query_id | source | decision | exclusion_reason | fulltext_status | diagnostic_flags | evidence_anchor | notes |
|---|---|---|---|---|---|---|---|---|
```

Use controlled `fulltext_status` values:

- `retrieved_oa`
- `retrieved_preprint`
- `abstract_only`
- `not_retrieved:paywall`
- `not_retrieved:embargo`
- `not_retrieved:broken_link`
- `not_retrieved:not_found`
- `not_retrieved:access_restricted`
- `not_retrieved:needs_provider`

Borderline records with `abstract_only` or `not_retrieved:*` must remain
visible in PRISMA counts and cannot be silently converted into included studies.

### 4. PRISMA flow data

Write `RESEARCH/[topic]/screening/prisma_flow.md` with counts that reconcile
against `search_results.csv`, `dedup_log.csv`, `title_abstract.md`, and
`full_text.md`.

Minimum fields:

- records identified by source
- records after deduplication
- records screened
- records excluded at title/abstract stage
- reports sought for retrieval
- reports not retrieved
- reports assessed for eligibility
- reports excluded at full-text stage with reasons
- studies included

## Output Contract

- `ScreeningDecisionLog`: write
  `RESEARCH/[topic]/screening/title_abstract.md`.
- `FullTextScreening`: write `RESEARCH/[topic]/screening/full_text.md`.
- `PRISMAFlowData`: write `RESEARCH/[topic]/screening/prisma_flow.md`.
- Consume `RESEARCH/[topic]/search_diagnostics.md`; unresolved diagnostic flags
  must remain visible in screening outputs.
- Separate finding, interpretation, and implication in narrative notes.
- Do not invent citations, data, sample sizes, statistical results, eligibility
  evidence, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose
  or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` only when a
  screening decision supports a central scholarly claim.
- Follow `references/evidence-ledger-contract.md`: supported claims need source
  pointers; unsupported central claims become `gap_note` rows and
  `RESEARCH/[topic]/context/gap_notes.md` entries.
- Preserve `record_id`, `query_id`, `source`, `relevance_reason`, and
  `diagnostic_flags` from search through screening.

## Quality Bar

- [ ] Screening criteria are explicit before decisions start.
- [ ] Every row keeps `record_id`, `query_id`, `source`,
      `relevance_reason`, and `diagnostic_flags`.
- [ ] Every exclusion has a criterion-specific reason.
- [ ] Full-text status uses the controlled vocabulary above.
- [ ] PRISMA counts reconcile with search and dedup artifacts.
- [ ] Review-grade claims are blocked when diagnostics are unresolved.

## Common Pitfalls

| Pitfall | Problem | Fix |
| --- | --- | --- |
| Screening before diagnostics | Coverage gaps get hidden | Inspect `search_diagnostics.md` first |
| Vague exclusion reasons | PRISMA audit fails | Tie each exclusion to a criterion |
| Dropping uncertain records | Borderline cases disappear | Carry `uncertain` to full-text stage |
| Ignoring full-text status | Retrieval limits are invisible | Use controlled `fulltext_status` values |
| Recomputing counts manually | PRISMA numbers drift | Reconcile against source artifacts |

## When to Use

- Use when B1 or targeted reading needs transparent include/exclude decisions.
- Do not use to retrieve PDFs or enrich bibliography metadata; use
  `fulltext-fetcher` and `reference-manager-bridge`.
