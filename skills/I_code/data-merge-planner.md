---
id: data-merge-planner
stage: I_code
version: "0.2.2"
description: "Define dataset join strategy, key validation, and provenance controls for multi-source empirical workflows."
inputs:
  - type: DatasetPlan
    description: "Selected datasets and expected linkages"
  - type: VariableSpec
    description: "Variable-level definitions and source ownership"
  - type: CleaningPlan
    description: "Cleaning assumptions that affect merge readiness"
    required: false
outputs:
  - type: MergePlan
    artifact: "data/merge_plan.md"
constraints:
  - "Must identify join keys, cardinality expectations, and unmatched-record handling"
  - "Must document the order of merges and post-merge validation checks"
  - "Must preserve source provenance for each merged field"
failure_modes:
  - "Join keys are non-unique or unstable across sources"
  - "Merge order changes the analysis sample without being documented"
tools: [filesystem, code-runtime]
tags: [code, data, merge, joins, provenance]
domain_aware: true
---

# Data Merge Planner Skill

Plan how multiple datasets become one analysis-ready panel or cross-section.

## Related Task IDs

- `I3` (data pipeline)

## Output (contract path)

- `RESEARCH/[topic]/data/merge_plan.md`

## Procedure

1. **Inventory source datasets** and required joins.
2. **Define merge keys and cardinality**:
   - One-to-one / one-to-many / many-to-one
   - Time alignment rules
   - Entity resolution assumptions
3. **Specify validation checks**:
   - Duplicate keys
   - Unmatched record counts
   - Post-merge row-count expectations
4. **Record provenance** for merged and derived fields.

## Minimal Output Format

```markdown
# Merge Plan

| Step | Left Source | Right Source | Join Key | Cardinality | Validation |
|---|---|---|---|---|---|

## Unmatched Handling
- Rule:
- Audit output:
```
