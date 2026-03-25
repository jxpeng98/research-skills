# Search Log Template

<!--
Usage: Document all search activities for reproducibility.
Required for PRISMA 2020 compliance and high-impact journal submissions.
Save to: RESEARCH/[topic]/search_log.md
-->

# Search Log

## Review: [Your Review Title]

## Protocol Registration: [PROSPERO ID / OSF DOI / None]

---

## Search Strategy Summary

| Item | Details |
|------|---------|
| **Date Range Searched** | [Start Year] - [End Year] |
| **Languages** | [e.g., English, Chinese] |
| **Last Search Date** | [YYYY-MM-DD] |
| **Search Updated** | [ ] Yes, on [Date] / [ ] No |

---

## Database Search Records

### Database 1: Semantic Scholar

| Field | Value |
|-------|-------|
| **Search Date** | [YYYY-MM-DD HH:MM TZ] |
| **Interface/API Version** | Graph API v1 |
| **Searcher** | [Name/ID] |

**Exact Query:**
```
Endpoint: https://api.semanticscholar.org/graph/v1/paper/search
Parameters:
  query: "[exact search terms]"
  fields: paperId,title,abstract,year,authors,citationCount,venue,openAccessPdf
  year: [start]-[end]
  limit: 100
  offset: [0, 100, 200, ...]
```

**Results:**
| Metric | Count |
|--------|-------|
| Total Results | |
| Retrieved (after pagination) | |
| After Date Filter | |

**Notes:** [Any issues, errors, or observations]

---

### Database 2: arXiv

| Field | Value |
|-------|-------|
| **Search Date** | [YYYY-MM-DD HH:MM TZ] |
| **Interface** | API (export.arxiv.org) |
| **Searcher** | [Name/ID] |

**Exact Query:**
```
URL: http://export.arxiv.org/api/query
Parameters:
  search_query: [all|ti|abs]:[terms] AND cat:[category]
  start: 0
  max_results: 100
  sortBy: relevance
```

**Category Filters Applied:** [e.g., cs.AI, cs.CL, cs.LG]

**Results:**
| Metric | Count |
|--------|-------|
| Total Results | |
| Retrieved | |

---

### Database 3: Google Scholar (Web Search)

| Field | Value |
|-------|-------|
| **Search Date** | [YYYY-MM-DD HH:MM TZ] |
| **Access Method** | Web browser / SerpAPI / Other |
| **Searcher** | [Name/ID] |
| **Location/IP Region** | [Important for reproducibility] |

**Exact Query String:**
```
"[exact search terms]" site:scholar.google.com
```

**Filters Applied:**
- [ ] Date range: [Start] - [End]
- [ ] Sort by: Relevance / Date
- [ ] Include patents: Yes / No
- [ ] Include citations: Yes / No

**Results:**
| Metric | Count |
|--------|-------|
| Approximate Total | |
| Retrieved (first N pages) | |

**Limitations:** [Note: Results may vary by time/location]

---

### Database 4: PubMed (if applicable)

| Field | Value |
|-------|-------|
| **Search Date** | [YYYY-MM-DD HH:MM TZ] |
| **Interface** | PubMed.gov / API |
| **Searcher** | [Name/ID] |

**Exact Query (PubMed Syntax):**
```
([MeSH Term 1] OR [Free text 1])
AND
([MeSH Term 2] OR [Free text 2])
AND
([Year range])

Filters: [Humans, English, etc.]
```

**Results:**
| Metric | Count |
|--------|-------|
| Total Results | |
| Retrieved | |

---

## Additional Sources

### Citation Searching

| Seed Paper | Forward Citations | Backward References |
|------------|-------------------|---------------------|
| Author (Year) | n = | n = |
| Author (Year) | n = | n = |

### Expert Recommendations

| Expert | Affiliation | Papers Suggested | Date |
|--------|-------------|------------------|------|
| | | | |

### Grey Literature

| Source | Search Date | Results |
|--------|-------------|---------|
| ProQuest Dissertations | | |
| Conference proceedings | | |
| Preprint servers | | |

---

## Deduplication Record

| Stage | Count |
|-------|-------|
| Total records from all sources | |
| Duplicates identified | |
| **Unique records for screening** | |

Detailed row-level decisions should be stored in `dedup_log.csv`.

**Deduplication Method:**
- [ ] DOI matching
- [ ] Semantic Scholar paperId matching
- [ ] arXiv ID matching
- [ ] Title + Year + First Author normalization
- [ ] Manual review of potential duplicates

---

## Search Strategy Development

### PRESS Checklist (Peer Review of Electronic Search Strategies)

| Element | Status | Notes |
|---------|--------|-------|
| Translation of RQ to concepts | ✓/✗ | |
| Boolean operators correct | ✓/✗ | |
| Subject headings appropriate | ✓/✗ | |
| Text word searching adequate | ✓/✗ | |
| Spelling/syntax correct | ✓/✗ | |
| Limits appropriate | ✓/✗ | |

### Search Terms Development

| Concept | Primary Terms | Synonyms | MeSH/Controlled Vocab |
|---------|---------------|----------|----------------------|
| Concept 1 | | | |
| Concept 2 | | | |
| Concept 3 | | | |

---

## Amendments Log

| Date | Amendment | Reason | Impact |
|------|-----------|--------|--------|
| [Date] | [Change made] | [Why] | [# additional/removed records] |

---

## Reproducibility Checklist

- [ ] All search dates documented with timestamps
- [ ] Exact queries recorded (copy-paste ready)
- [ ] API parameters fully specified
- [ ] Pagination/offset documented
- [ ] Rate limits/errors noted
- [ ] Deduplication method specified
- [ ] Amendments logged

---

*Log created: [Date]*
*Last updated: [Date]*
*Prepared by: [Name]*
