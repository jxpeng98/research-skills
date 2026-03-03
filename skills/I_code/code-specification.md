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

## Minimal spec format (`code/code_specification.md`)

```markdown
# Code Specification

## Goal

## Inputs (schema)
- ...

## Outputs (paths)
- ...

## Functional requirements
1. ...

## Non-functional requirements
- Performance:
- Determinism:
- Logging:

## Edge cases
- ...

## Validation plan
- Unit tests:
- Synthetic verification:
- Benchmarks:
```
