---
id: analysis-interpreter
stage: F_writing
version: "1.0.0"
description: "Translate statistical output into result narratives that preserve uncertainty, assumptions, and robustness boundaries."
inputs:
  - type: StatsReport
    description: "Model results, diagnostics, and robustness checks"
  - type: AnalysisPlan
    description: "Pre-specified estimands and decision rules"
    required: false
  - type: RobustnessPlan
    description: "Planned robustness checks and threats"
    required: false
outputs:
  - type: ResultInterpretation
    artifact: "manuscript/results_interpretation.md"
constraints:
  - "Must distinguish descriptive findings from causal interpretation"
  - "Must note uncertainty, assumptions, and failed robustness checks"
  - "Must avoid re-litigating the entire methods section"
failure_modes:
  - "Narrative overclaims beyond the estimator and identification strategy"
  - "Null or imprecise findings are reframed as support without justification"
tools: [filesystem]
tags: [writing, results, interpretation, robustness, uncertainty]
domain_aware: true
---

# Analysis Interpreter Skill

Turn model output into manuscript-ready result interpretation.

## Related Task IDs

- `F3` (full manuscript draft)

## Output (contract path)

- `RESEARCH/[topic]/manuscript/results_interpretation.md`

## Procedure

1. **Read the result pattern**:
   - Primary estimate
   - Precision and uncertainty
   - Diagnostics and robustness
2. **Write interpretation at the right level**:
   - What changed
   - For whom / when
   - Under which assumptions
3. **Flag limitations**:
   - Sensitivity to specification
   - Identification caveats
   - External validity constraints

## Minimal Output Format

```markdown
# Results Interpretation

## Main Findings
- ...

## Uncertainty and Robustness
- ...

## Boundary Conditions
- ...
```
