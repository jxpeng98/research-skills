---
id: concept-extractor
stage: B_literature
description: "Expand search concepts with controlled vocabulary, synonym mapping, and Boolean query drafting to reduce confirmation bias in literature search."
inputs:
  - type: RQSet
    description: "Research questions defining the conceptual scope"
  - type: PaperNotes
    description: "Notes from seed papers"
    required: false
outputs:
  - type: ConceptMap
    artifact: "literature/concept_extraction.md"
constraints:
  - "Must group concepts into 2–5 concept buckets"
  - "Must include controlled vocabulary candidates (MeSH, ACM CCS, JEL, etc.)"
  - "Must produce a Boolean query draft linked to search_strategy.md"
failure_modes:
  - "Concept buckets overlap significantly (concepts not orthogonal)"
  - "Controlled vocabulary missed relevant descriptor terms"
  - "Boolean query too narrow (high precision, low recall)"
tools: [filesystem, scholarly-search]
tags: [literature, keywords, concepts, boolean-query, controlled-vocabulary]
domain_aware: true
---

# Concept Extractor Skill

Systematically expand your search concepts beyond the initial keywords to ensure comprehensive retrieval and reduce confirmation bias.

## Purpose

Expand search concepts with controlled vocabulary, synonym mapping, and Boolean query drafting to reduce confirmation bias in literature search.

## Related Task IDs

- `B1_5` (concept/keyword extraction)

## Output (contract path)

- `RESEARCH/[topic]/literature/concept_extraction.md`

## When to Use

- After research questions (A1) are drafted but before search execution (B1)
- When initial searches return too few or too many results
- When working across disciplines where terminology differs

## Process

### Step 1: Decompose the RQ into Concept Buckets

Break each research question into 2–5 independent concept groups. Each bucket represents one "facet" of the search — they will be combined with AND in the Boolean query.

**Example**: RQ = "How does remote work affect productivity and well-being among software engineers?"

| Bucket | Core Concept | Role in Query |
|--------|-------------|---------------|
| A | Remote work | Exposure / IV |
| B | Productivity | Outcome 1 |
| C | Well-being | Outcome 2 |
| D | Software engineers | Population |

> **Rule**: If your query has >5 buckets, it is likely too narrow. Try merging related concepts or using a broader population bucket.

### Step 2: Expand Each Bucket with Synonyms and Related Terms

For each bucket, generate:

| Layer | What to Include | Sources |
|-------|----------------|---------|
| **Core terms** | The primary term(s) | Your RQ |
| **Synonyms** | Different words for same concept | Thesaurus, seed papers |
| **Narrower terms** | More specific instances | Controlled vocabularies |
| **Broader terms** | Parent concepts (for recall) | Controlled vocabularies |
| **Related terms** | Adjacent concepts that would catch relevant papers | Seed paper keywords, citation context |
| **Spelling variants** | British/American, hyphenation, abbreviations | Domain knowledge |

**Example for Bucket A (Remote Work)**:

```
Core:      remote work
Synonyms:  telecommuting, telework, work from home, WFH, distributed work
Narrower:  hybrid work, fully remote, home office
Broader:   flexible work arrangements, new ways of working
Related:   virtual teams, asynchronous collaboration
Variants:  tele-work, tele-commuting, work-from-home
```

### Step 3: Map to Controlled Vocabularies

Check whether your domain has a standard classification system:

| Discipline | Vocabulary | Where to Check |
|-----------|------------|----------------|
| Medicine / Health | MeSH (Medical Subject Headings) | PubMed MeSH Browser |
| Psychology | APA Thesaurus | PsycINFO |
| Computer Science | ACM Computing Classification System (CCS) | ACM Digital Library |
| Economics | JEL Classification | Journal of Economic Literature |
| Education | ERIC Thesaurus | eric.ed.gov |
| Sociology | Sociological Abstracts Thesaurus | ProQuest |
| Business / Management | Business Source Subject Headings | EBSCO |
| Multidisciplinary | Library of Congress Subject Headings (LCSH) | Library of Congress |

For each controlled vocabulary term found:
- Record the exact descriptor and code
- Note whether to "explode" (include narrower terms) or use exact match
- Document hierarchical position (broader/narrower terms in the tree)

### Step 4: Draft Boolean Query

Combine buckets using Boolean logic:

```
(Bucket A terms joined with OR)
AND
(Bucket B terms joined with OR)
AND
(Bucket C terms joined with OR)
[AND (Bucket D terms) — optional, may be too restrictive]
```

**Example**:

```
("remote work" OR telecommut* OR telework* OR "work from home" OR WFH
 OR "distributed work" OR "hybrid work" OR "flexible work arrangement*")
AND
(productiv* OR performance OR output OR efficiency)
AND
(well-being OR wellbeing OR "mental health" OR "job satisfaction"
 OR burnout OR "work-life balance")
```

**Query optimization tips**:
- Use truncation (*) for word stems: `productiv*` catches productivity, productive, productiveness
- Use phrase search ("") for multi-word concepts
- Use proximity operators (NEAR/n, W/n) when available
- Test query in one database first, then adapt syntax per database

### Step 5: Validate Coverage

Run a seed-paper test:
1. Collect 5–10 known relevant papers
2. Run the Boolean query
3. Check: how many seed papers are retrieved?
4. If recall < 80%: examine why missed papers were not caught → add missing terms
5. If too many irrelevant results: tighten with additional facets or narrower terms

## Quality Bar

The concept extraction is **ready** when:

- [ ] 2–5 concept buckets identified and labeled
- [ ] Each bucket has ≥5 terms across core/synonym/narrower/broader/related layers
- [ ] Controlled vocabulary checked for at least one discipline-relevant system
- [ ] Boolean query drafted with correct syntax
- [ ] Seed-paper retrieval test shows ≥80% recall
- [ ] Near-misses documented (why certain papers were almost missed)

## Minimal Output Format

```markdown
# Concept & Keyword Extraction

## Research Question
[RQ text]

## Concept Buckets

### Bucket A: [label]
- Core: ...
- Synonyms: ...
- Narrower: ...
- Broader: ...
- Related: ...
- Variants: ...
- Controlled vocabulary: [MeSH/ACM CCS/JEL term + code]

### Bucket B: [label]
...

## Boolean Query (draft)

```
(A1 OR A2 OR A3 ...) AND (B1 OR B2 ...) AND (C1 OR C2 ...)
```

## Seed Paper Retrieval Test
| Paper | Retrieved? | If missed, why |
|-------|-----------|----------------|

## Revised Query (after seed test)
...
```
