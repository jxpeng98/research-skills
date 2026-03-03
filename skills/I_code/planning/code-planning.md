---
id: code-planning
stage: I_code
version: "1.0.0"
description: "Transform code specifications into parallelizable, zero-decision execution plans."
inputs:
  - type: CodeSpec
    description: "Code specification with constraints"
outputs:
  - type: CodePlan
    artifact: "code/plan.md"
constraints:
  - "Must produce plans that require zero decisions during execution"
  - "Must identify parallelizable tasks"
failure_modes:
  - "Dependencies prevent meaningful parallelization"
  - "Plan too granular for practical execution"
tools: [filesystem]
tags: [code, planning, execution-plan, parallelization]
domain_aware: false
---

# Code Planning Skill

Transform a specification into a zero-decision execution plan that can be parallelized and audited.

## Related Task IDs

- `I6` (code planning)

## Output (contract path)

- `RESEARCH/[topic]/code/plan.md`

## Inputs (minimum)

- `code/code_specification.md` (or an equivalent spec in the prompt)
- Current repository/project structure under `RESEARCH/[topic]/`

## Procedure

1. **Decompose** into small tasks (≤ 30–60 min each).
2. **Order** tasks by dependencies and risk (do the riskiest first).
3. **Define checkpoints** with concrete “pass/fail” criteria.
4. **Plan artifacts**: where code, data, plots, and reports will be written.
5. **Plan commands**: exact commands to run for each checkpoint.

## Minimal plan format (`code/plan.md`)

```markdown
# Execution Plan

## Scope

## Task list
1. [ ] Task (owner/agent) — output — checkpoint

## Checkpoints
- CP1: ...
- CP2: ...

## Run commands
- ...

## Risks & mitigations
- ...
```
