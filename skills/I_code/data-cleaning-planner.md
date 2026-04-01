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

Specify reproducible, documented cleaning rules before execution — ensuring every data transformation is justified, auditable, and pre-specified.

## Purpose

Define executable data-cleaning rules covering validation, missingness, outliers, and harmonization.

## Related Task IDs

- `I3` (data pipeline)

## Output (contract path)

- `RESEARCH/[topic]/data/cleaning_plan.md`

## When to Use

- After dataset is obtained, before any analysis
- When working with secondary data that requires harmonization
- When combining multiple data sources (pair with `data-merge-planner`)
- When a reviewer asks "how did you handle missing data / outliers?"

## Process

### Step 1: Define Data States

Establish the pipeline stages — never overwrite raw data:

```
raw/          →  staged/       →  cleaned/      →  analysis/
(untouched)      (validated)      (transformed)    (model-ready)
```

| State | What Happens | Reversibility |
|-------|-------------|---------------|
| **Raw** | Original data as received; never modified | N/A — source of truth |
| **Staged** | Schema validated; type coercion; deduplication | Fully reversible (rerun from raw) |
| **Cleaned** | Missing data handled; outliers addressed; recoded | Documented; reversible with raw + rules |
| **Analysis-ready** | Final sample; derived variables; analysis format | Derived from cleaned + variable spec |

### Step 2: Schema Validation

For each variable in the data dictionary, verify:

| Check | Method | Action on Failure |
|-------|--------|-------------------|
| **Column exists** | Check against data dictionary | Flag missing columns → investigate |
| **Data type** | Compare expected vs actual type | Coerce with error logging |
| **Value range** | Check against valid_range from dictionary | Flag invalid → investigate or recode |
| **Unique identifiers** | Check ID uniqueness per unit | Flag duplicates → resolution rule |
| **Sentinel values** | Check for coded missing (-99, -98, etc.) | Convert to NA with type tag |
| **Date formats** | Verify consistent date parsing | Standardize to ISO 8601 |
| **Text encoding** | Check for encoding errors, special characters | Standardize to UTF-8 |

### Step 3: Duplicate Handling

| Duplicate Type | Detection Method | Resolution Rule |
|---------------|-----------------|-----------------|
| **Exact duplicate rows** | All columns identical | Keep first occurrence; log count |
| **Duplicate keys** | Same ID, different values | Business rule: keep latest / most complete / flag for manual review |
| **Near-duplicates** | Fuzzy matching on key fields | Set similarity threshold; manual review above threshold |

```markdown
## Duplicate Report

| Type | Count | Resolution | Records Removed |
|------|-------|-----------|-----------------|
| Exact duplicate | [n] | Kept first | [n] |
| Duplicate key | [n] | Kept latest | [n] |
```

### Step 4: Missing Data Assessment and Handling

#### Assessment

| Metric | How to Calculate | How to Report |
|--------|-----------------|---------------|
| **% missing per variable** | `missing_count / n * 100` | Table of all variables with missing % |
| **% missing per observation** | `missing_vars / total_vars * 100` | Distribution histogram |
| **Missing data pattern** | MCAR, MAR, or MNAR | Little's MCAR test; visual missing-data patterns |
| **Correlation with key variables** | Compare means for complete vs missing | t-tests for missingness ~ key variables |

#### Handling Strategies

| Strategy | When to Use | Assumptions | How to Implement |
|----------|------------ |-------------|-----------------|
| **Listwise deletion** | Missing < 5%; MCAR | MCAR | Drop incomplete cases; report N before/after |
| **Pairwise deletion** | Missing moderate; different variables per analysis | MCAR | Use available data per analysis |
| **Mean/median imputation** | Quick; for controls only | MCAR; underestimates variance | Replace with sample mean; NOT for outcomes |
| **Multiple imputation (MI)** | Missing 5–40%; MAR | MAR | MICE with m ≥ 20 imputations; pool estimates (Rubin's rules) |
| **Full information ML (FIML)** | Structural models (SEM) | MAR | Model-based; no explicit imputation |
| **Sensitivity analysis** | When MNAR is plausible | Various | Compare results across methods; Heckman selection model |

> **Pre-specification discipline**: Decide your missing data strategy BEFORE looking at associations between missingness and outcomes.

### Step 5: Outlier Detection and Handling

| Method | Formula/Rule | When to Use | Caveats |
|--------|-------------|------------|---------|
| **Z-score** | \|z\| > 3 | Normally distributed continuous data | Sensitive to the outliers themselves |
| **IQR method** | < Q1 − 1.5×IQR or > Q3 + 1.5×IQR | Skewed distributions | Conservative; may flag too many |
| **Mahalanobis distance** | Multivariate outliers | Multiple variables simultaneously | Assumes multivariate normality |
| **Cook's distance** | D > 4/n | Influential observations in regression | Post-estimation; model-specific |
| **Domain knowledge** | Impossible values based on context | Always the first check | Document domain constraints |

#### Handling Strategies

| Strategy | When | Pre-specified? |
|----------|------|---------------|
| **Remove** | Data entry error (impossible values) | ✅ Must pre-specify criteria |
| **Winsorize** | Extreme but plausible values | ✅ Specify percentile (1%/99%) |
| **Transform** | Skewed distribution | ✅ Log, square root, inverse |
| **Report both** | Sensitivity to outliers unclear | ✅ Primary with; robustness without |
| **Keep all** | Default — need strong reason to exclude | Default |

> **Anti-pattern**: Looking at how outliers affect your results, THEN deciding to remove them. This is p-hacking. Pre-specify your outlier rules.

### Step 6: Recoding and Harmonization

| Task | Specification Required |
|------|----------------------|
| **Reverse coding** | Which items; formula (e.g., `new = max + 1 − old`) |
| **Categorical recoding** | Mapping table (old → new labels/codes) |
| **Variable merging** | When combining synonymous variables across waves/sources |
| **Unit harmonization** | Converting different units (currency, dates, scales) |
| **Text standardization** | Lowercase, trim, standardize spellings |

### Step 7: Generate the Cleaning Log

Every cleaning action must be logged:

```markdown
## Cleaning Log

| Step | Variable(s) | Action | Records Affected | Before | After | Justification |
|------|------------|--------|-----------------|--------|-------|---------------|
| 1 | all | Remove exact duplicates | 23 | 5,023 | 5,000 | Duplicate submissions |
| 2 | age | Cap at 99 | 5 | observed max 147 | max 99 | Impossible values |
| 3 | income | Log transform | all | right-skewed | ~normal | Normality assumption |
| 4 | phq_3 | Reverse code | all | 0–3 | 3–0 | Scoring protocol |
```

## Quality Bar

The cleaning plan is **ready** when:

- [ ] Data states defined (raw → staged → cleaned → analysis-ready)
- [ ] Schema validation checks specified for all variables
- [ ] Duplicate handling rules defined
- [ ] Missing data assessed and handling strategy pre-specified with justification
- [ ] Outlier detection method and handling rules pre-specified
- [ ] Recoding and transformation rules documented
- [ ] Cleaning log template prepared
- [ ] No cleaning rule was chosen based on its effect on results

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Modifying raw data in place | Can't reproduce from scratch | Never touch raw/; derive all states |
| Choosing missing data method after seeing results | Researcher degrees of freedom | Pre-specify strategy before analysis |
| Removing outliers to get significant results | P-hacking | Pre-specify criteria; report both with/without |
| No cleaning log | Reviewer can't audit decisions | Log every action with count + justification |
| Harmonizing scales without documenting | Introduces hidden measurement error | Document every conversion formula |
| Dropping observations without reporting N flow | Reader can't assess selection bias | Report N at every stage (like CONSORT flow) |

## Minimal Output Format

```markdown
# Data Cleaning Plan

## Pipeline: raw → staged → cleaned → analysis

## Schema Validation
| Variable | Expected Type | Valid Range | Check |
|----------|-------------|-------------|-------|

## Duplicate Handling
| Type | Detection | Resolution |
|------|-----------|-----------|

## Missing Data
| Variable | % Missing | Pattern | Strategy |
|----------|-----------|---------|----------|

## Outlier Rules
| Variable | Detection | Threshold | Action |
|----------|-----------|-----------|--------|

## Transformations
| Variable | Transformation | Justification |
|----------|---------------|---------------|

## Cleaning Log
| Step | Action | N Before | N After | Justification |
|------|--------|----------|---------|---------------|
```
