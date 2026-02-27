# Metadata Enricher Skill

Normalize and complete bibliographic metadata for papers using authoritative sources.

## Purpose

Ensure consistent, complete metadata across all papers in a review by:
- Normalizing DOIs and identifiers
- Completing missing fields from authoritative sources
- Generating consistent citekeys
- Improving deduplication accuracy

## Process

### Step 1: Identifier Normalization

For each paper, normalize identifiers:

#### DOI Normalization
```
Input: "https://doi.org/10.1234/example" OR "doi: 10.1234/example" OR "10.1234/example"
Output: "10.1234/example" (canonical form)
```

#### Other Identifiers
| Identifier | Format | Example |
|------------|--------|---------|
| arXiv ID | `arXiv:YYMM.NNNNN` | arXiv:2303.08774 |
| S2 Paper ID | 40-char hex | abc123... |
| PubMed ID | PMID:NNNNNNNN | PMID:12345678 |
| ISBN | ISBN-10 or ISBN-13 | 978-0-123-45678-9 |

### Step 2: Metadata Completion

Query authoritative sources to fill missing fields:

#### Crossref API (for DOI-based lookup)
```
Endpoint: https://api.crossref.org/works/{doi}
Fields available:
- title, subtitle
- author (with ORCID if available)
- container-title (journal/conference)
- volume, issue, page
- published-print/published-online (date)
- publisher
- ISSN, ISBN
- reference (cited works)
- is-referenced-by-count (citation count)
- license, link (OA info)
```

#### OpenAlex API (for broader metadata + bibliometrics)
```
Endpoint: https://api.openalex.org/works/{doi|openalex_id}
Fields available:
- display_name, title
- authorships (with institutions, countries)
- primary_location (venue, OA status)
- publication_year, publication_date
- cited_by_count
- concepts (topic classification)
- referenced_works, related_works
- open_access (is_oa, oa_url)
```

### Step 3: Field Mapping

Map retrieved metadata to standard fields:

| Standard Field | Crossref | OpenAlex | Semantic Scholar |
|----------------|----------|----------|------------------|
| Title | title[0] | display_name | title |
| Authors | author[].given + family | authorships[].author.display_name | authors[].name |
| Year | published.date-parts[0][0] | publication_year | year |
| Venue | container-title[0] | primary_location.source.display_name | venue |
| Volume | volume | biblio.volume | - |
| Issue | issue | biblio.issue | - |
| Pages | page | biblio.first_page-last_page | - |
| DOI | DOI | doi | externalIds.DOI |
| Citations | is-referenced-by-count | cited_by_count | citationCount |
| OA URL | link[].URL (where content-type=pdf) | open_access.oa_url | openAccessPdf.url |

### Step 4: Citekey Generation

Generate consistent citekeys following the pattern:

**Standard Format:** `lastname[year]keyword`

**Rules:**
1. Take first author's last name (lowercase, ASCII-normalized)
2. Add publication year
3. Add first significant word from title (lowercase, no stopwords)

**Examples:**
| Authors | Year | Title | Citekey |
|---------|------|-------|---------|
| John Smith | 2024 | Machine Learning for Healthcare | smith2024machine |
| María García, Bob Lee | 2023 | Deep Neural Networks | garcia2023deep |
| 李明 (Li Ming) | 2024 | Transformer Architecture | li2024transformer |

**Conflict Resolution:**
- If citekey exists, append `a`, `b`, `c`: `smith2024machine`, `smith2024machineb`

### Step 5: Deduplication Support

Generate dedup keys for matching:

```markdown
## Deduplication Keys (Priority Order)

1. **DOI Match** (exact)
   - Normalize: lowercase, remove leading "https://doi.org/"

2. **arXiv ID Match** (exact)
   - Normalize: extract "YYMM.NNNNN" portion

3. **Title + Year + First Author** (fuzzy)
   - Title: lowercase, remove punctuation, normalize whitespace
   - Year: exact match
   - First Author: last name only, lowercase
   - Similarity threshold: Levenshtein ratio > 0.9
```

## Output Format

```markdown
## Metadata Enrichment Report

### Paper: [Original Title/ID]

**Identifiers:**
| Type | Value | Source |
|------|-------|--------|
| DOI | 10.1234/example | Crossref |
| arXiv | 2303.08774 | arXiv |
| OpenAlex | W1234567890 | OpenAlex |

**Enriched Metadata:**
| Field | Original | Enriched | Source |
|-------|----------|----------|--------|
| Title | [original] | [enriched] | Crossref |
| Authors | [original] | [enriched] | OpenAlex |
| Year | - | 2024 | Crossref |
| Venue | - | Nature | Crossref |
| Volume | - | 615 | Crossref |
| Pages | - | 123-130 | Crossref |
| Citations | - | 150 | OpenAlex |
| OA URL | - | https://... | OpenAlex |

**Generated Citekey:** `smith2024machine`

**Dedup Keys:**
- DOI: `10.1234/example`
- Title+Year+Author: `machine learning healthcare|2024|smith`

**Completeness Score:** 9/10 fields populated
```

## API Reference

### Crossref
- Base URL: `https://api.crossref.org`
- Rate limit: Polite pool (50/sec with mailto header)
- Auth: None required (add `mailto` for polite pool)

### OpenAlex
- Base URL: `https://api.openalex.org`
- Rate limit: 100,000/day (unauthenticated), 10/sec
- Auth: Optional email for higher limits

### Usage Notes
- Prefer Crossref for canonical bibliographic data
- Prefer OpenAlex for OA status and bibliometrics
- Cache responses to avoid rate limits

## Usage

This skill is called by:
- `/lit-review` - Metadata normalization after search
- `/paper-read` - Enrich paper metadata
- Deduplication processes
