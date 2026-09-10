---
id: concept-extractor
stage: B_literature
description: "Extract searchable concept buckets, controlled vocabulary, near misses, and seed recall checks before provider-backed literature search."
inputs:
  - type: RQSet
    description: "Research question, scope, population/context, and outcome or phenomenon terms"
  - type: PaperNotes
    description: "Optional seed papers or notes used to test query recall"
    required: false
outputs:
  - type: ConceptMap
    artifact: "literature/concept_extraction.md"
constraints:
  - "Must produce 2-5 concept buckets before B1 search execution"
  - "Must record excluded ambiguous terms and near misses"
  - "Must run or specify seed recall checks when seed papers exist"
failure_modes:
  - "Concept buckets are too broad to produce useful provider queries"
  - "Seed papers are not recalled by any draft query"
  - "Controlled vocabulary differs across domains"
tools: [filesystem, scholarly-search]
tags: [literature, keywords, concepts, boolean-query, controlled-vocabulary]
domain_aware: true
---

# Concept Extractor Skill

## Purpose

Prepare B1_5 concept extraction for reproducible search. This skill turns a
research question into concept buckets, synonyms, controlled vocabulary
candidates, near misses, excluded ambiguous terms, and seed recall checks that
can be copied into `search_strategy.md`.

## Related Task IDs

- `B1_5` concept and keyword extraction
- Supports `B1` provider-backed search and `B3` snowballing seed rationale.

## Output (contract path)

- `RESEARCH/[topic]/literature/concept_extraction.md`

## Inputs

- `RQSet`: research question, population/context, exposure/intervention or
  phenomenon, comparator if relevant, outcome, time range, and domain.
- Optional seed papers: `RESEARCH/[topic]/notes/*.md`,
  `RESEARCH/[topic]/bibliography.bib`, or user-supplied DOI/title list.
- If inputs are missing or insufficient, write
  `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing scope,
  domain, or seed decision instead of inventing terms.
- Treat seed papers, controlled vocabularies, user-provided terms, and provider
  metadata as evidence sources. Keep unsupported assumptions visibly marked.

## Process

### 1. Decompose the research question

Create 2-5 concept buckets. Each bucket must have one job in the Boolean query.

| Bucket field | Requirement |
| --- | --- |
| label | short mechanism, construct, population, method, or context label |
| required | whether the block must appear in every query |
| core terms | exact terms from the RQ or protocol |
| synonyms | alternate phrases and spelling variants |
| controlled vocabulary | MeSH, JEL, ACM CCS, PsycINFO, or domain vocabulary when relevant |
| near misses | adjacent terms to test but not trust without review |
| excluded ambiguous terms | terms likely to retrieve wrong literatures |

### 2. Draft Boolean blocks

Write provider-neutral blocks first:

```text
(core_term OR synonym OR controlled_vocab_term)
AND
(context_term OR setting_term)
AND
(outcome_or_mechanism_term)
```

Then note provider-specific translation needs without hard-coding provider API
calls. `academic-searcher` owns provider execution.

### 3. Run or specify seed recall test

When seed papers exist, each draft query must state whether it should recall the
seed. Record results in the concept artifact.

| Seed | Expected bucket match | Recalled? | Action |
| --- | --- | --- | --- |
| DOI/title/citekey | bucket names | yes/no/not tested | keep, revise term, or record query gap |

If a known seed is missing, record a `query gap`. Do not broaden the query
silently. Either revise the relevant concept bucket or mark the seed as outside
scope with a reason.

### 4. Produce search-ready handoff

End with a handoff block for `academic-searcher`:

- final concept buckets
- Boolean draft
- required filters
- seed recall status
- unresolved query gaps
- terms excluded on purpose

## Output Contract

- `ConceptMap`: write
  `RESEARCH/[topic]/literature/concept_extraction.md`.
- The final section must be directly reusable in
  `RESEARCH/[topic]/search_strategy.md`.
- Separate finding, interpretation, and implication in any narrative notes.
- Do not invent citations, seed papers, controlled vocabulary membership,
  datasets, sample sizes, statistics, or provider results.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose
  or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` only when a
  concept choice supports a central scholarly claim.
- Follow `references/evidence-ledger-contract.md`: supported claims need source
  pointers; unsupported central claims become `gap_note` rows and
  `RESEARCH/[topic]/context/gap_notes.md` entries.
- Mark vocabulary evidence as `source_anchor` values such as seed citekey,
  controlled vocabulary name, protocol field, or user-provided term.

## Quality Bar

- [ ] 2-5 concept buckets exist and every bucket has a label.
- [ ] Each bucket records core terms, synonyms, controlled vocabulary candidates,
      near misses, and excluded ambiguous terms.
- [ ] Boolean blocks preserve the intended AND/OR logic.
- [ ] Seed recall test is recorded when seed papers exist.
- [ ] Missing seed recall becomes a visible query gap.
- [ ] Final handoff is ready for `academic-searcher`.

## Common Pitfalls

| Pitfall | Problem | Fix |
| --- | --- | --- |
| One giant keyword list | Provider queries become noisy | Split into concept buckets |
| No excluded terms | Ambiguous vocabulary pollutes results | Record excluded ambiguous terms |
| Ignoring seed recall | Known papers disappear from search | Test seeds and record query gaps |
| Silent query broadening | Scope becomes unauditable | Update buckets and explain the change |
| Domain vocabulary skipped | Search misses indexed records | Add controlled vocab candidates |

## When to Use

- Use after Stage A framing and before B1 search when vocabulary, controlled
  terms, or seed recall are uncertain.
- Do not use to execute provider searches; use `academic-searcher`.
