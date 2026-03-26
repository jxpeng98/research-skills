---
id: submission-packager
stage: H_submission
description: "Assemble submission-ready package: cover letter, compliance files, title page, disclosures, and supplementary materials."
inputs:
  - type: Manuscript
    description: "Finalized manuscript"
  - type: VenueAnalysis
    description: "Target venue requirements"
    required: false
outputs:
  - type: SubmissionPackage
    artifact: "submission/cover_letter.md"
constraints:
  - "Must include all venue-required items"
  - "Must generate CRediT author contributions when required"
  - "Must verify anonymization for double-blind venues"
failure_modes:
  - "Missing co-author information for disclosures"
  - "Venue requirements changed since analysis"
  - "Supplementary materials reference missing files"
tools: [filesystem, submission-kit]
tags: [submission, cover-letter, checklist, disclosures, supplementary]
domain_aware: true
---

# Submission Packager Skill

Assemble a complete, venue-compliant submission package that minimizes administrative desk-rejection risk.

## Related Task IDs

- `H1` (submission packaging)

## Output (contract paths)

- `RESEARCH/[topic]/submission/cover_letter.md`
- `RESEARCH/[topic]/submission/submission_checklist.md`
- `RESEARCH/[topic]/submission/title_page.md`
- `RESEARCH/[topic]/submission/` — additional files as needed

## When to Use

- You have a near-final manuscript draft ready for submission
- You want to prepare a complete package before uploading to the submission system
- After a revision, to ensure nothing was dropped from the package

## Process

### Step 1: Extract Venue-Specific Requirements

Build a requirements checklist from the venue's author guidelines:

| Requirement Category | Common Items | Source |
|---------------------|-------------|--------|
| **Manuscript format** | Word/LaTeX/PDF, template, line numbering, double spacing | Author guidelines |
| **Page/word limit** | Including or excluding references, abstract | Author guidelines |
| **Anonymization** | Remove author info, self-citations, tracked changes metadata | "Preparing for submission" |
| **Title page** | Separate from manuscript; includes authors, affiliations, ORCID, correspondence | Author guidelines |
| **Abstract** | Structured or narrative; word limit | Author guidelines |
| **Keywords** | Number required; controlled vocabulary? | Author guidelines |
| **Figures/tables** | Embedded or separate files; resolution; file formats | Author guidelines |
| **Supplementary** | Separate file(s); formatting; word limit | Author guidelines |
| **Cover letter** | Required? Suggested content? | Author guidelines |
| **Declarations** | COI, ethics, funding, data availability, AI disclosure | Author guidelines |
| **Reporting checklist** | CONSORT, STROBE, PRISMA, etc. | Author guidelines |
| **Suggested reviewers** | Number required; can you exclude? | Submission system |

### Step 2: Prepare the Cover Letter

A strong cover letter should accomplish three things: fit the venue scope, convey the contribution, and demonstrate professionalism.

#### Cover Letter Template

```markdown
Dear [Editor name or "Editors of [Journal]"],

We submit our manuscript entitled "[Title]" for consideration as a
[full paper / research letter / review article] in [Journal Name].

**Why this venue**: [1–2 sentences connecting your topic to the journal's
scope and recent published work]

**What we contribute**: [2–3 sentences on the core contribution — gap
filled, method, key finding]

**Key findings**: [2–3 bullet points with specific results]

**Significance**: [1 sentence on why this matters for the field / practice]

[The manuscript is [word count] words, includes [n] tables and [n] figures.]

[This manuscript has not been published elsewhere and is not under
consideration at another journal. All authors have read and approved
the final manuscript.]

[We have no conflicts of interest to disclose. / COI disclosure: ...]

[Optional: We suggest the following potential reviewers: ...]

[Optional: We request that [name] not serve as reviewer due to [reason].]

Thank you for your consideration.

Sincerely,
[Corresponding author name]
[Affiliation, email, ORCID]

on behalf of all authors
```

> **Anti-pattern**: A cover letter that is just "please consider our paper." It should sell the paper in 30 seconds of reading.

### Step 3: Prepare the Title Page

```markdown
# Title Page

## Title
[Full title]

## Running title
[≤50 characters]

## Authors
1. [First Last]¹*, ORCID: 0000-0000-0000-0000
2. [First Last]², ORCID: 0000-0000-0000-0000

## Affiliations
¹ [Department, University, City, Country]
² [Department, University, City, Country]

## Corresponding author
[Name], [Email], [Phone], [Address]

## Author contributions (CRediT)
[See credit-taxonomy-helper if needed]
- [Author 1]: Conceptualization (lead), Methodology (lead), Writing – original draft
- [Author 2]: Formal analysis (equal), Writing – review & editing

## Acknowledgments
[People who helped but don't meet authorship criteria]

## Funding
[Grant number, funder, role of funder]

## Conflict of interest
[Disclosure or "The authors declare no conflicts of interest"]

## Data availability
[Statement matching your DMP and data sharing plan]

## AI disclosure
[Tools used, purpose, verification statement]

## Word count
[Main text: n words; Abstract: n words]
```

### Step 4: Prepare Required Statements

For each venue-required statement, generate using templates:

| Statement | Key Content | Template Path |
|-----------|-------------|---------------|
| **Ethics statement** | IRB protocol #, approval date, consent procedure | `templates/ethics-statement.md` |
| **Data availability** | Repository/DOI, or "available upon request," or restricted access rationale | `templates/data-availability.md` |
| **Code availability** | Repository link, language, dependencies | Where applicable |
| **COI declaration** | Each author's potential conflicts, or "none declared" | `templates/coi-statement.md` |
| **Funding** | Grant numbers, funder names, role of funder in study | `templates/funding-statement.md` |
| **AI disclosure** | Tools used (ChatGPT, Copilot, etc.), purpose, verification | `templates/ai-disclosure.md` |
| **CRediT** | Each author's roles from the 14 categories | `templates/author-contributions-credit.md` |

### Step 5: Anonymization Check (for double-blind)

| What to Check | Where to Look | Fix |
|---------------|--------------|-----|
| Author names | Title page, headers, footers | Remove from manuscript; keep only on separate title page |
| Self-citations | Reference list, in-text citations | Change to "Author (year)" → "[blinded for review]" |
| Acknowledgments | Acknowledgment section | Remove or anonymize |
| Tracked changes metadata | Document properties (Word) | Accept all changes; strip metadata |
| Institution names | Methods (data source, lab name) | Anonymize if identifying |
| Repository links | GitHub, OSF | Use anonymous link (OSF anonymous view) or note "available upon acceptance" |
| File properties | PDF/Word file metadata | Strip author metadata from file |

### Step 6: Submission Checklist

Generate a final pre-submission checklist:

```markdown
# Submission Checklist

## Files
- [ ] Manuscript file ([format]) — properly formatted, within word limit
- [ ] Title page (separate file for double-blind)
- [ ] Cover letter
- [ ] Figures (embedded or separate per venue policy, [resolution] DPI)
- [ ] Tables (embedded or separate)
- [ ] Supplementary materials
- [ ] Reporting checklist (CONSORT/STROBE/PRISMA/COREQ/SRQR)

## Content
- [ ] Abstract within word limit ([n] words)
- [ ] Keywords provided ([n] keywords)
- [ ] All author information complete (ORCID, affiliations)
- [ ] CRediT author contributions prepared
- [ ] References complete and in correct format ([style])
- [ ] All figures/tables referenced in text and numbered sequentially

## Compliance
- [ ] Ethics statement included
- [ ] Data availability statement included
- [ ] COI declaration included
- [ ] Funding statement included
- [ ] AI disclosure included (if applicable)
- [ ] Anonymization verified (if double-blind)

## Final Checks
- [ ] Manuscript spell-checked
- [ ] Page numbers visible
- [ ] Line numbers included (if required)
- [ ] File sizes within upload limits
- [ ] Suggested reviewers prepared (if required)
```

### Step 7: Suggested Reviewers

If the venue requests suggested reviewers:

| Criterion | Guidance |
|-----------|----------|
| **Expertise match** | Cite their relevant work in your paper |
| **Diversity** | Different institutions, countries, career stages |
| **No conflicts** | No co-authors, co-PIs, or institutional colleagues within last 3 years |
| **Recent activity** | Published in last 2–3 years in the field |
| **Typically 3–5** | Check venue requirement |

## Quality Bar

The submission package is **ready** when:

- [ ] Cover letter is venue-specific and sells the contribution
- [ ] Title page has all required elements (ORCID, CRediT, declarations)
- [ ] All venue-required statements are prepared
- [ ] Anonymization verified (if double-blind)
- [ ] Reporting checklist completed
- [ ] Submission checklist shows all items green
- [ ] All referenced supplementary materials are included
- [ ] File formats and sizes meet venue requirements

## Minimal Output Format

```markdown
# Submission Package Inventory

## Venue: [name]
## Submission type: [initial / revision]

## Files Prepared

| File | Path | Format | Status |
|------|------|--------|--------|
| Manuscript | submission/manuscript.pdf | PDF | ✅ |
| Cover letter | submission/cover_letter.md | MD → PDF | ✅ |
| Title page | submission/title_page.md | MD → PDF | ✅ |
| Supplementary | submission/supplementary.pdf | PDF | ✅ |
| Reporting checklist | reporting_checklist.md | MD | ✅ |

## Declarations Status

| Statement | Status | Notes |
|-----------|--------|-------|
| Ethics | ✅ | IRB #12345 |
| Data availability | ✅ | Repository DOI |
| COI | ✅ | None declared |
| Funding | ✅ | Grant #ABC |
| AI disclosure | ✅ | ChatGPT for editing |
| CRediT | ✅ | 2 authors |

## Anonymization: [Verified / N/A]
## Submission checklist: [All items green / n items pending]
```
