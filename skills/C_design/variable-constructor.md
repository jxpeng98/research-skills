---
id: variable-constructor
stage: C_design
version: "0.2.1"
description: "Translate constructs and hypotheses into an operational variable specification with coding, units, and derivation rules."
inputs:
  - type: DesignSpec
    description: "Study design describing constructs, measures, and units"
  - type: AnalysisPlan
    description: "Planned estimands, model structure, and outcome definitions"
  - type: HypothesisSet
    description: "Hypotheses defining focal relationships"
    required: false
  - type: DatasetPlan
    description: "Preferred data source and coverage constraints"
    required: false
outputs:
  - type: VariableSpec
    artifact: "design/variable_spec.md"
constraints:
  - "Must separate outcome, treatment, moderator, mediator, and control variables"
  - "Must identify source column, unit, coding, and transformation for each variable"
  - "Must flag proxies and unverifiable constructs explicitly"
failure_modes:
  - "Constructs cannot be operationalized with available data"
  - "Variable definitions leak post-treatment information into baseline covariates"
tools: [filesystem, metadata-registry]
tags: [design, variables, operationalization, measures, coding]
domain_aware: true
---

# Variable Constructor Skill

Build an auditable variable specification before coding or estimation starts.

## Related Task IDs

- `C3` (analysis plan and operationalization)

## Output (contract path)

- `RESEARCH/[topic]/design/variable_spec.md`

## Procedure

1. **Enumerate target constructs** from the design and hypotheses.
2. **Operationalize each construct**:
   - Variable name
   - Data source
   - Unit and scale
   - Coding scheme
   - Expected direction or role in the model
3. **Mark derived variables** with exact formula or transformation rule.
4. **Check design validity**:
   - Avoid post-treatment controls
   - Flag weak proxies
   - Note missing or partially observed variables

## Minimal Output Format

```markdown
# Variable Specification

| Role | Variable | Source | Unit / Coding | Transformation | Notes |
|---|---|---|---|---|---|

## Risks
- Proxy quality:
- Missing constructs:
```
