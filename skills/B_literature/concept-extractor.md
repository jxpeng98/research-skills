---
id: concept-extractor
stage: B_literature
description: "Expand and structure search concepts/keywords to build reproducible search strategy and avoid confirmation bias."
inputs:
  - type: RQSet
    description: "Research questions for concept extraction"
  - type: PaperNotes
    description: "3-10 seed paper titles/abstracts"
    required: false
outputs:
  - type: ConceptMap
    artifact: "literature/concept_extraction.md"
constraints:
  - "Must add controlled vocabulary candidates (MeSH/ACM CCS/JEL) when relevant"
  - "Must group into 2-5 concept buckets"
failure_modes:
  - "Domain-specific terminology not captured by initial seeds"
tools: [filesystem, scholarly-search]
tags: [literature, keywords, concepts, boolean-query, controlled-vocabulary]
domain_aware: true
---

# Concept Extractor Skill

Expand and structure search concepts/keywords to build a reproducible search strategy and avoid confirmation bias.

## Related Task IDs

- `B1_5` (concept/keyword extraction)

## Output (contract path)

- `RESEARCH/[topic]/literature/concept_extraction.md`

## Inputs

- `framing/research_question.md`
- 3–10 seed papers (titles/abstracts) if available

## Procedure

1. Extract key constructs and synonyms from seed abstracts + keywords.
2. Add controlled vocabulary candidates (MeSH / ACM CCS / JEL) if relevant.
3. Group into 2–5 concept buckets.
4. Produce a draft Boolean query that can be copied into `search_strategy.md`.
