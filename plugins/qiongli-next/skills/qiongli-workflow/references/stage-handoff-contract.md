# Stage Handoff Contract

Stage handoffs preserve what a downstream stage inherits and what remains uncertain.

## Canonical Path

- `RESEARCH/[topic]/context/stage_handoff.md`

## Required Sections

- `Completed Artifacts`
- `Decision Summary`
- `Resolved Grill Decisions`
- `Unresolved Questions`
- `Open Grill Issues`
- `Evidence Dependencies`
- `Assumptions Passed Forward`
- `Risks For Next Stage`
- `Revisit Triggers`
- `Recommended Next Tasks`

## Rules

- Link every completed artifact using a concrete `RESEARCH/[topic]/...` path or project-relative path.
- Record unresolved questions explicitly; write `None` only when the stage has been checked.
- Record `Resolved Grill Decisions` when a boundary interview, stage-aware grill, or self-critique loop closed a question that affects downstream scope, claim strength, methods, evidence thresholds, code, submission, or presentation.
- Record `Open Grill Issues` when a light automatic grill or deep grill found a risk that cannot be resolved in the current stage.
- Evidence dependencies should point to the evidence ledger, bibliography, analysis output, or gap note.
- `Revisit Triggers` must state what new evidence, user decision, reviewer comment, diagnostic failure, or analysis result would reopen a resolved decision.
- Do not treat a stage as ready for downstream work when inherited assumptions are hidden.
