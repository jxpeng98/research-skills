---
id: effect-size-interpreter
stage: F_writing
version: "1.0.0"
description: "Translate coefficients and effect sizes into substantive magnitude statements that readers can understand quickly."
inputs:
  - type: StatsReport
    description: "Model output with coefficients, effect sizes, and standard errors"
  - type: DesignSpec
    description: "Study context needed for substantive interpretation"
    required: false
  - type: VenueAnalysis
    description: "Venue expectations for reporting effect sizes"
    required: false
outputs:
  - type: EffectInterpretation
    artifact: "manuscript/effect_interpretation.md"
constraints:
  - "Must convert estimates into interpretable units, percentages, or standardized changes when possible"
  - "Must separate statistical significance from substantive importance"
  - "Must avoid effect-size translation that depends on undocumented assumptions"
failure_modes:
  - "Magnitude claims are detached from actual measurement units"
  - "Small but precise effects are described as practically important without context"
tools: [filesystem]
tags: [writing, results, effect-size, magnitude, interpretation]
domain_aware: true
---

# Effect Size Interpreter Skill

Explain what an estimate means in substantive, not just statistical, terms.

## Related Task IDs

- `F3` (full manuscript draft)

## Output (contract path)

- `RESEARCH/[topic]/manuscript/effect_interpretation.md`

## Procedure

1. **Identify the reporting scale**:
   - Raw units
   - Percentage change
   - Standard deviations
   - Odds / hazard / risk differences
2. **Translate the estimate** into a reader-facing statement.
3. **Contextualize importance**:
   - Baseline level
   - Comparison benchmark
   - Practical threshold if available

## Minimal Output Format

```markdown
# Effect Interpretation

## Magnitude Translation
- ...

## Practical Importance
- ...

## Caveats
- ...
```
