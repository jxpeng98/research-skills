---
id: citation-snowballer
stage: B_literature
version: "0.2.0"
description: "Trace citations forward and backward from seed papers to expand corpus coverage and identify seminal works."
inputs:
  - type: SearchResults
    description: "Initial search results for seed selection"
outputs:
  - type: SnowballLog
    artifact: "snowball_log.md"
constraints:
  - "Must select 5-15 seed papers"
  - "Must apply relevance scoring with weighted factors"
  - "Must deduplicate against existing corpus"
failure_modes:
  - "API rate limits during large-scale snowballing"
  - "Diminishing returns after level 1"
tools: [filesystem, scholarly-search, citation-graph]
tags: [literature, citations, snowballing, forward-backward]
domain_aware: false
---

# Citation Snowballer Skill

Systematically trace citations forward and backward to identify additional relevant papers.

## Purpose

Extend literature search beyond database queries by:
- Forward citation searching (who cites this paper?)
- Backward reference searching (what does this paper cite?)
- Identifying seminal and highly-cited works
- Capturing papers missed by keyword searches

## Process

### Step 1: Identify Seed Papers

Select seed papers from initial search results based on:

**Selection Criteria:**
| Criterion | Description |
|-----------|-------------|
| High citations | Top 10% by citation count in search results |
| Seminal works | Foundational papers identified in abstracts |
| Recent reviews | Systematic reviews or meta-analyses (last 5 years) |
| Key authors | Papers by recognized domain experts |

**Recommended seed set size:** 5-15 papers

### Step 2: Forward Citation Search

Find papers that cite the seed papers.

#### Using Semantic Scholar API
```
Endpoint: https://api.semanticscholar.org/graph/v1/paper/{paper_id}/citations
Parameters:
  fields: paperId,title,abstract,year,authors,citationCount,venue
  limit: 100
  offset: [paginate as needed]
```

#### Using OpenAlex API
```
Endpoint: https://api.openalex.org/works
Parameters:
  filter: cites:{openalex_id}
  per-page: 100
  cursor: [paginate as needed]
```

**Process:**
1. For each seed paper, retrieve all citing papers
2. Apply date filters (match original search criteria)
3. Remove papers already in search results (by DOI/title)
4. Score by relevance (citation count, venue quality)

### Step 3: Backward Reference Search

Find papers cited by the seed papers.

#### Using Semantic Scholar API
```
Endpoint: https://api.semanticscholar.org/graph/v1/paper/{paper_id}/references
Parameters:
  fields: paperId,title,abstract,year,authors,citationCount,venue
  limit: 100
```

#### Using Crossref API (for DOI-based lookup)
```
Endpoint: https://api.crossref.org/works/{doi}
Field: reference[] (list of cited works)
```

**Process:**
1. For each seed paper, retrieve reference list
2. Apply date filters
3. Remove duplicates
4. Prioritize frequently cited references (appear in multiple seed papers)

### Step 4: Iterative Snowballing

For thorough coverage, consider second-level snowballing:

```
Level 0: Original search results
    ↓
Level 1: Citations of seed papers (forward) + References of seed papers (backward)
    ↓
Level 2 (optional): Citations/references of high-value Level 1 papers
```

**Stopping criteria:**
- Saturation: Few new unique papers found
- Diminishing returns: New papers increasingly off-topic
- Resource limits: Time/API rate constraints

### Step 5: Deduplication

Apply strict deduplication against existing corpus:

**Priority Order:**
1. DOI match (exact)
2. Semantic Scholar ID match
3. arXiv ID match
4. Title + Year + First Author (fuzzy, threshold > 0.9)

### Step 6: Relevance Scoring

Score new candidates for inclusion priority:

| Factor | Weight | Scoring |
|--------|--------|---------|
| Citation frequency | 30% | How many seed papers cite/are cited by this |
| Citation count | 20% | Total citations (log-scaled) |
| Recency | 20% | Newer = higher score |
| Venue quality | 15% | Top venue = higher score |
| Title relevance | 15% | Keyword match with search terms |

## Output Format

```markdown
# Citation Snowballing Log

## Review: [Topic]
## Date: [Date]

---

## Seed Papers

| # | Citation | Citations | Selected Because |
|---|----------|-----------|------------------|
| 1 | Smith (2023) | 150 | High citations |
| 2 | Jones (2020) | 500 | Seminal work |
| 3 | Lee (2022) | 80 | Recent review |

---

## Forward Citation Results

### From: Smith (2023)

| # | Title | Year | Citations | Relevance | Status |
|---|-------|------|-----------|-----------|--------|
| 1 | [Title] | 2024 | 25 | High | NEW |
| 2 | [Title] | 2023 | 10 | Medium | DUPLICATE |

**Total forward citations found:** X
**New unique papers:** Y
**Duplicates removed:** Z

### From: Jones (2020)
[Similar table...]

---

## Backward Reference Results

### From: Smith (2023)

| # | Title | Year | Cited by N seeds | Status |
|---|-------|------|------------------|--------|
| 1 | [Title] | 2018 | 3 | NEW - HIGH PRIORITY |
| 2 | [Title] | 2015 | 1 | NEW |

**Total references found:** X
**New unique papers:** Y

---

## Summary

| Metric | Count |
|--------|-------|
| Seed papers | |
| Total forward citations examined | |
| Total backward references examined | |
| New unique papers identified | |
| Already in corpus (duplicates) | |
| **Papers added to screening** | |

## High-Priority Additions

Papers cited by multiple seeds or with high relevance scores:

| # | Title | Year | Score | Reason |
|---|-------|------|-------|--------|
| 1 | | | | Cited by 3 seeds |
| 2 | | | | 500+ citations, seminal |

---

## API Calls Log

| Timestamp | Endpoint | Parameters | Results |
|-----------|----------|------------|---------|
| [Time] | S2 citations | paper_id=X | 45 |
| [Time] | S2 references | paper_id=X | 30 |

---

*Snowballing completed: [Date]*
```

## API Reference

### Semantic Scholar
- Citations: `GET /paper/{id}/citations`
- References: `GET /paper/{id}/references`
- Rate limit: 100 requests/5 min (public)

### OpenAlex
- Citing works: `GET /works?filter=cites:{id}`
- Referenced works: Access via work's `referenced_works` field
- Rate limit: 10 requests/sec

### Crossref
- References: `GET /works/{doi}` → `reference` field
- Note: Reference quality varies by publisher

## Usage

This skill is called by:
- `/lit-review` Phase 3.5 - Citation snowballing
- `/find-gap` - Identifying seminal works and research clusters
