# Search Diagnostics Template

<!--
Usage: Record quality diagnostics after search execution and before screening.
Save to: RESEARCH/[topic]/search_diagnostics.md
Validator: scripts/audit_literature_search_quality.py
-->

# Search Diagnostics: [Review or Project Title]

## Machine-Readable Diagnostics

```yaml
mode: systematic_review
review_grade: true
status: ok
provider_coverage:
  semantic_scholar: 0
  openalex: 0
query_coverage:
  q1: 0
known_item_recall:
  missing: []
flags: []
dedup_ratio: 0.0
```

## Search Scope

- Search mode: `systematic_review` / `targeted_search`
- Review-grade claim: yes/no
- Databases/providers:
- Date searched:
- Time window, language, document-type limits:
- Search strategy artifact: `search_strategy.md`
- Search log artifact: `search_log.md`

## Known-Item Recall

List benchmark papers, seed DOIs, or known seminal items declared in `search_strategy.md`.

| known_item | present_in_results | matched_record_id | action_if_missing |
|---|---|---|---|
|  | yes/no |  | revise query / justify exclusion |

## Provider Coverage

| provider | queries_attempted | raw_hits | normalized_hits | failures | notes |
|---|---:|---:|---:|---:|---|
| Semantic Scholar |  |  |  |  |  |
| OpenAlex |  |  |  |  |  |

## Query Coverage

| query_id | concept_block | required | retrieved_count | zero_hit_reason | next_action |
|---|---|---|---:|---|---|
| q1 |  | yes/no |  |  |  |

## Deduplication Summary

| metric | value | evidence |
|---|---:|---|
| raw records |  | `search_log.md` |
| normalized records |  | `search_results.csv` |
| duplicate decisions |  | `dedup_log.csv` |
| dedup ratio |  | duplicate decisions / raw records |

## Coverage Semantics

- discovery_coverage_basis:
- native_fulltext_candidate_coverage:
- zotero_attachment_coverage:
- known_item_recall:
- duplicate_saturation:
- full_text_access_coverage:
- evidence_limit_distribution:
- cannot_claim_absolute_completeness: true

## Coverage Gaps

Use controlled diagnostic flags where possible:
- `known_item_missing`
- `provider_undercoverage`
- `query_too_narrow`
- `high_duplicate_rate`
- `weak_screening_readiness`

| flag | severity | evidence | resolution |
|---|---|---|---|
|  | warning/error |  |  |

## Next Search Actions

| action | owner | required_before_screening | status |
|---|---|---|---|
|  |  | yes/no | open/complete |
