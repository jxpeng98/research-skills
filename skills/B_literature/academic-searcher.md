---
id: academic-searcher
stage: B_literature
description: "Conduct systematic multi-database searches across Semantic Scholar, arXiv, OpenAlex, and other sources to build a reproducible corpus."
inputs:
  - type: RQSet
    description: "Research questions with key terms"
  - type: SearchQueryPlan
    description: "Optional pre-built search strategy"
    required: false
outputs:
  - type: SearchQueryPlan
    artifact: "search_strategy.md"
  - type: SearchResults
    artifact: "search_results.csv"
  - type: SearchLog
    artifact: "search_log.md"
  - type: DedupLog
    artifact: "dedup_log.csv"
constraints:
  - "Must log exact query strings and timestamps for reproducibility"
  - "Must deduplicate across databases"
failure_modes:
  - "API rate limits exhausted"
  - "Zero results for narrow queries"
tools: [filesystem, scholarly-search, fulltext-retrieval]
tags: [literature, search, databases, API, reproducibility]
domain_aware: false
---

# Academic Searcher Skill

Conduct systematic searches across academic databases to find relevant scholarly literature.

## Purpose

Execute comprehensive literature searches using:
- Semantic Scholar API (primary)
- arXiv API (CS/AI/Physics/Math)
- Web search for Google Scholar, PubMed, etc.

## Granularity Boundary

Treat the following as embedded subflows or provider calls inside `academic-searcher`, not separate top-level skills:

- Query drafting / Boolean assembly
- Keyword expansion / synonym expansion
- Provider-specific search translation
- Crossref / OpenAlex metadata verification
- Deduplication and result normalization

If a change only affects one of those concerns, update this skill, its templates, or the MCP/provider layer instead of creating a new top-level skill.

## Related Task IDs

- `B1` (systematic review pipeline)
- Support tasks: `A4` (gap scan), `A5` (venue scan), `B3` (snowballing expansion)

## Standard Outputs (contract paths)

- `RESEARCH/[topic]/search_strategy.md`
- `RESEARCH/[topic]/search_log.md`
- `RESEARCH/[topic]/search_results.csv`
- `RESEARCH/[topic]/dedup_log.csv`

Template references:
- `templates/search-strategy.md`
- `templates/search-log.md`
- `templates/dedup-log.csv`

## Provider Ownership Boundary

- `scholarly-search` owns query planning, provider execution, and raw hit capture
- `scholarly-search` appends dedup candidate decisions to `dedup_log.csv`
- `metadata-registry` owns final normalized bibliography state
- `fulltext-retrieval` owns full-text provenance and retrieval manifests

Do not collapse all literature responsibilities into this skill just because one agent is executing the step.

## Supported Databases

| Database | Coverage | Access Method | Best For |
|----------|----------|---------------|----------|
| Semantic Scholar | 200M+ papers, all domains | API | Broad searches, citations |
| arXiv | Physics, Math, CS, Stats | API | Preprints, CS/AI research |
| OpenAlex | 250M+ works, all domains | API | Bibliometrics, author data |
| Crossref | 140M+ DOIs | API | Metadata verification |
| Google Scholar | Broad coverage | Web search | Comprehensive coverage |
| PubMed | Biomedical, Life sciences | Web search | Medical research |
| SSRN | Social sciences, Economics | Web search | Business, Law, Economics |
| Unpaywall | OA status for DOIs | API | Full-text availability |
| CORE | 200M+ OA articles | API | Repository content |

## Process

### Step 1: Query Construction

From the research question and key terms, construct database-specific queries:

#### Boolean Query Building

```
(concept1 OR synonym1a OR synonym1b)
AND
(concept2 OR synonym2a OR synonym2b)
AND
(concept3 OR synonym3a)
```

#### Semantic Scholar Query

```
Search URL: https://api.semanticscholar.org/graph/v1/paper/search
Parameters:
- query: [search terms]
- fields: paperId,title,abstract,year,authors,citationCount,venue,openAccessPdf
- limit: 100
- year: [start]-[end]
```

#### arXiv Query

```
Search URL: http://export.arxiv.org/api/query
Parameters:
- search_query: all:[terms] OR ti:[title terms] OR abs:[abstract terms]
- start: 0
- max_results: 100
- sortBy: relevance
```

Use `templates/search-strategy.md` to keep concept groups, synonyms, limits, and provider translations in one canonical structure.

### Step 2: Execute Searches

For each database:

1. **Semantic Scholar**
   - Use search_web tool with site:semanticscholar.org
   - Or construct API call via read_url_content
   - Extract: title, authors, year, abstract, citation count, PDF link

2. **arXiv**
   - Use read_url_content with arXiv API endpoint
   - Parse Atom feed response
   - Extract: title, authors, abstract, categories, PDF link

3. **Google Scholar**
   - Use search_web tool
   - Filter by date range
   - Note: Limited metadata

### Step 3: Result Processing

For each result, extract:

```markdown
| Field | Value |
|-------|-------|
| Title | |
| Authors | |
| Year | |
| Venue | |
| Abstract | |
| Citations | |
| DOI/URL | |
| PDF | |
| Database | |
```

### Step 4: Deduplication

Identify and merge duplicates based on:
- Title similarity (>90%)
- DOI match
- Author + Year match

### Step 5: Output

Write the canonical artifacts (CSV + logs) first; optional narrative summaries can be added after.

#### `search_strategy.md` (canonical planning artifact)

Minimum sections:
- Research question + scope
- Concept groups and keyword expansion
- Provider-specific query translations
- Date/language/type filters
- Deduplication and exclusion rules

#### `search_results.csv` (one row per record)

Recommended minimal schema:

```csv
record_id,source,query_id,retrieved_at,title,authors,year,venue,doi,url,abstract
```

#### `dedup_log.csv` (one row per dedup decision)

```csv
candidate_record_id,canonical_record_id,decision,match_basis,resolver,notes
```

#### `search_log.md` (reproducibility log)

Append one entry per database execution with:
- timestamp + interface/API version
- exact query string + filters
- counts (retrieved, after dedup, etc.)

#### Optional: human-readable results summary

If needed, generate a short narrative summary (1–2 pages) *in addition* to the canonical CSV/log outputs.

Suggested sections:
- databases searched + date range
- query strings
- results counts per database (pre/post dedup)
- top 10 most relevant papers (titles + citekeys)

Write summaries into the project `notes/` folder when appropriate.

## API Reference

### Semantic Scholar API

Base URL: `https://api.semanticscholar.org/graph/v1`

Endpoints:
- `/paper/search` - Search papers
- `/paper/{paper_id}` - Get paper details
- `/paper/{paper_id}/citations` - Get citing papers
- `/paper/{paper_id}/references` - Get references

Fields:
- `paperId`, `title`, `abstract`, `venue`, `year`
- `authors.name`, `authors.authorId`
- `citationCount`, `referenceCount`
- `openAccessPdf.url`

### arXiv API

Base URL: `http://export.arxiv.org/api/query`

Query parameters:
- `search_query` - Search terms
- `id_list` - Specific arXiv IDs
- `start` - Starting index
- `max_results` - Max results (default 10)
- `sortBy` - relevance, lastUpdatedDate, submittedDate
- `sortOrder` - ascending, descending

Categories (cs.*):
- cs.AI, cs.CL, cs.CV, cs.LG, cs.NE, cs.RO, cs.SE

### OpenAlex API

Base URL: `https://api.openalex.org`

Endpoints:
- `/works` - Search works
- `/works/{openalex_id}` - Get work details
- `/authors/{id}` - Get author info
- `/sources/{id}` - Get venue info

Search parameters:
- `search` - Full-text search
- `filter` - Structured filters (e.g., `publication_year:2020-2024`)
- `sort` - Sort order (e.g., `cited_by_count:desc`)
- `per-page` - Results per page (max 200)

Example:
```
https://api.openalex.org/works?search=machine+learning&filter=publication_year:2020-2024&sort=cited_by_count:desc
```

### Crossref API

Base URL: `https://api.crossref.org`

Endpoints:
- `/works` - Search works
- `/works/{doi}` - Get work by DOI

Parameters:
- `query` - Free text search
- `query.title` - Title search
- `query.author` - Author search
- `filter` - Filters (e.g., `from-pub-date:2020`)
- `rows` - Results per page (max 1000)

Headers:
- Add `mailto` parameter for polite pool access

### Unpaywall API

Base URL: `https://api.unpaywall.org/v2`

Endpoint: `/{doi}?email={your_email}`

Response includes:
- `is_oa` - Boolean
- `best_oa_location` - Best available OA version
- `oa_locations[]` - All OA versions

### CORE API

Base URL: `https://api.core.ac.uk/v3`

Endpoints:
- `/search/works` - Search works
- `/works/{id}` - Get work details

Parameters:
- `q` - Query string
- `limit` - Max results

## Error Handling & Fallback Strategy

When API calls fail, apply this fallback chain:

### Fallback Priority

```
Discovery Layer:
1. Semantic Scholar API (fast relevance scan, concept discovery)
2. OpenAlex API (work metadata, broader scholarly graph)
3. Crossref API (DOI-focused normalization and reproducible record lookup)

Review-Grade Extension:
4. Citation graph expansion from seed papers
5. Full-text retrieval from Zotero / OA resolver
```

### Error Types & Recovery

| Error | Cause | Recovery Action |
|-------|-------|-----------------|
| 429 Too Many Requests | Rate limit | Wait 60s, then retry (max 3 attempts) |
| 503 Service Unavailable | Server overload | Wait 30s, try next source |
| 404 Not Found | Invalid query | Log error, continue with other sources |
| Timeout | Network issue | Retry once, then skip source |
| Empty Results | No matches | Broaden query, try synonyms |

### Fallback Logging

Document all fallback events:
```markdown
## Fallback Events Log

| Timestamp | Primary Source | Error | Fallback Source | Result |
|-----------|----------------|-------|-----------------|--------|
| [time] | Semantic Scholar | 429 | OpenAlex | Success |
| [time] | OpenAlex | Timeout | Crossref | Success |
```

### Rate Limit Management

| API | Rate Limit | Strategy |
|-----|------------|----------|
| Semantic Scholar | 100 req/5min | Batch requests, 1 req/3sec |
| OpenAlex | 10 req/sec | Reasonable pacing |
| Crossref | Polite pool: 50/sec | Add mailto header |
| CORE | Varies by plan | Check quota before batch |
| Unpaywall | 100k/day | Cache responses |

### Review-Grade Search Policy

For rigorous academic literature search, do not rely on a single search engine.

Minimum review-grade stack:
1. Run discovery queries across Semantic Scholar plus one metadata-focused source (OpenAlex or Crossref)
2. Normalize DOI, author, venue, and year via `metadata-registry`
3. Record exact search strings, dates, provider names, and result counts in `search_log.md`
4. Deduplicate before screening
5. Snowball backward and forward citations from high-value seed papers
6. Resolve full text with provenance before synthesis

Avoid using Google Scholar as the primary reproducible pipeline source. It can still be useful for manual spot checks, but it is not the default audit-friendly fallback for this system.

## Usage

This skill is called by:
- `/lit-review` - During search phase
- `/find-gap` - To map literature landscape
- `/build-framework` - To find theoretical papers

## Quality Bar

- [ ] 检索式使用 Boolean 逻辑且每个 concept block 至少包含 3 个同义词
- [ ] 至少搜索 2 个数据库
- [ ] search_log.md 完整记录了每次检索的日期、数据库、命中数
- [ ] 去重后记录了 dedup_log.csv 中的合并决策
- [ ] 检索式经过 domain expert 或 librarian 审议（或标注待审）

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 检索式过窄 | 漏掉关键文献 | 加 MeSH/Emtree + 自由词 |
| 未覆盖灰色文献 | 系统偏差 | 明确说明灰色文献策略 |
| 去重不一致 | 同一篇文献保留多次 | 用 DOI + 标题 fuzzy match |
| 检索式不可复现 | Reviewer 无法验证 | 完整记录 search string + 日期 |
| 只搜 PubMed | 学科覆盖偏差 | 加 Scopus/WoS/domain-specific DB |

## When to Use

- 需要可复现的检索式设计和数据库搜索时
- 系统综述或 scoping review 的初始检索阶段
- 需要生成 search log 和 dedup log 时
- 需要覆盖多个数据库（PubMed, Scopus, WoS 等）时
