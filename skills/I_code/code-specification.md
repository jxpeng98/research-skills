---
id: code-specification
stage: I_code
version: "0.2.1"
description: "Generate strict constraint sets and requirement specifications before coding, following OPSX-style methodology."
inputs:
  - type: UserQuery
    description: "High-level method or feature description"
outputs:
  - type: CodeSpec
    artifact: "code/code_specification.md"
constraints:
  - "Must produce machine-parseable constraints"
  - "Must identify all external dependencies"
failure_modes:
  - "Ambiguous requirements leading to specification gaps"
  - "Over-specification constraining implementation flexibility"
tools: [filesystem]
tags: [code, specification, requirements, OPSX, constraints]
domain_aware: false
---

# Code Specification Skill

Generate a strict, testable constraint set before writing research code.

## Related Task IDs

- `I5` (code specification)

## Output (contract path)

- `RESEARCH/[topic]/code/code_specification.md`

## Inputs (ask for)

- What needs to be implemented (method/pipeline/reproduction target)
- Expected inputs/outputs (file formats, schemas, sample data)
- Constraints: runtime, memory, dependency limits, hardware (CPU/GPU)
- Required validation: unit tests, benchmarks, replication targets

## Procedure (low freedom)

1. **Define success criteria**: what counts as “correct” (and what doesn’t).
2. **Specify I/O contracts**:
   - input files/columns/shapes
   - output artifacts (paths + formats)
3. **List invariants**: properties that must always hold (e.g., probability sums to 1).
4. **Enumerate edge cases**: missing data, empty groups, small-k meta-analysis, non-convergence.
5. **Allowed dependencies**: language + libraries + version constraints.
6. **Determinism rules**: seeds, randomness sources, hardware nondeterminism notes.
7. **Validation plan**:
   - synthetic data checks
   - unit tests for core functions
   - regression tests for known examples

## Required spec format (`code/code_specification.md`)

```markdown
---
task_id: I5
template_type: code_specification
topic: <topic>
primary_artifact: code/code_specification.md
---

# Code Specification

## Spec Contract Block
```json
{
  "task_id": "I5",
  "topic": "<topic>",
  "method_or_pipeline": "<name>",
  "primary_artifact": "code/code_specification.md",
  "inputs": [
    {"path": "...", "schema": "..."}
  ],
  "outputs": [
    {"path": "...", "format": "..."}
  ],
  "dependencies": {
    "python": ["package>=version"]
  },
  "seeds_policy": {
    "global_seed": "...",
    "nondeterminism_notes": "..."
  },
  "acceptance_tests": [
    {"name": "...", "metric": "...", "pass_condition": "..."}
  ],
  "blocked_decisions": ["..."]
}
```

## Goal
- ...

## Non-Goals
- ...

## Inputs (Schema)
- ...

## Outputs (Paths)
- ...

## Functional Requirements
1. ...

## Non-Functional Requirements
- Performance:
- Determinism:
- Logging:

## Edge Cases And Failure Modes
- ...

## Validation Matrix
| Check | Metric / Observable | Pass Condition | Artifact |
| --- | --- | --- | --- |
| ... | ... | ... | ... |

## Disallowed Shortcuts
- ...

## Blocked Decisions / Escalations
- ...
```
