---
id: data-dictionary-builder
stage: C_design
description: "Create structured data dictionaries defining every variable's name, type, range, coding, and source."
inputs:
  - type: DesignSpec
    description: "Study design with measures and instruments"
  - type: Instruments
    description: "Survey/measurement instruments"
    required: false
outputs:
  - type: DataDictionary
    artifact: "data_dictionary.md"
constraints:
  - "Every variable must have: name, label, type, valid range, coding scheme"
  - "Must flag derived/computed variables with formula"
failure_modes:
  - "Instrument not yet finalized"
  - "Variable naming conflicts across instruments"
tools: [filesystem]
tags: [design, data-dictionary, codebook, variables, metadata]
domain_aware: false
---

# Data Dictionary Builder Skill

Create a structured codebook defining every variable in the dataset.

## Related Task IDs

- `C3` (data management)

## Output (contract path)

- `RESEARCH/[topic]/data_dictionary.md`

## Procedure

1. **Enumerate all variables** from instruments, design spec, and administrative records
2. **For each variable, specify**:
   - `variable_name` — snake_case, max 32 chars
   - `label` — human-readable description
   - `type` — categorical / ordinal / continuous / datetime / text / binary
   - `valid_range` — min-max, or list of levels
   - `coding_scheme` — e.g., "1=Male, 2=Female, 99=Missing"
   - `source` — instrument name, item number, or computation formula
   - `missing_codes` — how missingness is coded
3. **Flag derived variables** with exact computation formula
4. **Cross-reference** with instruments to ensure coverage

## Minimal Output Format

```markdown
# Data Dictionary

| Variable | Label | Type | Range | Coding | Source | Missing |
|---|---|---|---|---|---|---|
| age | Participant age | continuous | 18-99 | years | demographics Q1 | -99 |
```
