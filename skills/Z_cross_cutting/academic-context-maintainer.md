---
id: academic-context-maintainer
stage: Z_cross_cutting
description: "Maintain project-level academic continuity by updating research state, locked decisions, disputes, and next-stage priorities across major workflow transitions."
inputs:
  - type: TaskPacket
    description: "Current stage/task packet plus latest stage artifacts"
  - type: AnyArtifact
    description: "Any artifact that changes the project's academic state"
outputs:
  - type: ResearchStateSnapshot
    artifact: "context/research_state.md"
  - type: ResearchDecisionLog
    artifact: "context/decision_log.md"
constraints:
  - "Must distinguish stable findings from tentative leads or open questions"
  - "Must anchor every locked decision to concrete upstream artifacts or task IDs"
  - "Must summarize unresolved disputes instead of silently collapsing disagreement"
failure_modes:
  - "Generic summaries that restate workflow steps without preserving academic meaning"
  - "Decision log entries that omit rationale, rejected alternatives, or revisit triggers"
tools: [filesystem]
tags: [cross-cutting, context, continuity, state-tracking, decision-log, handoff]
domain_aware: true
---

# Academic Context Maintainer Skill

Project-level academic continuity layer for long-running research workflows.

This skill is not a runtime memory compactor. It maintains the research state that should survive across stage changes, long conversations, model switches, and revision cycles.

## Purpose

Keep the project from losing its academic thread by updating two canonical continuity artifacts:

- `context/research_state.md`
- `context/decision_log.md`

The goal is to preserve:
- the current research question, thesis, or focal puzzle
- scope boundaries and contested definitions
- locked methodological or interpretive decisions
- stable findings versus tentative leads
- unresolved disputes, nulls, contradictions, and fragility points
- next-stage priorities and what must not be forgotten

## When to Use

- after a major stage has materially changed the project's academic state
- when the conversation has become long enough that the research logic risks drifting
- when different models or collaborators need the same project-level academic frame
- before moving from discovery to design, design to execution, synthesis to writing, or writing to submission
- whenever the project has accumulated new constraints, contradictions, or irreversible choices that must be recorded explicitly

## Not the Same as Runtime Memory

Do not confuse this skill with:

- `compact`: conversation/window compression
- `handoff trace`: runtime ownership and orchestration lineage
- `resume_state`: workflow checkpointing for incomplete execution steps

This skill records academic state, not shell/runtime state.

## Canonical Outputs

### 1. `context/research_state.md`

Use this file as the current authoritative academic snapshot.

It should answer:
- What exact question is this project now trying to answer?
- What definitions and boundaries are locked?
- What is currently well-supported?
- What remains uncertain or contested?
- What has to be preserved in the next stage?

### 2. `context/decision_log.md`

Use this file as the auditable ledger of major choices.

Each entry should make it clear:
- what was decided
- why it was decided
- what alternatives were considered and rejected
- what evidence justified the decision
- what would trigger reopening it later

## Stage Refresh Matrix

Refresh the continuity layer at these stage transitions:

| Transition | What must be preserved in `research_state.md` | What must be logged in `decision_log.md` |
|---|---|---|
| `A` close (`A5`) | Working RQ, focal constructs, scope boundaries, contribution thesis, venue assumptions | Why the RQ was narrowed this way, which constructs/frames were rejected, what venue logic matters |
| `B` close (`B6`) | Literature topology, inclusion/exclusion logic, dominant theories, contradictory streams, evidence gaps | Why search/screening boundaries were set, why certain literatures were excluded, how key concepts were normalized |
| `C` close (`C5`) | Design commitments, identification logic, measurement choices, dataset constraints, prereg-ready decision rules | Why this design was chosen over rivals, what assumptions are now binding, what design risks remain |
| `D` close (`D3`) | Ethics constraints, privacy boundaries, deidentification commitments, access restrictions | Why governance/privacy constraints altered the plan, what cannot be collected/shared, what review conditions must persist |
| `E` close (`E5`) | Stable findings, nulls, heterogeneity, certainty limits, unresolved tensions | Why the current evidence position is credible, what synthesis decisions were made, what contrary evidence remains open |
| `F` close (`F6`) | Manuscript thesis, claims-evidence alignment status, interpretation boundaries, narrative fragilities | Why the manuscript now makes these specific claims, what was deliberately softened, what still feels overclaimed or thin |
| `H` close (`H4`) | Submission-readiness view, reviewer-sensitive weaknesses, revision priorities, fatal risks | Why the team is or is not ready to submit, which flaws are accepted vs. blocking, what would force a major revision |

## Process

### 1. Identify the transition and its source artifacts

Before writing anything, determine:

- which stage transition is being updated
- which artifacts are now authoritative
- which prior state entries are still valid
- which prior state entries must be overwritten or marked stale

Never summarize from memory alone. Read the stage outputs that actually changed the project state.

### 2. Update `context/research_state.md`

Write only project-level academic state. Avoid procedural chatter.

Required sections:

```markdown
# Research State

## Paper Identity
- topic:
- paper_type:
- target_venue:
- current_stage:
- last_updated_by_task:

## Current Research Question / Thesis
- main_question_or_thesis:
- why_this_question_now:

## Scope Boundaries and Definitions
- included_scope:
- excluded_scope:
- contested_terms_and_working_definitions:

## Locked Decisions
| Decision area | Current position | Confidence | Source artifacts |
|---|---|---|---|

## Current Evidence Position
- strongest_supported_claims:
- claims_not_yet_stable:
- contradictory_or_null_evidence:

## Active Risks and Fragility Points
- conceptual_risks:
- methodological_risks:
- evidentiary_risks:
- submission_risks:

## Next-Stage Priorities
1. ...
2. ...
3. ...

## Source Artifact Anchors
- `task_id`:
- `artifact_paths`:
- `what_changed_the_state`:
```

### 3. Update `context/decision_log.md`

Log only decisions with downstream consequences. Do not turn this into a generic activity diary.

Recommended structure:

```markdown
# Research Decision Log

| Decision ID | Stage | Status | Decision | Rationale | Alternatives Rejected | Evidence Basis | Revisit Trigger | Downstream Impact |
|---|---|---|---|---|---|---|---|---|
| DEC-001 | A | locked | ... | ... | ... | `framing/research_question.md`; `gap_analysis.md` | New evidence invalidates boundary | Changes search strategy and contribution framing |
```

Use status values such as:
- `locked`
- `tentative`
- `revisit-after-B`
- `revisit-after-E`
- `blocked`

### 4. Preserve disagreement instead of flattening it

If collaborators or models disagree, record:

- what the disagreement is actually about
- which interpretation is currently preferred
- why it is preferred
- what evidence would resolve the disagreement later

Do not erase disagreement just to make the state file look tidy.

### 5. Write for academic handoff, not personal memory

A new researcher should be able to open the two continuity files and understand:

- the current intellectual position of the project
- the most binding decisions already made
- the most dangerous assumptions still in play
- what the next stage must preserve or challenge

## Output Discipline

Good continuity updates are:

- selective rather than exhaustive
- anchored to artifacts and task IDs
- explicit about uncertainty
- specific about what changed since the previous update
- written in the language of claims, evidence, scope, and decisions

Bad continuity updates:

- retell the full workflow chronologically
- copy entire manuscript paragraphs
- hide uncertainty behind vague confidence language
- blur stable findings with speculative ideas
- record execution trivia that belongs in logs, not research state

## Templates to Use

- `templates/research-state.md`
- `templates/decision-log.md`

## Quality Bar

- [ ] `research_state.md` distinguishes stable claims from tentative leads
- [ ] every major locked decision is backed by specific source artifacts
- [ ] unresolved disputes are visible rather than silently collapsed
- [ ] next-stage priorities are concrete and stage-specific
- [ ] `decision_log.md` contains revisit triggers, not just rationales

## Common Pitfalls

| Pitfall | Problem | Fix |
|---|---|---|
| Summary masquerading as continuity | Restates what was done, not what now matters | Rewrite in terms of question, scope, decisions, evidence, disputes, and next priorities |
| False certainty | Treats all current findings as settled | Split stable findings from provisional interpretations |
| Missing rationale | Decision log records conclusions without reasons | Add evidence basis, rejected alternatives, and downstream impact |
| Drift from source artifacts | Continuity file no longer matches the current stage outputs | Re-anchor every update to the latest authoritative artifacts |
| Overloaded state file | Mixes runtime/debug notes into academic continuity | Keep orchestration notes in logs and only academic state in `context/` |
