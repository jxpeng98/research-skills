---
id: rubric-mapper
stage: L_coursework
description: "Map marking rubrics and learning outcomes to required content, evidence, sections, and coursework risks."
inputs:
  - type: AssignmentBrief
    description: "Parsed assignment brief and constraints"
outputs:
  - type: RubricMap
    artifact: "assignment/rubric_map.md"
  - type: LearningOutcomeMap
    artifact: "assignment/learning_outcomes.md"
constraints:
  - "Must preserve rubric wording when supplied"
  - "Must mark missing criteria instead of inventing grading rules"
  - "Must not promise marks, grades, or pass/fail outcomes"
failure_modes:
  - "Converts a rubric into generic writing advice"
  - "Maps every learning outcome to the same paragraph"
  - "Treats optional guidance as mandatory assessment criteria"
tools: [filesystem]
tags: [coursework, rubric, learning-outcomes, assessment, mapping]
domain_aware: true
---

# Rubric Mapper Skill

Turn supplied marking criteria and learning outcomes into a practical coursework planning map.

## Purpose

Create traceable maps from assessment criteria to sections, evidence, source needs, and revision risks.

## When to Use

- After `assignment-brief-analyzer` when a rubric, marking criteria, or learning outcomes are available.
- Before outlining or revising coursework against an assessment brief.
- When the user asks whether a draft meets the rubric.

## Inputs

- `AssignmentBrief`: parsed prompt, constraints, missing fields, and integrity notes.
- Optional rubric or learning outcomes text.
- If rubric wording or learning outcomes are missing or insufficient, create a
  gap note and ask for them; do not turn generic guidance into criteria.

## Process

1. List every supplied rubric criterion verbatim or with clear paraphrase.
2. Convert each criterion into required capability, content, evidence, target section, and risk if missing.
3. Map each learning outcome to direct, indirect, or missing coverage.
4. Record missing criteria and policy gaps.

## Output Contract

Write `RESEARCH/[topic]/assignment/rubric_map.md` and `RESEARCH/[topic]/assignment/learning_outcomes.md`.

## Quality Bar

- [ ] Criteria are traceable to supplied rubric text.
- [ ] Missing criteria remain visible.
- [ ] Each learning outcome has a coverage status.
- [ ] No grade prediction is included.
- [ ] Each finding quotes or clearly paraphrases supplied wording;
  interpretation states the assessable capability; implication identifies
  required content or evidence without predicting a mark.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---|---|---|
| Grade prediction | Unsupported and risky | Use readiness language only |
| Generic mapping | Does not help revision | Tie each criterion to content and evidence |
| Hidden gaps | Encourages false readiness | Keep missing criteria explicit |
| Invented criteria | Misstates assessment authority | Never invent rules, rubric weights, citations, data, marks, or outcomes |
