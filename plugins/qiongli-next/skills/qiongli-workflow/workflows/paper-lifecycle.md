---
description: Full-cycle academic paper workflow harness from topic selection to journal fit and feedback.
---

# Full-Cycle Paper Lifecycle Workflow

Use this workflow when the user wants Qiongli to coordinate the whole paper
pipeline rather than one isolated task.

## Inputs

$ARGUMENTS

## Contract

Read `references/workflow-contract.md`,
`references/stage-handoff-contract.md`, and
`references/full-cycle-workflow-harness.md` before producing a lifecycle plan.

The workflow is preview-first. Default to a planning and gate-review report.
Do not launch local agents unless the caller explicitly sets `run_agents: true`.

## Required Checkpoints

1. Stage A: topic, research question, contribution, boundary review, and
   initial venue assumptions.
2. Stage B: broad literature search, search logs, deduplication, full-text
   status, and Zotero or retrieval evidence where available.
3. Stage C/I: study design, data plan, analysis plan, or reproducibility
   status.
4. Stage F: manuscript outline, draft, claim-evidence map, figures or tables
   plan.
5. Stage G/J: reporting compliance, cross-section integrity, proofreading, and
   citation-risk checks when material.
6. Stage H: peer review simulation, fatal flaw analysis, and H5 reverse journal
   fit.
7. Feedback loop: response matrix, revision plan, reviewer empathy check, and
   stage reopen decisions.

## Output

Produce a lifecycle plan that lists:

- current lifecycle status,
- passed and blocked stage gates,
- missing artifacts,
- drift risks,
- recommended next task IDs,
- whether H5 journal fit is ready,
- whether `submission/journal_fit_recommendation.md` can be produced, and
- what must be written to `context/stage_handoff.md` before the next stage.

If H5 is not ready, explain which manuscript, method, evidence, limitation, or
venue-profile input is missing. Do not claim a best journal until H5 has enough
manuscript-first evidence to classify venues.

## Optional Execution

When `run_agents: true`, keep roles bounded by the lifecycle contract:

- The controller owns the lifecycle plan and gate decisions.
- Evidence, methods, writing, review, strong judge, and journal-fit roles may
  contribute only within their assigned checkpoints.
- The strong judge can pass, revise, block submission, or reopen a stage; it
  must not draft manuscript text.

If `run_agents` is absent or false, stay in preview mode and return the plan,
gate report, and next actions only.
