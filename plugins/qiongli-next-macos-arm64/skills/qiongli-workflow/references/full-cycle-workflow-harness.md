# Full-Cycle Workflow Harness

The Full-Cycle workflow harness keeps long-running paper work aligned across
topic framing, literature search, design, data, writing, compliance, review,
journal fit, and feedback. It is a contract for lifecycle planning and gate
review; implementation-specific preview tools may automate these checks, but
the artifact rules below are authoritative for the workflow entrypoint.

## Required State Files

Every lifecycle pass must read the current research state before recommending a
next task. If an expected file is missing, the lifecycle status must record the
gap instead of assuming the content exists.

- `context/research_state.md`
- `context/decision_log.md`
- `context/boundary_review.md`
- `context/stage_handoff.md`
- `evidence/claim-evidence-ledger.csv`

## Gate Decisions

Each checkpoint reports one gate decision:

- `not_started`: no evidence shows that the checkpoint has begun.
- `blocked_missing_artifact`: a required artifact is absent or unreadable.
- `blocked_unresolved_boundary`: a locked decision, non-goal, evidence
  threshold, or claim-strength boundary is missing.
- `blocked_unresolved_judge`: H3 or H4 found a major or fatal issue that
  remains open.
- `ready_for_agent`: prerequisites are present and the next task can be
  assigned if `run_agents: true`.
- `ready_for_human_review`: machine or role checks are complete, but a human
  decision is needed before proceeding.
- `passed`: required artifacts and drift checks are satisfied.
- `reopened_by_revisit_trigger`: new evidence or feedback invalidated an
  earlier decision and a prior stage must be revisited.

## Lifecycle Checkpoints

| Checkpoint | Main tasks | Required proof |
|---|---|---|
| Idea lock | `A1`, `A2`, `A4`, `A5` | research question, contribution, boundary review, initial venue assumptions |
| Evidence base | `B1`, `B2`, `B3`, `B6` | search plan, search log, dedup log, retrieval manifest, literature map |
| Design and data | `C1`, `C3`, `C4`, optional `I3-I8` | study design, variable or construct spec, data plan, analysis or reproducibility status |
| Manuscript build | `F1-F6` | outline, draft, claim-evidence map, figures or tables plan |
| Compliance and proofread | `G1-G4`, `J1-J4` | reporting checklist, cross-section integrity, tone and proofread reports |
| Strong judge | `H3`, `H4` | peer review simulation and fatal flaw analysis |
| Journal fit | `H5` | ranked journal fit report with evidence-based recommendation |
| Feedback loop | `H2`, `H2_5` | response matrix, revision plan, reviewer empathy check |

## Drift Checks

The harness must block or warn when downstream work diverges from locked
decisions:

- The manuscript changes the locked research question without a
  `context/decision_log.md` entry.
- Claims in `manuscript/manuscript.md` have no row in
  `evidence/claim-evidence-ledger.csv`.
- A methods, data, or generalizability claim exceeds the current Stage C or
  Stage I evidence status.
- A journal recommendation ignores limitations, fatal flaws, reporting gaps, or
  the current maturity of the manuscript.
- H3 or H4 reports a major or fatal issue while lifecycle status still says the
  project is submission-ready.
- A revision promise has no source artifact, feasibility note, or stage reopen
  decision.
- A role output omits locked non-goals, evidence thresholds, or claim-strength
  boundaries from `context/boundary_review.md`.

## Strong Judge Rule

The strong judge is a gate, not a coauthor. It may return only:

- `pass`
- `revise`
- `block_submission`
- `reopen_stage`

Every strong judge decision must identify the affected claim or artifact, the
evidence basis, missing artifact if any, required revision, stage to reopen if
blocked, and whether an existing journal recommendation remains valid.

The strong judge must not draft manuscript text, invent reviewer comments,
recommend a venue without reading the H5 report, or override a locked decision
without a revisit trigger.

## H5 Journal Fit Readiness

H5 is ready only when the lifecycle package includes a manuscript draft or
structured sections, contribution statement, methods or evidence design
summary, claim-evidence map, limitations or fatal flaw status, and venue
profile evidence. If these inputs are missing, write the gap and block any
best-journal claim.
