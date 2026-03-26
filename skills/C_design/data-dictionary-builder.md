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
  - "Must cover all data states (raw, cleaned, analysis-ready)"
failure_modes:
  - "Instrument not yet finalized"
  - "Variable naming conflicts across instruments"
tools: [filesystem]
tags: [design, data-dictionary, codebook, variables, metadata]
domain_aware: false
---

# Data Dictionary Builder Skill

Create a comprehensive codebook (data dictionary) that defines every variable in the dataset — serving as the authoritative reference for data cleaning, analysis, and replication.

## Related Task IDs

- `C3` (data management)

## Output (contract path)

- `RESEARCH/[topic]/data_dictionary.md`

## When to Use

- After study design is finalized and instruments are selected
- Before any data collection begins (baseline dictionary)
- After data cleaning (update with derived variables and transformations)
- When preparing data for sharing or archival

## Process

### Step 1: Inventory All Variables

Enumerate variables from every source:

| Source | Variables to Extract |
|--------|---------------------|
| **Survey instruments** | All items, response scales, scoring rules |
| **Administrative records** | ID fields, dates, demographics, system-generated fields |
| **Experimental design** | Treatment assignment, condition, block, randomization seed |
| **Derived/computed** | Composite scores, indices, transformations, recodes |
| **Metadata** | Data collection timestamp, interviewer ID, wave/round |

### Step 2: Define Each Variable

For every variable, specify all of the following:

| Field | Description | Example |
|-------|-------------|---------|
| **`variable_name`** | Programmatic name; snake_case, max 32 chars, no special characters | `gse_total_score` |
| **`label`** | Human-readable description | "General Self-Efficacy total score (10–40)" |
| **`type`** | Data type | `continuous` / `categorical` / `ordinal` / `binary` / `datetime` / `text` / `integer` |
| **`valid_range`** | Acceptable values — numeric range or level set | `10–40` (continuous) or `{1, 2, 3}` (categorical) |
| **`coding_scheme`** | How values map to meaning | `1=Strongly disagree, 2=Disagree, 3=Agree, 4=Strongly agree` |
| **`source`** | Instrument name + item number, or computation rule | `GSE-10, items 1–10, sum` |
| **`missing_codes`** | How missingness is represented | `-99=Not applicable, -98=Refused, -97=Skip logic, NA=System missing` |
| **`derivation`** | Formula if computed from other variables | `gse_total = sum(gse_item_1 : gse_item_10)` |
| **`notes`** | Special handling, known issues, caveats | "Reverse-coded before summing; items 3, 5 inverted" |

### Step 3: Apply Naming Conventions

| Convention | Rule | Example |
|-----------|------|---------|
| **Prefix by construct** | Group related variables | `gse_item_1`, `gse_item_2`, `gse_total` |
| **Suffix by wave** | Distinguish longitudinal data | `gse_total_t1`, `gse_total_t2` |
| **Suffix by source** | Distinguish informant/mode | `stress_self`, `stress_peer`, `stress_obs` |
| **Indicator suffix** | Binary flags | `dropout_flag`, `outlier_flag`, `consent_yn` |
| **Max chars** | Keep names manageable | ≤32 characters for compatibility |

> **Anti-pattern**: `q12_a3_rev_log_z` — opaque, non-self-documenting. Use `gse_item_3_reversed` instead.

### Step 4: Document Derived Variables

For each computed variable, provide:

```markdown
## Derived Variable: [variable_name]

- **Formula**: `total = sum(item_1, item_2, ..., item_n)` or `index = (x - mean) / sd`
- **Input variables**: [list exact variable names]
- **Missing handling**: "If >20% items missing, set total = NA; else impute with person mean"
- **Reverse coding**: "Items [list] reverse-coded before aggregation"
- **Validation**: "Expected range: [min]–[max]; flag values outside range"
```

### Step 5: Document Missing Data Codes

Standardize missing data coding across the entire dataset:

| Code | Meaning | Use When |
|------|---------|----------|
| `NA` | System missing / not collected | Value was never recorded |
| `-99` | Not applicable | Question does not apply to participant |
| `-98` | Refused | Participant declined to answer |
| `-97` | Missing due to skip logic | Question skipped by branching logic |
| `-96` | Technical failure | System/equipment failure during collection |
| `""` (empty string) | Text field left blank | Text fields only |

> **Key principle**: Different types of missingness should have different codes — this affects imputation strategy and analysis.

### Step 6: Cross-Reference with Instruments

Create a traceability table:

| Instrument | Item | Variable Name | Response Scale | Scoring |
|-----------|------|--------------|----------------|---------|
| GSE-10 | Q1 | `gse_item_1` | 1–4 Likert | Direct |
| GSE-10 | Q3 | `gse_item_3` | 1–4 Likert | Reverse (5 − response) |
| PHQ-9 | Q1 | `phq_item_1` | 0–3 | Direct |

### Step 7: Version Control the Dictionary

| Event | Action |
|-------|--------|
| Initial creation | Version 1.0 — before data collection |
| Post-pilot revision | Version 1.1 — document what changed and why |
| Post-cleaning | Version 2.0 — add derived variables, document cleaning transformations |
| Pre-release | Final version — freeze; include in release package |

## Quality Bar

The data dictionary is **ready** when:

- [ ] Every variable in the dataset has an entry (no undocumented fields)
- [ ] All fields specified: name, label, type, range, coding, source, missing codes
- [ ] Derived variables have exact formulas and input variable references
- [ ] Naming conventions are consistent across the dataset
- [ ] Missing data codes are standardized and documented
- [ ] Instrument-to-variable traceability is complete
- [ ] Dictionary version matches the analysis-ready dataset version

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Variables without labels | Analysts can't interpret variable meaning | Every variable gets a human-readable label |
| Single missing code for all types | Conflates "refused" with "not applicable" | Use differentiated missing codes |
| Derived variables without formulas | Can't reproduce computations | Document exact derivation + input variables |
| Naming collisions across instruments | `q1` appears in three instruments | Prefix by instrument/construct |
| Dictionary not updated after cleaning | Drift between dictionary and actual data | Version-control; update on each data state change |
| No range validation | Impossible values go undetected | Specify valid ranges; flag violations |

## Minimal Output Format

```markdown
# Data Dictionary v[version]

## Dataset: [name]
## Date: [date]
## Variables: [count]

## Variable Definitions

| Variable | Label | Type | Range | Coding | Source | Missing |
|----------|-------|------|-------|--------|--------|---------|
| participant_id | Unique participant identifier | integer | 1001–9999 | — | System | — |
| age | Participant age at enrollment | continuous | 18–99 | years | Demographics Q1 | -99 |
| gse_total | General Self-Efficacy total | continuous | 10–40 | sum of items | GSE-10 | -99 |

## Derived Variables

| Variable | Formula | Inputs | Missing Rule |
|----------|---------|--------|-------------|

## Missing Data Codes

| Code | Meaning |
|------|---------|

## Change Log

| Version | Date | Changes |
|---------|------|---------|
```
