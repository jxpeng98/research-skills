---
id: variable-constructor
stage: C_design
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

Build an auditable variable specification that bridges from research design to analysis code — ensuring every variable in the model is explicitly defined, sourced, and validated before any estimation begins.

## Related Task IDs

- `C3` (analysis plan and operationalization)

## Output (contract path)

- `RESEARCH/[topic]/design/variable_spec.md`

## When to Use

- After study design is finalized
- Before writing analysis code (variable spec drives code construction)
- When working with secondary data (variable mapping is critical)
- When a reviewer asks "how exactly did you define X?"

## Process

### Step 1: Enumerate Target Constructs from Design and Hypotheses

For each hypothesis, extract all required variables:

```
H1: Remote work ratio → Productivity index (moderated by team size)

Required variables:
- Outcome: productivity_index (DV)
- Treatment: remote_ratio (IV)
- Moderator: team_size
- Controls: tenure, education, industry (from theory)
- Interaction: remote_ratio × team_size
```

### Step 2: Classify Variable Roles

| Role | Definition | Analytical Implications | Common Mistakes |
|------|-----------|------------------------|-----------------|
| **Outcome (DV)** | What you are trying to explain | Must match the estimand exactly | Using a proxy without acknowledging it |
| **Treatment / Exposure (IV)** | What you hypothesize causes change | Must be pre-treatment or exogenous | Endogenous "treatment" without identification strategy |
| **Mediator** | Mechanism through which IV affects DV | Must be temporally between IV and DV | Including mediators as controls (blocks the causal path) |
| **Moderator** | Boundary condition that changes IV→DV relationship | Include as interaction term | Confusing moderators with mediators |
| **Control** | Confounders that must be held constant | Each must be theoretically justified | "Kitchen sink" controls; post-treatment controls |
| **Instrumental variable** | Exogenous source of variation in IV | Must satisfy exclusion restriction + relevance | Weak instruments (F < 10) |
| **Fixed effect** | Absorbs unobserved group-level confounders | Removes between-group variation | When between-group variation IS the question |

> **Critical check**: Is any control variable post-treatment? If so, including it will bias the causal estimate. Draw the causal diagram (DAG) first.

### Step 3: Specify Each Variable Fully

For every variable in the analysis:

| Field | What to Document | Example |
|-------|-----------------|---------|
| **Variable name** | Programmatic name (snake_case) | `productivity_index` |
| **Role** | DV / IV / mediator / moderator / control / FE | DV |
| **Source** | Dataset, table, column / survey item / computation | HR system, quarterly_reports.output_hours |
| **Unit and scale** | Measurement unit and numeric scale | Output units per hour; continuous; range 0–∞ |
| **Coding scheme** | How raw data maps to analysis variable | Direct (no transformation) |
| **Transformation** | Log, z-score, winsorize, categorize, etc. | `log(productivity_index + 1)` |
| **Temporal alignment** | When is this measured relative to treatment? | Pre-treatment (baseline) / concurrent / post-treatment |
| **Expected direction** | Hypothesized sign in model | Positive (β > 0) |
| **Missing data plan** | How missing values are handled | Multiple imputation if < 15%; listwise if < 5% |

### Step 4: Document Derived and Transformed Variables

For each computed variable:

| Derivation Type | Specification Required | Example |
|----------------|----------------------|---------|
| **Scale composite** | Aggregation rule + missing handling | `gse_total = mean(gse_1:gse_10) if valid_items ≥ 8` |
| **Log transformation** | Base + offset for zeros | `log_revenue = log(revenue + 1)` |
| **Standardization** | Reference group for mean/SD | `z_score = (x − mean_all) / sd_all` |
| **Binning/categorization** | Cut points + labels | `age_cat: 18–29=1, 30–44=2, 45–59=3, 60+=4` |
| **Interaction term** | Component variables × method | `interact_remote_team = remote_ratio × team_size` |
| **Lagged variable** | Lag structure + panel structure | `productivity_lag1 = productivity[t-1]` (quarterly lag) |
| **Ratio/index** | Numerator, denominator, edge cases | `remote_ratio = wfh_days / total_work_days` (exclude leave days) |
| **Difference** | Pre-post or between-source | `productivity_change = productivity_t2 − productivity_t1` |

### Step 5: Check Design Validity

| Check | What to Verify | Why It Matters |
|-------|---------------|---------------|
| **No post-treatment controls** | Controls measured before or at treatment | Post-treatment controls bias causal estimates |
| **No collider conditioning** | Not conditioning on a variable caused by both IV and DV | Opens non-causal paths |
| **Proxy quality** | If using proxies, how well do they capture the construct? | Attenuation bias from measurement error |
| **Temporal ordering** | IV measured before DV; mediator between IV and DV | Establishes causal direction |
| **Level-of-analysis match** | Variable level matches the unit of analysis | Team-level construct aggregated from individual data needs justification |
| **Multicollinearity risk** | Highly correlated predictors identified | Inflate standard errors; unstable estimates |
| **Missing data patterns** | Missing not at random (MNAR) detected | May need Heckman correction or sensitivity analysis |

### Step 6: Create the Variable Specification Table

```markdown
# Variable Specification

## Model: DV ~ IV + moderator × IV + controls + FE

### Outcome Variables

| Variable | Source | Unit | Transform | Temporal | Missing |
|----------|--------|------|-----------|----------|---------|
| productivity_index | HR quarterly | units/hr | log(x+1) | Post-treatment (Q3) | MI if <15% |

### Treatment Variables

| Variable | Source | Unit | Transform | Temporal | Missing |
|----------|--------|------|-----------|----------|---------|
| remote_ratio | HR monthly | proportion | none | Treatment period (Q1–Q2) | Exclude if missing |

### Moderators

| Variable | Source | Unit | Transform | Temporal | Interaction |
|----------|--------|------|-----------|----------|-------------|
| team_size | HR roster | count | none | Baseline (Q0) | remote_ratio × team_size |

### Control Variables

| Variable | Source | Unit | Transform | Temporal | Justification |
|----------|--------|------|-----------|----------|---------------|
| tenure | HR records | years | none | Baseline | Experience confounds productivity |
| education | Survey | ordinal (1–5) | factor | Baseline | Human capital theory |

### Fixed Effects

| Variable | Level | Purpose |
|----------|-------|---------|
| department_fe | Department | Absorb unobserved dept culture |
```

## Quality Bar

The variable specification is **ready** when:

- [ ] Every variable in the analysis model has a complete specification
- [ ] Roles clearly classified (DV, IV, mediator, moderator, control, FE)
- [ ] Temporal alignment documented (pre/post-treatment)
- [ ] No post-treatment controls included without justification
- [ ] Every control has a theoretical justification (not "kitchen sink")
- [ ] Derived variables have exact formulas and edge-case handling
- [ ] Proxy measures flagged with quality assessment
- [ ] Missing data handling specified per variable
- [ ] Multicollinearity risks identified

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Post-treatment control | Biases causal estimate | Draw DAG; verify temporal ordering |
| Including mediators as controls | Blocks the causal path you're studying | Separate mediation analysis |
| "Kitchen sink" controls | Introduces bad controls, reduces power | Each control needs theoretical justification |
| Inconsistent transformation across models | Coefficients not comparable | Document and standardize transformations |
| Using raw dollars across time | Inflation confound | Adjust for CPI; document base year |
| Proxy without acknowledgment | Overstates measurement precision | Flag proxies; discuss attenuation |
