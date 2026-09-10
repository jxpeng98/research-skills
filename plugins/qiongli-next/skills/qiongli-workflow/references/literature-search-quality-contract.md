# Literature Search Quality Contract

This contract defines the minimum evidence needed before Stage B can claim that a literature search is reproducible, review-grade, or ready for screening.

Runtime artifacts:
- templates/search-diagnostics.md is the source template for `RESEARCH/[topic]/search_diagnostics.md`.
- `scripts/audit_literature_search_quality.py` enforces mode-aware gates on a project root or a diagnostics file.
- `scripts/materialize_literature_search_bundle.py` can materialize provider JSON output into `search_strategy.md`, `search_results.csv`, `search_log.md`, `dedup_log.csv`, and `search_diagnostics.md`.
- `tests/test_literature_search_quality_audit.py` covers the blocking and nonblocking validator behavior.

## Modes

Use `mode: systematic_review` when the output will support a systematic review, PRISMA-style evidence base, or any claim that the search is comprehensive. This mode is review-grade by default.

Use `mode: targeted_search` when the goal is a focused scan, seed-paper discovery, venue positioning, or a bounded background search. This mode may warn on limited coverage without blocking the workflow, but it cannot be used to claim exhaustive coverage.

## Required Sections

`search_diagnostics.md` must include:
- `Search Scope`
- `Known-Item Recall`
- `Provider Coverage`
- `Query Coverage`
- `Deduplication Summary`
- `Coverage Gaps`
- `Next Search Actions`

The machine-readable YAML block should record `mode`, `review_grade`, `status`, `provider_coverage`, `query_coverage`, `known_item_recall`, `flags`, and `dedup_ratio`.

## Blocking Gates

For `systematic_review` or `review_grade` diagnostics:
- provider coverage must include at least two productive providers
- all providers failing is always blocking
- zero-hit required concept queries are blocking until revised or justified
- missing known items are blocking until recovered, justified as out of scope, or logged as a search gap
- `weak_screening_readiness` blocks screening from being treated as review-grade

For `targeted_search` diagnostics:
- single-provider coverage is a warning, not a blocker
- zero-hit query blocks are warnings unless all providers fail
- missing known items require a gap note before any later review-grade claim

## Stage Consumption

`academic-searcher` produces `search_diagnostics.md` alongside the search bundle. `citation-snowballer` uses the diagnostics to select seeds, expand undercovered concept streams, and report saturation. `paper-screener` must carry diagnostic flags into screening records so weak search coverage is visible before PRISMA counts are finalized.

Project validation:
- `validate_project_artifacts.py` calls the audit for `B1`.
- `B3` requires seed evidence, forward and backward citation evidence, dedup reconciliation, and saturation status in snowball artifacts.
- `B6` requires `literature/literature_map.md` to include included studies, concept streams, and evidence gaps.
