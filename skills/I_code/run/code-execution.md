---
id: code-execution
stage: I_code
version: "1.0.0"
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

## Performance profile format (`code/performance_profile.md`)

```markdown
# Performance Profile

## Environment
- OS:
- Python/R version:
- Key deps:

## Workload
- Dataset size:
- Command:

## Results
| Component | Time | Notes |
|---|---:|---|

## Optimization actions taken
1. ...
```
