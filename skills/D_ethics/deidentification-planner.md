---
id: deidentification-planner
stage: D_ethics
version: "0.2.2"
description: "Design technical privacy measures via k-anonymity, differential privacy, or secure data pipelines for sensitive data."
inputs:
  - type: DesignSpec
    description: "Study design with data types"
  - type: EthicsPackage
    description: "Ethics review requirements"
outputs:
  - type: DeidentificationPlan
    artifact: "compliance/deidentification_plan.md"
constraints:
  - "Must specify re-identification risk threshold"
  - "Must document data transformation procedures"
failure_modes:
  - "Small dataset makes k-anonymity impractical"
  - "Utility loss from aggressive anonymization"
tools: [filesystem]
tags: [ethics, privacy, deidentification, k-anonymity, differential-privacy]
domain_aware: false
---

# Deidentification Planner Skill

Plan concrete technical de-identification steps for sensitive data (not legal advice).

## Related Task IDs

- `D3` (de-identification plan)

## Output (contract path)

- `RESEARCH/[topic]/compliance/deidentification_plan.md`

## Procedure

1. Classify fields (direct identifiers / quasi-identifiers / sensitive attributes).
2. Define transformations (suppression/generalization/aggregation/perturbation).
3. Document residual risk and limitations.
