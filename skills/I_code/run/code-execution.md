---
id: code-execution
stage: I_code
version: "0.2.0"
description: "Execute code plans with performance profiling (cProfile/vectorization) and optimization."
inputs:
  - type: CodePlan
    description: "Execution plan to follow"
outputs:
  - type: PerformanceProfile
    artifact: "code/performance_profile.md"
constraints:
  - "Must follow plan exactly without improvisation"
  - "Must capture performance metrics"
failure_modes:
  - "Runtime errors from environment mismatch"
  - "Performance bottlenecks in unoptimized code"
tools: [filesystem, code-runtime]
tags: [code, execution, profiling, performance, optimization]
domain_aware: false
---

# Code Execution Skill

Execute the plan: implement, test, profile, and document research code with reproducible outputs.

## Related Task IDs

- `I7` (code execution)

## Outputs (contract paths)

- `RESEARCH/[topic]/analysis/`
- `RESEARCH/[topic]/code/performance_profile.md`
- `RESEARCH/[topic]/code/container_config/` (optional but recommended)
- `RESEARCH/[topic]/code/documentation/`

## Inputs

- `code/plan.md`
- Any required datasets (or a synthetic generator for verification)

## Procedure

1. **Implement incrementally** with tests after each unit.
2. **Validate** against the spec (I/O, invariants, edge cases).
3. **Add a minimal runnable entrypoint** (script or CLI).
4. **Profile** hot paths (time + memory) and record results.
5. **Document** how to reproduce:
   - dependencies
   - commands
   - expected outputs

## Required performance profile format (`code/performance_profile.md`)

```markdown
---
task_id: I7
template_type: performance_profile
topic: <topic>
primary_artifact: code/performance_profile.md
---

# Performance Profile

## Execution Contract Block
```json
{
  "task_id": "I7",
  "topic": "<topic>",
  "plan_source": "code/plan.md",
  "performance_artifact": "code/performance_profile.md",
  "analysis_outputs": ["analysis/..."],
  "documentation_outputs": ["code/documentation/..."],
  "container_outputs": ["code/container_config/..."],
  "validation_runs": [
    {"step_id": "S1", "evidence": "..."}
  ],
  "profiling_targets": [
    {"component": "...", "command": "..."}
  ]
}
```

## Scope Executed
- ...

## Implementation Ledger
| Step ID | Planned Output | Observed Output | Status | Notes |
| --- | --- | --- | --- | --- |
| S1 | ... | ... | PASS | ... |

## Validation Evidence
| Check | Evidence | Result | Artifact |
| --- | --- | --- | --- |
| ... | ... | ... | ... |

## Artifact Inventory
- `analysis/...`
- `code/documentation/...`
- `code/container_config/...`

## Environment / Containerization
- OS:
- Python/R version:
- Key deps:

## Profiling Results
- Dataset size:
- Command:
| Component | Time | Notes |
|---|---:|---|

## Optimization Actions Taken
1. ...

## Reproduction Commands
1. ...

## Remaining Gaps / Blockers
- ...
```
