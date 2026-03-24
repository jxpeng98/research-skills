---
id: robustness-planner
stage: C_design
version: "0.2.0"
description: "Pre-specify robustness checks, sensitivity analysis, and bounds scaling for empirical studies."
inputs:
  - type: DesignSpec
    description: "Study design with analysis plan"
  - type: AnalysisPlan
    description: "Primary analysis specification"
outputs:
  - type: RobustnessPlan
    artifact: "design/robustness_plan.md"
constraints:
  - "Must pre-specify all checks before analysis to avoid p-hacking"
  - "Must include at least 3 robustness checks"
failure_modes:
  - "Checklist not domain-appropriate"
  - "Over-specification leading to excessive multiple testing"
tools: [filesystem]
tags: [design, robustness, sensitivity-analysis, heteroskedasticity, endogeneity]
domain_aware: true
---

# Robustness Planner Skill

Pre-specify robustness and sensitivity checks so results are credible and reviewer-proof.

## Related Task IDs

- `C3_5` (robustness plan)

## Output (contract path)

- `RESEARCH/[topic]/design/robustness_plan.md`

## Procedure

1. Tie each robustness check to a specific threat:
   - functional form / specification
   - outliers / influential points
   - alternative operationalizations
   - alternative samples / exclusions
   - placebo / falsification tests (if causal)
2. Define interpretation rules: what would change the conclusion?
