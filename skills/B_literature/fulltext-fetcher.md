---
id: fulltext-fetcher
stage: B_literature
description: "Retrieve full-text PDFs through open access channels (Unpaywall, S2, CORE, arXiv) with PRISMA-compliant status tracking."
inputs:
  - type: ScreeningDecisionLog
    description: "Papers requiring full-text retrieval"
outputs:
  - type: FullTextStatus
    artifact: "screening/full_text.md"
  - type: RetrievalManifest
    artifact: "retrieval_manifest.csv"
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

## Provider Ownership Boundary

- `fulltext-retrieval` owns `screening/full_text.md` and `retrieval_manifest.csv`
- it should not overwrite `search_strategy.md` or `bibliography.bib`
- if retrieval changes study eligibility, that decision still flows back through screening artifacts
- the built-in provider is a retrieval-planning stub: it can draft manifests and flag OA/manual follow-up candidates, but actual downloads still usually come from an external resolver such as Zotero

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

When the built-in stub is used without an external resolver, it typically emits planning statuses such as:
- `not_retrieved:oa_candidate`
- `not_retrieved:needs_provider`
- `not_retrieved:missing_locator`

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

```csv
record_id,citekey,doi,retrieval_status,version_label,source_provider,retrieved_at,fulltext_path,access_url,license,notes
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

## Quality Bar

- [ ] 每篇入选论文的全文获取状态已记录
- [ ] Retrieval manifest 包含 DOI、来源、获取日期、版本信息
- [ ] 无法获取的论文标注了原因和替代方案
- [ ] PDF 文件命名统一且可追踪
- [ ] 获取状态可直接用于更新 PRISMA flow

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 版本不一致 | Preprint 与 published 版本结果不同 | 标注版本并优先 final published |
| 付费墙阻断 | 无法获取闭源文献 | 尝试 Unpaywall、作者邮件、ILL |
| 文件未命名编号 | PDF 无法追踪到文献 | 用 citekey 命名 |
| 未记录失败 | PRISMA flow 数据缺失 | 每次失败都写入 manifest |
| 忽略 supplementary | 关键方法/数据在 supplement | 全文 + supplement 一同获取 |

## When to Use

- 已完成筛选但缺少入选论文的全文时
- 需要 PRISMA-compliant 的全文获取状态追踪时
- 需要通过 OA 渠道或数据库下载全文 PDF 时
- 需要生成 retrieval manifest 记录获取结果时
