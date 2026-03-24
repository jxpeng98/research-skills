---
id: fulltext-fetcher
stage: B_literature
version: "0.2.2"
description: "Retrieve full-text PDFs through open access channels (Unpaywall, S2, CORE, arXiv) with PRISMA-compliant status tracking."
inputs:
  - type: ScreeningDecisionLog
    description: "Papers requiring full-text retrieval"
outputs:
  - type: FullTextStatus
    artifact: "screening/full_text.md"
constraints:
  - "Must attempt retrieval through priority-ordered OA pipeline"
  - "Must document not-retrieved reasons for PRISMA reporting"
failure_modes:
  - "Paywall blocks access to key papers"
  - "Broken OA links"
tools: [filesystem, fulltext-retrieval]
tags: [literature, fulltext, open-access, retrieval, PRISMA]
domain_aware: false
---

# Full-text Fetcher Skill

Retrieve full-text PDFs through open access channels and document retrieval status.

## Purpose

Maximize access to full-text papers while maintaining PRISMA-compliant documentation:
- Attempt OA retrieval through multiple channels
- Document retrieval success/failure with reasons
- Support "reports not retrieved" PRISMA reporting
- Avoid paywalled/illegal access

## Process

### Step 1: Identify Paper

Extract identifiers for lookup:
- DOI (preferred)
- arXiv ID
- PubMed ID
- Semantic Scholar ID
- Title + Authors (fallback)

### Step 2: OA Resolution Pipeline

Attempt retrieval in priority order:

#### Priority 1: Direct OA Sources

**arXiv (for arXiv papers)**
```
Check: Does paper have arXiv ID?
URL: https://arxiv.org/pdf/{arxiv_id}.pdf
Status: Usually immediate access
```

**PubMed Central**
```
Check: Does paper have PMCID?
URL: https://www.ncbi.nlm.nih.gov/pmc/articles/{pmcid}/pdf/
Status: NIH-funded papers often available
```

#### Priority 2: Unpaywall API

```
Endpoint: https://api.unpaywall.org/v2/{doi}?email={your_email}
Response fields:
  - is_oa: boolean
  - best_oa_location:
    - url: direct PDF link
    - url_for_pdf: PDF-specific link
    - version: published/accepted/submitted
    - license: CC-BY, etc.
    - host_type: publisher/repository
```

**Version Preference:**
1. `publishedVersion` - Final publisher version
2. `acceptedVersion` - Author's accepted manuscript (post-review)
3. `submittedVersion` - Preprint (use with caution)

#### Priority 3: Semantic Scholar

```
Endpoint: https://api.semanticscholar.org/graph/v1/paper/{paper_id}
Fields: openAccessPdf
Response:
  openAccessPdf: {
    url: "https://...",
    status: "GREEN" | "BRONZE" | null
  }
```

#### Priority 4: CORE API

```
Endpoint: https://api.core.ac.uk/v3/search/works
Parameters:
  q: doi:"{doi}" OR title:"{title}"
Response:
  downloadUrl: PDF link from repository
```

#### Priority 5: Publisher OA

Check publisher website for:
- Gold OA (fully open)
- Bronze OA (free to read, no license)
- Delayed OA (embargo expired)

### Step 3: Retrieval Attempt

For each source, attempt retrieval and verify:

**Verification Checks:**
- [ ] URL returns 200 status
- [ ] Content-Type is application/pdf
- [ ] File size > 10KB (not error page)
- [ ] PDF is readable (not corrupted)

### Step 4: Document Status

Record retrieval status for PRISMA compliance:

**Status Codes:**
| Code | Meaning | PRISMA Impact |
|------|---------|---------------|
| `RETRIEVED_OA` | Full text via OA | Include in review |
| `RETRIEVED_PREPRINT` | Preprint version available | Note version in extraction |
| `ABSTRACT_ONLY` | Only abstract accessible | May proceed if sufficient |
| `NOT_RETRIEVED` | Full text unavailable | Document reason |
| `PAYWALL` | Behind paywall, no OA | Exclude or note limitation |
| `NOT_FOUND` | Paper not found in any source | Verify bibliographic info |

**"Not Retrieved" Reasons (PRISMA required):**
| Reason | Description |
|--------|-------------|
| `paywall` | Paper behind paywall, no OA version |
| `embargo` | Under embargo, not yet OA |
| `broken_link` | OA link exists but returns error |
| `language` | Full text in non-target language |
| `retracted` | Paper has been retracted |
| `not_found` | Could not locate paper |
| `access_restricted` | Institutional/geographic restriction |

### Step 5: Generate Report

Create retrieval log for PRISMA flow diagram:

## Output Format

```markdown
# Full-text Retrieval Log

## Review: [Topic]
## Date: [Date]

---

## Retrieval Summary

| Status | Count | % |
|--------|-------|---|
| Retrieved (OA) | | |
| Retrieved (Preprint) | | |
| Abstract Only | | |
| Not Retrieved | | |
| **Total** | | 100% |

---

## Successful Retrievals

| # | Citation | DOI | Source | Version | URL |
|---|----------|-----|--------|---------|-----|
| 1 | Smith (2024) | 10.1234/... | Unpaywall | Published | https://... |
| 2 | Jones (2023) | - | arXiv | Preprint | https://arxiv.org/... |

---

## Not Retrieved (PRISMA Reporting)

| # | Citation | DOI | Reason | Notes |
|---|----------|-----|--------|-------|
| 1 | Lee (2022) | 10.5678/... | paywall | Springer, no OA |
| 2 | Wang (2021) | - | not_found | Title search failed |

### Reasons Summary

| Reason | Count |
|--------|-------|
| paywall | |
| embargo | |
| not_found | |
| broken_link | |
| **Total Not Retrieved** | |

---

## Retrieval Attempts Log

### Paper: Smith (2024)
| Attempt | Source | Result | Notes |
|---------|--------|--------|-------|
| 1 | Unpaywall | SUCCESS | Published version |
| 2 | - | - | Stopped on success |

### Paper: Lee (2022)
| Attempt | Source | Result | Notes |
|---------|--------|--------|-------|
| 1 | Unpaywall | FAILED | No OA location |
| 2 | S2 | FAILED | openAccessPdf null |
| 3 | CORE | FAILED | Not indexed |
| 4 | Publisher | FAILED | Paywall |

---

*Retrieval completed: [Date]*
*Sources attempted: Unpaywall, Semantic Scholar, CORE, arXiv, PMC*
```

## API Reference

### Unpaywall
- Endpoint: `https://api.unpaywall.org/v2/{doi}`
- Auth: Email required in query string
- Rate limit: 100,000/day
- Best for: DOI-based OA lookup

### CORE
- Endpoint: `https://api.core.ac.uk/v3/`
- Auth: API key required
- Rate limit: Varies by plan
- Best for: Repository content

### Semantic Scholar
- Endpoint: `https://api.semanticscholar.org/graph/v1/paper/{id}`
- Fields: `openAccessPdf`
- Rate limit: 100 requests/5 min
- Best for: Quick OA check

## Usage

This skill is called by:
- `/lit-review` Phase 3.5 - Full-text retrieval
- `/paper-read` Step 1 - Paper retrieval
- Screening processes requiring full text
