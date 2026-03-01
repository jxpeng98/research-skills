# Stats Engine Skill

Run and report statistical analyses in a way that supports peer review and reproducibility.

## When to Use

- You need to execute modeling/testing for synthesis (`E3/E3_5`) or empirical results (`F` stage) with a clear report of assumptions and uncertainty.

## Output (recommended path)

- `RESEARCH/[topic]/analysis/stats_report.md`

## Inputs (ask for)

- Estimand(s) / hypothesis / outcome definitions
- Data schema (columns, units, missingness expectations)
- Model family preferences (GLM, mixed models, time series, causal estimators, etc.)

## Procedure

1. Select model(s) aligned to estimand and data type.
2. Run diagnostics (assumptions, residual patterns, convergence).
3. Report effect sizes + uncertainty (CI/SE), not only p-values.
4. Record robustness checks and sensitivity analyses.

## Minimal report format (`analysis/stats_report.md`)

```markdown
# Statistical Report

## Data
- N:
- Missingness:

## Models
- Primary:
- Alternatives/robustness:

## Results
| Outcome | Effect | Uncertainty | Interpretation |
|---|---:|---|---|

## Diagnostics

## Caveats
```
