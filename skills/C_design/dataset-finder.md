---
id: dataset-finder
stage: C_design
description: "Identify feasible primary or secondary datasets, access routes, and coverage limitations for an empirical design."
inputs:
  - type: RQSet
    description: "Research questions defining the target phenomenon and unit of analysis"
  - type: DesignSpec
    description: "Study design constraining sampling frame, timeframe, and measures"
  - type: AnalysisPlan
    description: "Planned estimands and identification strategy"
    required: false
outputs:
  - type: DatasetPlan
    artifact: "design/dataset_plan.md"
constraints:
  - "Must distinguish candidate datasets from the recommended primary dataset"
  - "Must document access constraints, licensing, and expected update cadence"
  - "Must state key coverage gaps that threaten identification or validity"
failure_modes:
  - "No feasible dataset can identify the target construct or treatment timing"
  - "Access restrictions make the preferred dataset non-viable within project constraints"
tools: [filesystem, scholarly-search, metadata-registry]
tags: [design, data, datasets, feasibility, provenance]
domain_aware: true
---

# Dataset Finder Skill

Identify and compare feasible datasets before implementation starts.

## Related Task IDs

- `C4` (data management and dataset planning)

## Output (contract path)

- `RESEARCH/[topic]/design/dataset_plan.md`

## Procedure

1. **Translate the design into data requirements**:
   - Unit of observation
   - Time coverage
   - Treatment / outcome / control requirements
   - Geographic or platform scope
2. **List candidate datasets**:
   - Primary collection
   - Public secondary data
   - Restricted administrative / proprietary data
3. **Evaluate each option**:
   - Coverage and granularity
   - Access process, cost, license, lead time
   - Key variables available vs. missing
   - Match to identification strategy
4. **Recommend one path**:
   - Preferred dataset or combined-source strategy
   - Backup option if access fails
   - Main risks and mitigation steps

## Minimal Output Format

```markdown
# Dataset Plan

## Data Requirements
- Unit of analysis:
- Time window:
- Required variables:

## Candidate Datasets
| Dataset | Coverage | Access | Strengths | Gaps |
|---|---|---|---|---|

## Recommended Dataset Strategy
- Primary source:
- Backup source:
- Main risks:
```
