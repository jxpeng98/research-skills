---
id: analysis-interpreter
stage: F_writing
version: "1.1.0"
description: "Translate empirical or synthesized findings into analytical narratives that preserve uncertainty, surface mechanisms, and narrow claims to defensible scope conditions."
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
  - "Must separate observation, interpretation, and implication"
  - "Must surface mechanism candidates, rival explanations, and boundary conditions when evidence permits"
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
   - Null, contradictory, or heterogeneous findings
   - Diagnostics and robustness
2. **Climb the interpretive depth ladder**:
   - What happened?
   - Why might it have happened?
   - What else could explain it?
   - When or where does the claim narrow or break?
   - Why does it matter theoretically, methodologically, or practically?
3. **Write interpretation at the right level**:
   - What changed
   - For whom / when
   - Under which assumptions
   - Compared with which baseline, prior stream, or expectation
4. **Flag limitations**:
   - Sensitivity to specification
   - Identification caveats
   - External validity constraints

For qualitative or mixed-method evidence, do not simply restate themes, quotes, or codes.
Translate them into analytic claims about process, mechanism, meaning, or boundary conditions, and note disconfirming cases when available.

## Minimal Output Format

```markdown
# Results Interpretation

## Main Findings
- ...

## Mechanisms and Rival Explanations
- ...

## Uncertainty and Robustness
- ...

## Boundary Conditions
- ...

## Implications Without Overclaiming
- ...
```
