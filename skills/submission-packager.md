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
- Reporting checklist → `RESEARCH/[topic]/submission/reporting_checklist.md` (use `templates/reporting-checklist.md`)

