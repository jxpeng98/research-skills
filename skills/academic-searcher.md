# Academic Searcher Skill

Conduct systematic searches across academic databases to find relevant scholarly literature.

## Purpose

Execute comprehensive literature searches using:
- Semantic Scholar API (primary)
- arXiv API (CS/AI/Physics/Math)
- Web search for Google Scholar, PubMed, etc.

## Supported Databases

| Database | Coverage | Access Method | Best For |
|----------|----------|---------------|----------|
| Semantic Scholar | 200M+ papers, all domains | API | Broad searches, citations |
| arXiv | Physics, Math, CS, Stats | API | Preprints, CS/AI research |
| Google Scholar | Broad coverage | Web search | Comprehensive coverage |
| PubMed | Biomedical, Life sciences | Web search | Medical research |
| SSRN | Social sciences, Economics | Web search | Business, Law, Economics |

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

Generate search results document:

```markdown
# Search Results: [Topic]

## Search Strategy

### Databases Searched
- [ ] Semantic Scholar
- [ ] arXiv
- [ ] Google Scholar
- [ ] PubMed

### Search Queries

**Semantic Scholar:**
```
[query]
```

**arXiv:**
```
[query]
```

### Date Range
[Start] - [End]

### Filters Applied
- Language: English
- Document type: Journal articles, Conference papers
- ...

## Results Summary

| Database | Results Found | After Dedup |
|----------|---------------|-------------|
| Semantic Scholar | X | |
| arXiv | Y | |
| Google Scholar | Z | |
| **Total** | | **N** |

## Papers Found

### 1. [Title]
- **Authors**: 
- **Year**: 
- **Venue**: 
- **Citations**: 
- **Abstract**: [first 200 chars...]
- **URL**: 
- **Source**: [Database]

### 2. [Title]
...
```

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

## Usage

This skill is called by:
- `/lit-review` - During search phase
- `/find-gap` - To map literature landscape
- `/build-framework` - To find theoretical papers
