---
id: coursework-reviser
stage: L_coursework
description: "Revise coursework drafts against rubrics, learning outcomes, evidence plans, word count, and integrity constraints."
inputs:
  - type: CourseworkOutline
    description: "Coursework structure plan"
  - type: CourseworkClaimEvidencePlan
    description: "Claim, evidence, and citation plan"
  - type: RubricMap
    description: "Rubric criteria and risks"
outputs:
  - type: CourseworkRevisionPlan
    artifact: "coursework/revision_plan.md"
  - type: CourseworkSubmissionChecklist
    artifact: "assignment/submission_checklist.md"
constraints:
  - "Must not guarantee marks or grades"
  - "Must flag unsupported claims and missing user material"
  - "Must preserve academic-integrity notes"
failure_modes:
  - "Rewrites beyond supplied evidence"
  - "Treats a checklist as a grade prediction"
  - "Deletes uncertainty markers that protect integrity"
tools: [filesystem]
tags: [coursework, revision, final-checklist, rubric, integrity]
domain_aware: true
---

# Coursework Reviser Skill

Review and revise coursework against the assignment's explicit assessment constraints.

## Purpose

Revise the requested coursework material against supplied criteria without
promising grades. A formal L6 run produces its revision plan and checklist.

## When to Use

- When the user has a draft and wants rubric-facing revision.
- Before final submission checks.
- When learning outcomes, word count, citation coverage, or integrity notes need a final pass.

## Inputs

For a direct passage revision, use the supplied draft, relevant rubric criteria
and evidence. Return the requested text/advice in chat; do not require a new
outline, evidence plan or rubric-map file when those materials already settle
the requested correction. Missing relevant evidence still blocks the affected
claim. The declared inputs and saved outputs below apply to formal L6 runs.

- `CourseworkOutline`
- `CourseworkClaimEvidencePlan`
- `RubricMap`
- Optional draft text, learning outcome map, citation plan, and academic-integrity notes.
- If the draft, rubric, evidence plan, user material, or cited source is missing
  or insufficient, record a gap note and leave the affected revision blocked.

## Process

1. Compare draft coverage to rubric criteria.
2. Check learning outcome coverage and word-count pressure.
3. Identify unsupported claims, missing citations, and missing user-supplied facts.
4. Produce concrete revisions and blocked items.
5. For formal L6, create a submission checklist with visible unresolved risks.
   For a direct revision, stop after checking the requested changes; do not
   extend a passage correction into whole-assignment readiness.

## Output Contract

For formal L6, write `RESEARCH/[topic]/coursework/revision_plan.md` and
`RESEARCH/[topic]/assignment/submission_checklist.md`. Registered project writes
use preview/approval/CAS. Direct answers do not claim formal task completion.

## Quality Bar

- [ ] Revision advice maps to rubric criteria.
- [ ] Unsupported claims are flagged.
- [ ] AI policy and integrity notes are preserved.
- [ ] No grade guarantee appears.
- [ ] Each finding identifies draft evidence; interpretation explains the
  rubric-facing weakness at the evidence's actual strength; implication gives
  a bounded revision or an explicit blocker.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---|---|---|
| "This will get an A" | Unsupported and unsafe | Use readiness and risk language |
| Evidence-free polish | Makes prose smoother but weaker | Prioritize claim support |
| Hiding unresolved gaps | Creates false confidence | Keep blocked items explicit |
| Filling evidence gaps | Fabricates support | Never invent citations, sources, data, user facts, statistics, or results |
