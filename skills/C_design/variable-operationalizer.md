---
id: variable-operationalizer
stage: C_design
version: "0.2.1"
description: "Map abstract constructs to concrete, measurable variables with validity/reliability justification."
inputs:
  - type: TheoreticalFramework
    description: "Constructs requiring operationalization"
  - type: DesignSpec
    description: "Study design context"
outputs:
  - type: OperationalizationMap
    artifact: "design/operationalization.md"
constraints:
  - "Each construct must map to at least one measurable variable"
  - "Must justify measurement validity (content, construct, criterion)"
  - "Must cite established scales when available"
failure_modes:
  - "No validated instrument exists for construct"
  - "Construct too abstract for single operationalization"
tools: [filesystem, scholarly-search]
tags: [design, operationalization, constructs, measurement, validity]
domain_aware: true
---

# Variable Operationalizer Skill

Bridge from abstract theoretical constructs to concrete, measurable variables.

## Related Task IDs

- `C1` (study design — operationalization step)

## Output (contract path)

- `RESEARCH/[topic]/design/operationalization.md`

## Procedure

1. **List all constructs** from the theoretical framework
2. **For each construct**:
   - Identify validated scales/instruments from literature
   - Specify exact items, response format, scoring
   - Justify measurement validity type (content, construct, criterion)
   - Report reliability (Cronbach's α, test-retest) from prior studies
   - Note adaptation/translation if applicable
3. **For constructs without validated instruments**:
   - Propose operationalization with rationale
   - Flag for pilot testing
   - Design face validity check
4. **Map construct → variable → instrument → item** hierarchy

## Minimal Output Format

```markdown
# Operationalization Map

| Construct | Variable | Instrument | Items | Validity | α (prior) |
|---|---|---|---|---|---|
| Self-efficacy | self_efficacy_score | GSE-10 | 10 items, 4-pt Likert | construct | .87 |
```
