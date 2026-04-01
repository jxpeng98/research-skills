---
id: prereg-writer
stage: C_design
description: "Generate preregistration documents for OSF/AsPredicted/ClinicalTrials.gov from study design and analysis plan."
inputs:
  - type: DesignSpec
    description: "Complete study design"
  - type: AnalysisPlan
    description: "Pre-specified analysis plan"
  - type: HypothesisSet
    description: "Testable hypotheses"
outputs:
  - type: Preregistration
    artifact: "preregistration.md"
constraints:
  - "Must include all hypotheses with directional predictions"
  - "Must specify primary and secondary outcomes before data collection"
  - "Must lock analysis plan with exact model specifications"
failure_modes:
  - "Exploratory research doesn't fit prereg template"
  - "Analysis plan too flexible to pre-specify fully"
tools: [filesystem]
tags: [design, preregistration, OSF, AsPredicted, transparency]
domain_aware: true
---

# Pre-registration Writer Skill

Generate structured preregistration documents that lock in your hypotheses, design, and analysis plan — maximizing transparency and credibility.

## Purpose

Generate preregistration documents for OSF/AsPredicted/ClinicalTrials.gov from study design and analysis plan.

## Related Task IDs

- `C4` (preregistration)

## Output (contract path)

- `RESEARCH/[topic]/preregistration.md`

## When to Use

- After study design and analysis plan are finalized
- Before ANY data is collected or accessed (for confirmatory research)
- Before accessing a dataset you haven't yet examined (for secondary data)
- When the target venue values or requires preregistration

## Process

### Step 1: Select the Right Template

| Platform | Best For | Format | Sections | Link |
|----------|---------|--------|----------|------|
| **OSF Prereg** | General social/behavioral science | Free-text, 28 fields | Comprehensive (hypotheses, design, sampling, analysis) | osf.io/prereg |
| **AsPredicted** | Quick preregistration | 9 fixed questions | Streamlined | aspredicted.org |
| **PROSPERO** | Systematic reviews | Structured protocol | Review-specific (search, inclusion, risk of bias) | crd.york.ac.uk/prospero |
| **ClinicalTrials.gov** | Clinical/health interventions | Regulatory format | Protocol-grade detail | clinicaltrials.gov |
| **AEA RCT Registry** | Economics experiments/RCTs | Structured with PAP | Design + pre-analysis plan | socialscienceregistry.org |

> **Rule**: Register BEFORE collecting or accessing data. If you've already seen the data, state this explicitly in the registration ("Secondary analysis of existing data; we have not yet examined the relevant variables").

### Step 2: Write Each Core Section

#### Section 1: Study Information

```markdown
## Study Information

**Title**: [Exact study title]
**Authors**: [All investigators]
**Date of registration**: [YYYY-MM-DD]
**Data collection status**: [Not yet begun / In progress / Complete]

**Research questions**:
1. [RQ1: clear, specific, answerable]
2. [RQ2: ...]

**Prior related work**: [Brief statement of what is known and why this study is needed]
```

#### Section 2: Hypotheses

For each hypothesis:

| Component | What to Specify | Example |
|-----------|----------------|---------|
| **ID** | Unique identifier | H1a |
| **Direction** | Predicted direction (positive, negative, null) | Positive |
| **Statement** | Clear, testable, specific | "Remote work ratio will be positively associated with productivity index" |
| **Mechanism** | Theory-based rationale | "Autonomy theory predicts increased focus time" |
| **Decision criterion** | What would confirm/disconfirm? | "Reject H₀ if β > 0, p < .05, corrected" |

```markdown
## Hypotheses

**H1**: [IV] will be [direction] associated with [DV].
- Mechanism: [theoretical justification]
- Decision criterion: β [direction] 0, p < .05 (two-tailed)
- Primary/secondary: Primary

**H2**: The effect of [IV] on [DV] will be moderated by [moderator],
such that [direction of moderation].
- Mechanism: [justification]
- Decision criterion: Interaction coefficient β₃ [direction] 0, p < .05
- Primary/secondary: Secondary
```

> **Discipline**: Separate confirmatory (pre-registered) from exploratory hypotheses. Exploratory analyses are valuable but must be clearly labeled as such.

#### Section 3: Design Plan

```markdown
## Design Plan

**Study type**: [Experiment / Observational / Survey / Qualitative / Mixed]
**Blinding**: [Double-blind / Single-blind / Not applicable]
**Randomization**: [Method / Not applicable]

**Design details**:
- Unit of analysis: [individual / team / firm]
- Treatment groups: [control vs treatment; or exposure levels]
- Timing: [pre-post / cross-sectional / longitudinal]
- Design-specific details: [e.g., factorial structure, nesting]
```

#### Section 4: Sampling Plan

```markdown
## Sampling Plan

**Existing data**: [Registration prior to / after data collection]
**Data collection procedures**: [How data will be collected]

**Sample size**: N = [target]
**Sample size rationale**:
- Test: [specific test: t-test / regression / etc.]
- Effect size: [d = / f² = / OR = ] based on [prior study / pilot]
- α = [.05], Power = [.80/.90]
- Tool: [G*Power / pwr / simulation]
- Adjustments: [clustering ICC, attrition buffer, multiple testing]

**Stopping rule**: [Fixed N / sequential analysis / saturation criterion]
```

#### Section 5: Variables

```markdown
## Variables

### Measured Variables
| Role | Variable | Operationalization | Scale | Source |
|------|----------|-------------------|-------|--------|
| DV | productivity_index | Output / hours | Continuous | HR system |
| IV | remote_ratio | WFH days / total days | 0–1 | HR system |
| Moderator | team_size | Current roster count | Integer | HR roster |

### Indices / Composites
| Index | Components | Aggregation | Reliability |
|-------|-----------|-------------|-------------|
| gse_total | gse_1 to gse_10 | Mean | α = .87 (prior) |

### Manipulated Variables (if experiment)
| Variable | Levels | Assignment | Manipulation Check |
|----------|--------|-----------|-------------------|
```

#### Section 6: Analysis Plan

This is the most critical section. Pre-specify with enough detail that someone else could replicate:

```markdown
## Analysis Plan

### Primary Analysis
- **Model**: OLS regression: `productivity_index ~ remote_ratio + team_size +
  remote_ratio × team_size + controls + department_FE`
- **Estimator**: OLS with robust standard errors (HC1)
- **Covariates**: tenure, education, industry (justified by [theory])
- **Inference criterion**: Two-tailed, α = .05

### Transformations
- `log(productivity_index + 1)` — to address right-skew
- All continuous predictors z-scored before interaction

### Exclusion Criteria (pre-data)
- Participants with < 3 months tenure (probation period)
- Departments with < 5 employees (insufficient variation)
- Observations with > 50% missing predictors

### Missing Data
- If missing < 5%: listwise deletion
- If 5–20%: multiple imputation (m = 20, MICE)
- If > 20%: sensitivity analysis comparing MI vs complete case

### Multiple Testing Correction
- Bonferroni correction for [n] primary hypotheses (α_adjusted = .05/[n])
- Secondary hypotheses: report uncorrected p-values with note

### Robustness Checks
1. Alternative estimator: [fixed effects / GEE / matched sample]
2. Outlier sensitivity: Winsorize at 1%/99%
3. Alternative outcome: [backup measure]
4. Sample restriction: Exclude [specific subgroup]

### Subgroup Analyses (pre-specified)
| Subgroup | Variable | Levels | Hypothesis |
|----------|----------|--------|-----------|
| By industry | industry_cat | Tech / Non-tech | Effect stronger in knowledge work |
```

#### Section 7: Exploratory Analyses

```markdown
## Exploratory Analyses

The following analyses are NOT confirmatory. Results will be labeled as exploratory.

1. [Mediation analysis exploring autonomy as mechanism]
2. [Non-linear relationship between remote_ratio and productivity]
3. [Additional moderators not predicted by theory]
```

### Step 3: Lock the Specification

Before registering:

| Check | What to Verify |
|-------|---------------|
| **Completeness** | All hypotheses have corresponding analysis plan entries |
| **Specificity** | Could someone else run your analysis from this document alone? |
| **Separation** | Confirmatory vs. exploratory clearly demarcated |
| **Feasibility** | Analysis plan matches available data and tools |
| **Consistency** | Variables in hypotheses match variables in analysis plan |
| **Time-stamp** | Registration date is before data collection/access date |

### Step 4: Plan for Deviations

> Deviations from the preregistration are normal and acceptable — but they must be documented.

```markdown
## Deviation Tracking

| Date | Section | Original Plan | Change Made | Justification |
|------|---------|--------------|-------------|---------------|
| [date] | Analysis | OLS | Changed to logit | DV is binary (discovered in data) |
```

## Quality Bar

The preregistration is **ready** when:

- [ ] All hypotheses stated with direction and decision criteria
- [ ] Design plan fully specifies study type and procedures
- [ ] Sample size justified with power analysis or saturation plan
- [ ] All variables operationalized with measurement details
- [ ] Analysis plan specifies exact model(s), estimator, covariates, and inference rules
- [ ] Exclusion criteria defined before data access
- [ ] Missing data and multiple testing strategies pre-specified
- [ ] Robustness checks enumerated
- [ ] Exploratory analyses clearly separated from confirmatory
- [ ] Registration timestamped before data collection/access

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Registering after seeing data | Undermines the entire purpose | Register before collection; if using existing data, state what you have/haven't seen |
| Vague hypotheses | "X is related to Y" — not testable | Specify direction + criterion + primary/secondary |
| Flexible analysis plan | "We will use appropriate methods" — anything goes | Specify exact models, exclusion criteria, missing data handling |
| No exploratory section | Exploratory findings presented as confirmatory | Add exploratory section; label findings accordingly |
| No deviation plan | Researchers feel trapped by suboptimal choices | Deviations are fine — just document them transparently |
| Forgetting multiple testing | Inflated false positive rate | Pre-specify correction method for primary hypotheses |
