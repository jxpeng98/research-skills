---
id: data-cleaning-planner
stage: I_code
description: "Define executable data-cleaning rules covering validation, missingness, outliers, and harmonization."
inputs:
  - type: DatasetPlan
    description: "Selected datasets and access constraints"
  - type: VariableSpec
    description: "Operational variable definitions and expected coding"
  - type: AnalysisPlan
    description: "Planned estimators and assumptions sensitive to data quality"
    required: false
outputs:
  - type: CleaningPlan
    artifact: "data/cleaning_plan.md"
constraints:
  - "Must distinguish raw, intermediate, and analysis-ready data states"
  - "Must define missingness, duplicate, and invalid-value handling explicitly"
  - "Must preserve provenance for any irreversible cleaning decision"
failure_modes:
  - "Cleaning rules silently change estimands or sample definition"
  - "Outlier handling is justified only after looking at outcome effects"
tools: [filesystem, code-runtime]
tags: [code, data, cleaning, missingness, validation]
domain_aware: true
---

# Data Cleaning Planner Skill

Specify reproducible cleaning rules before execution.

## Related Task IDs

- `I3` (data pipeline)

## Output (contract path)

- `RESEARCH/[topic]/data/cleaning_plan.md`

## Procedure

1. **Define data states**: raw, staged, cleaned, analysis-ready.
2. **Write cleaning rules** for:
   - Type coercion and schema validation
   - Missing values and sentinel codes
   - Duplicate handling
   - Range checks and impossible values
   - Harmonization across waves or sources
3. **Document provenance**:
   - Which fields are overwritten vs. appended
   - Which logs must be retained

## Minimal Output Format

```markdown
# Cleaning Plan

## Data States
- Raw:
- Cleaned:

## Rules
| Field | Check | Action | Rationale |
|---|---|---|---|
```
