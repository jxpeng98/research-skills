# Search Strategy Template

<!--
Usage: Document the canonical search plan before or during provider execution.
Save to: RESEARCH/[topic]/search_strategy.md
-->

# Search Strategy: [Review or Project Title]

## Machine-Readable Search Plan

```yaml
search_mode: systematic_review
concept_blocks:
  - id: c1_population
    label: Population or corpus
    required: true
    terms: []
    phrases: []
    controlled_vocab: []
    exclusions: []
provider_translations:
  - provider: semantic_scholar
    query_id: q1
    translated_query: "replace with provider-specific query"
    filters: {}
    rationale: Initial Semantic Scholar translation; replace with executed query rationale.
filters:
  year_start: ""
  year_end: ""
  language: ""
known_items: []
stopping_rules:
  max_rounds: 2
  stop_when_new_included_below: 3
```

## 1) Research Scope
- Review question / focal RQ:
- Population / context:
- Outcome / construct of interest:
- Time window:
- Language limits:

## 2) Concept Groups and Keyword Expansion

| Concept Group | Core Term | Synonyms / Variants | Exclusions / Notes |
|---|---|---|---|
| Concept A | | | |
| Concept B | | | |
| Concept C | | | |

## 3) Canonical Boolean Logic

```text
(concept_a OR synonym_a1 OR synonym_a2)
AND
(concept_b OR synonym_b1 OR synonym_b2)
AND
(concept_c OR synonym_c1)
```

## 4) Provider-Specific Query Translation

| Provider | Query / Filter Translation | Notes |
|---|---|---|
| Semantic Scholar | | |
| OpenAlex | | |
| Crossref | | |
| arXiv | | |
| Other | | |

## 5) Inclusion / Exclusion Constraints
- Include:
- Exclude:
- Document type limits:
- Venue / field limits:

## 6) Deduplication and Metadata Rules
- Primary key priority: DOI / title / author-year
- Duplicate merge rule:
- Missing metadata fallback:
- Full-text follow-up rule:

## 7) Execution Log Pointers
- Search log file: `search_log.md`
- Results file: `search_results.csv`
- Dedup decisions file: `dedup_log.csv`
- Full-text provenance file: `retrieval_manifest.csv`
- Notes / exceptions:
