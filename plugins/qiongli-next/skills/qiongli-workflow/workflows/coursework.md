---
description: Coursework and learning-assessment workflow for assignment briefs, rubrics, drafts, and final readiness checks
---

# Coursework Workflow

Support coursework planning, drafting, revision, and final readiness while preserving rubric fit, learning outcomes, source integrity, academic integrity, and institutional AI-policy limits.

Canonical Task IDs:
- `L1` assignment brief intake
- `L2` rubric and learning-outcome mapping
- `L3` coursework outline and structure plan
- `L4` coursework claim-evidence and citation plan
- `L5` coursework draft or section draft
- `L6` coursework revision against rubric
- `L7` coursework final readiness check

## Request

$ARGUMENTS

## Required Boundaries

- Do not promise marks or grades.
- Do not invent module rules, citations, sources, data, fieldwork, personal experience, or supervisor comments.
- Record missing rubric, learning outcome, source, and AI-policy information in `assignment/academic_integrity_notes.md`.
- Treat timed exams, quizzes, and assessed problem sets as concept-explanation requests, not coursework drafting requests.

## Choose the requested task

Use the brief, rubric, draft and decisions already supplied. A coursework label
does not start the full sequence. For a direct explanation or a small language
edit, answer in chat using the relevant reading/writing/proofreading skill.
Do not require a new brief, rubric map or project for an answer the material
already supports. Ask only for missing information that changes this task.

| Requested outcome | Entry |
|---|---|
| Interpret an assignment brief | L1 |
| Map marking criteria or learning outcomes | L2 |
| Plan structure or supporting evidence | L3 or L4 |
| Draft a specified section | L5, reusing the relevant rubric and evidence |
| Revise an existing draft against supplied criteria | L6 |
| Check final readiness | L7; unavailable required evidence remains a gap |

For a formal task, keep its canonical outputs and applicable integrity gates.
Reuse valid upstream artifacts; do not regenerate them just to enter later.
Registered project writes use preview/approval/CAS. Stop at the requested task.

## Full Workflow

Use the full sequence when a complete coursework workflow is requested:

1. Run `L1` to write `assignment/brief.md`.
2. Run `L2` to write `assignment/rubric_map.md` and `assignment/learning_outcomes.md`.
3. Run `L3` to write `coursework/outline.md` with structure matched to the assignment type.
4. Run `L4` to write `coursework/claim_evidence_plan.md` and `coursework/citation_plan.md`.
5. Run `L5` only after missing user facts, personal experience, data, and source gaps are marked.
6. Run `L6` to compare the draft against the rubric and learning outcomes.
7. Run `L7` to write `assignment/submission_checklist.md`.

Begin in preview mode. Ask for missing assignment rules rather than inventing them.
