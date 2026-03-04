---
description: 投稿前打包（reporting checklist + cover letter + submission checklist + statements）
argument-hint: [RESEARCH下topic文件夹名] [可选: venue]
---

# Submission Preparation

Assemble a submission-ready package for a target venue.

Canonical Task ID from `standards/research-workflow-contract.yaml`:
- `H1` submission package

## Target

$ARGUMENTS

## Workflow

### Step 0: Select Project Folder

Ask the user:
> "Which `RESEARCH/[topic]/` folder is this submission for?"

Ensure output structure exists:
```
RESEARCH/[topic]/submission/
├── cover_letter.md
├── submission_checklist.md
├── title_page.md
├── highlights.md
├── suggested_reviewers.md
├── author_contributions_credit.md
├── funding_statement.md
├── coi_statement.md
├── data_availability.md
├── ai_disclosure.md
└── supplementary_inventory.md
```

Also ensure reporting artifacts exist at project root:
```
RESEARCH/[topic]/reporting_checklist.md
```

### Step 1: Collect Submission Requirements

Ask:
1. Target venue + manuscript type
2. Double-blind? (Y/N)
3. Word/page limits + formatting constraints
4. Required statements (ethics, COI, funding, data/code availability)
5. Any required reporting guideline checklist upload? (Y/N)

Also ask where the current manuscript draft lives (path or paste).

### Step 2: Reporting Compliance Check

Use **reporting-checker** to generate:
- `RESEARCH/[topic]/reporting_checklist.md` (use `templates/reporting-checklist.md`)

If this is a systematic review, run **prisma-checker** instead (or in addition).

### Step 3: Package Submission Artifacts

Use **submission-packager** to draft:
- `RESEARCH/[topic]/submission/cover_letter.md` (use `templates/cover-letter.md`)
- `RESEARCH/[topic]/submission/submission_checklist.md` (use `templates/submission-checklist.md`)
- `RESEARCH/[topic]/submission/title_page.md` (use `templates/title-page.md`)
- `RESEARCH/[topic]/submission/highlights.md` (use `templates/highlights.md`)
- `RESEARCH/[topic]/submission/suggested_reviewers.md` (use `templates/suggested-reviewers.md`)
- `RESEARCH/[topic]/submission/author_contributions_credit.md` (use `templates/author-contributions-credit.md`)
- `RESEARCH/[topic]/submission/funding_statement.md` (use `templates/funding-statement.md`)
- `RESEARCH/[topic]/submission/coi_statement.md` (use `templates/coi-statement.md`)
- `RESEARCH/[topic]/submission/data_availability.md` (use `templates/data-availability.md`)
- `RESEARCH/[topic]/submission/ai_disclosure.md` (use `templates/ai-disclosure.md`)
- `RESEARCH/[topic]/submission/supplementary_inventory.md` (use `templates/supplementary-inventory.md`)

Begin submission preparation now.
