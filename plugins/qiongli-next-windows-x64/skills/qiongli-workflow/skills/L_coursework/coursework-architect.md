---
id: coursework-architect
stage: L_coursework
description: "Design coursework structure, claim-evidence plans, and citation plans from assignment briefs and rubrics."
inputs:
  - type: AssignmentBrief
    description: "Parsed coursework brief"
  - type: RubricMap
    description: "Assessment criteria mapped to sections and evidence needs"
  - type: LearningOutcomeMap
    description: "Learning outcome coverage map"
outputs:
  - type: CourseworkOutline
    artifact: "coursework/outline.md"
  - type: CourseworkClaimEvidencePlan
    artifact: "coursework/claim_evidence_plan.md"
constraints:
  - "Must choose structure based on assignment type, not default manuscript IMRaD"
  - "Must distinguish user-supplied personal evidence from source-supported academic claims"
  - "Must mark citation and evidence gaps before drafting"
failure_modes:
  - "Forces all coursework into a journal paper structure"
  - "Creates claims without evidence thresholds"
  - "Ignores word count and rubric weighting"
tools: [filesystem]
tags: [coursework, outline, claim-evidence, citation-plan, structure]
domain_aware: true
---

# Coursework Architect Skill

Build coursework outlines and claim-evidence plans that fit the assignment, not a generic paper template.

## Purpose

Write `coursework/outline.md`, `coursework/claim_evidence_plan.md`, and a citation plan when required.

## When to Use

- After the brief and rubric have been parsed.
- Before drafting an essay, report, case analysis, reflection, literature review, proposal, presentation, portfolio, or capstone component.

## Inputs

- `AssignmentBrief`
- `RubricMap`
- `LearningOutcomeMap`
- Optional user notes, sources, data, and draft fragments.
- If a brief, rubric, learning outcome, user fact, or source is missing or
  insufficient, write a gap note and block only the affected claim or section.

## Process

1. Select the correct assignment structure family.
2. Allocate word count and rubric emphasis across sections.
3. State each section's job, central claim, evidence threshold, and missing material.
4. Build a citation plan for source-supported claims.
5. Block drafting where required user facts or source evidence are missing.

## Output Contract

Write `RESEARCH/[topic]/coursework/outline.md` and `RESEARCH/[topic]/coursework/claim_evidence_plan.md`. If citations are material, also write `RESEARCH/[topic]/coursework/citation_plan.md`.

## Quality Bar

- [ ] Structure matches assignment type.
- [ ] Claims map to evidence, sources, or user-supplied material.
- [ ] Rubric and learning outcomes are visible in section planning.
- [ ] Missing evidence is not silently filled.
- [ ] Each finding names the supplied requirement or evidence; interpretation
  calibrates the supported claim; implication identifies the section-level
  drafting action.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---|---|---|
| IMRaD by default | Misfits essays and case work | Choose structure by assignment type |
| Citation dumping | Weak synthesis | Tie each source to a claim |
| Overlong plan | Fails word count | Allocate word budget early |
| Invented support | Creates false evidence | Never invent citations, sources, data, personal facts, or results |
