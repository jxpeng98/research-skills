# Method Diagnostic Contract

Method diagnostics make design threats explicit before analysis or writing locks them in.

## Canonical Paths

- `RESEARCH/[topic]/design/method-diagnostic-report.md`
- `RESEARCH/[topic]/design/validity-threat-matrix.md`

## Required Threat Categories

- construct validity
- internal validity
- external validity
- statistical conclusion validity
- measurement validity
- data leakage
- missingness
- confounding
- selection bias

## Rules

- Record insufficient inputs instead of guessing design details.
- Connect each threat to a mitigation, robustness check, or limitation.
- Stage I code planning should consume the diagnostic report when present.
