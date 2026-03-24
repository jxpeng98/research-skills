---
id: submission-packager
stage: H_submission
version: "0.1.0"
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
failure_modes:
  - "Missing co-author information for disclosures"
  - "Venue requirements changed since analysis"
tools: [filesystem, submission-kit]
tags: [submission, cover-letter, checklist, disclosures, supplementary]
domain_aware: true
---

# Submission Packager Skill

Assemble a submission-ready package: cover letter, required statements, reporting checklist, and a final “submission checklist” to reduce desk-reject and revision cycles.

## When to Use

- You have a near-final manuscript draft
- You know the target venue (or have 2–3 candidates)

## Inputs (Ask / Collect)

- Target venue(s) + whether double-blind
- Manuscript type (full paper / short / letter / note)
- Required formatting constraints (word limit, references style)
- Required statements: COI, funding, data/code availability, ethics
- Supplementary materials (appendix, instruments, additional analyses)

## Process

1. **Venue fit & constraints**: scope alignment + formatting checklist
2. **Anonymization** (if double-blind): remove self-identifying details; anonymize repo links
3. **Reporting compliance**: run **reporting-checker** (and **prisma-checker** if SR)
4. **Open science package**: decide what to share now vs upon acceptance
5. **Final packaging**: compile all artifacts and a one-page checklist

## Outputs

- Cover letter → `RESEARCH/[topic]/submission/cover_letter.md` (use `templates/cover-letter.md`)
- Submission checklist → `RESEARCH/[topic]/submission/submission_checklist.md` (use `templates/submission-checklist.md`)
- Title page → `RESEARCH/[topic]/submission/title_page.md` (use `templates/title-page.md`)
- Highlights → `RESEARCH/[topic]/submission/highlights.md` (use `templates/highlights.md`)
- Suggested reviewers → `RESEARCH/[topic]/submission/suggested_reviewers.md` (use `templates/suggested-reviewers.md`)
- Author contributions (CRediT) → `RESEARCH/[topic]/submission/author_contributions_credit.md` (use `templates/author-contributions-credit.md`)
- Funding statement → `RESEARCH/[topic]/submission/funding_statement.md` (use `templates/funding-statement.md`)
- COI statement → `RESEARCH/[topic]/submission/coi_statement.md` (use `templates/coi-statement.md`)
- Data availability → `RESEARCH/[topic]/submission/data_availability.md` (use `templates/data-availability.md`)
- AI disclosure → `RESEARCH/[topic]/submission/ai_disclosure.md` (use `templates/ai-disclosure.md`)
- Supplementary inventory → `RESEARCH/[topic]/submission/supplementary_inventory.md` (use `templates/supplementary-inventory.md`)
- Reporting checklist (produced in stage `G1`) → `RESEARCH/[topic]/reporting_checklist.md` (use `templates/reporting-checklist.md`)
