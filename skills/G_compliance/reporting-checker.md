---
id: reporting-checker
stage: G_compliance
version: "0.2.0"
description: "Validate reporting guideline completeness for target study type (CONSORT, STROBE, COREQ, SRQR, TRIPOD, etc.)."
inputs:
  - type: Manuscript
    description: "Draft manuscript"
  - type: DesignSpec
    description: "Study design for guideline selection"
outputs:
  - type: ReportingChecklist
    artifact: "reporting_checklist.md"
constraints:
  - "Must select appropriate guideline based on study design"
  - "Must reference specific manuscript sections for each item"
failure_modes:
  - "No standard guideline exists for the study type"
  - "Multiple guidelines applicable with conflicting requirements"
tools: [filesystem, reporting-guidelines]
tags: [compliance, reporting, CONSORT, STROBE, COREQ, guidelines]
domain_aware: true
---

# Reporting Checker Skill

Check whether an empirical or qualitative manuscript/report is complete and aligned with an appropriate reporting guideline, producing a structured “what’s missing” action list.

## When to Use

- Before submission (final QA)
- Before sharing a preprint
- When converting notes into a paper draft

## Guideline Selection (Heuristic)

Pick the closest match (confirm with the target venue):
- Systematic review → PRISMA (use **prisma-checker**)
- Randomized trial → CONSORT
- Observational (cohort/case-control/cross-sectional) → STROBE
- Qualitative interview/focus group studies → COREQ
- Broader qualitative studies (case study, ethnography, process research, document-heavy qualitative designs) → SRQR, with venue-specific transparency addenda when required
- Prediction model development/validation → TRIPOD

If the design is mixed, apply the dominant design’s checklist and add a “design-specific addendum”.

## Process

1. Identify study design and core claims (causal/descriptive/predictive)
2. Select guideline(s) + justify selection
3. Map checklist items to manuscript sections
4. Record missing/partial items and what evidence is needed
5. Generate prioritized fixes (must-fix vs nice-to-have)

## Outputs

- Reporting checklist + fixes → `RESEARCH/[topic]/reporting_checklist.md` (use `templates/reporting-checklist.md`)

## Notes

- Keep this checklist **lightweight** in-repo; confirm final items against the official guideline required by the venue.
