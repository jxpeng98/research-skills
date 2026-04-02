---
id: effect-size-calculator
stage: E_synthesis
description: "Calculate standard effect sizes (e.g., Cohen's d, Hedges' g, odds ratios) from raw statistics to enable meta-analysis synthesis."
inputs:
  - type: ExtractionTable
    description: "Raw statistics and sample sizes from included papers"
  - type: VariableSpec
    description: "Definitions of dependent and independent variables"
outputs:
  - type: EffectSizeTable
    artifact: "effect_size_table.md"
  - type: AnalysisCode
    artifact: "analysis/effect_size_calc.py"
constraints:
  - "Must explicitly state the formula used for each conversion"
  - "Must clearly specify the direction of the effect ensuring consistent algebraic sign"
  - "Must calculate 95% Confidence Intervals for all computed effect sizes"
failure_modes:
  - "Failing to account for cluster-randomization or repeated measures in variance extraction"
  - "Assuming independent samples when groups are paired"
tools: [filesystem, code-runtime]
tags: [meta-analysis, effect-size, statistics, synthesis, hedges-g, cohens-d]
domain_aware: true
---

# Effect Size Calculator Skill

Systematically extract raw descriptive and inferential statistics from primary studies and convert them into standardized effect sizes (and their standard errors) to prepare for quantitative synthesis or meta-analysis.

## Purpose

Calculate standard effect sizes (e.g., Cohen's d, Hedges' g, odds ratios) from raw statistics to enable meta-analysis synthesis.

## Related Task IDs

- `E2` (effect-size-table)

## Output (contract path)

- `RESEARCH/[topic]/effect_size_table.md`
- `RESEARCH/[topic]/analysis/effect_size_calc.py` (optional executable artifact)

## When to Use

- When preparing data for a quantitative meta-analysis
- When primary studies report heterogeneous statistical test results (e.g., F-tests, t-tests, Chi-square) that need to be harmonized into a single metric (e.g., correlation $r$ or Hedges' $g$)
- When estimating variances bounds for missing standard deviations

## Process

### Step 1: Input Harmonization
- Read the `ExtractionTable` to gather the reported statistics ($N$, Means, SDs, proportions, $t$-values, $F$-values, exact $p$-values).
- Read the `VariableSpec` to ensure standard mapping of outcomes and treatments.

### Step 2: Family Selection
Select the appropriate effect size family based on the research question:
- **d-family (Standardized Mean Differences):** Cohen's $d$, Hedges' $g$. Used for continuous outcomes comparing two groups.
- **r-family (Correlations):** Pearson's $r$, partial correlations. Used for continuous predictors and continuous outcomes.
- **OR/RR-family:** Odds Ratios, Risk Ratios. Used for dichotomous outcomes.

### Step 3: Calculation & Conversion
For each finding:
1. Apply the primary calculation formula if complete descriptives (Mean, SD, N) are available.
2. If descriptives are missing, apply exact-test conversions (e.g., $t$ to $d$, $F(1, df)$ to $d$).
3. If only $p$-values are given, back-calculate using conservative assumptions (e.g., assuming the $p$-value represents the upper bound).
4. Apply sample size corrections (e.g., converting Cohen's $d$ to Hedges' $g$ for small samples).
5. Always compute the standard error ($SE$) of the effect size.

### Step 4: Directional Alignment
- Ensure that the algebraic sign (+/-) of the effect size is consistent across all studies (e.g., positive = "treatment is better"). Document the alignment logic.

### Step 5: Table and Code Generation
Produce the markdown `effect_size_table.md`.
Optionally, emit a Python or R script reproducing the exact calculation logic using `scipy.stats` or `metafor`.

## Quality Bar

- [ ] Every extracted effect size is accompanied by its Standard Error (SE) and 95% CI.
- [ ] Explicit distinction made between independent vs. dependent samples.
- [ ] Directional signs are strictly normalized and verified against original source conclusions.
- [ ] All conversions and assumptions are clearly documented in the table footer.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Mismatched Signs | Study 1 defines (+) as success, Study 2 defines (-) as success | Manually reverse signs for Study 2 and log the inversion |
| Ignored Clustering | Standard errors are falsely precise for clustered data | Apply an assumed Intra-Class Correlation (ICC) inflation factor |
| Exact vs Threshold $p$ | Converting "p < 0.05" as exactly $p=0.05$ | Explicitly flag these as "conservative imputed bounds" rather than exact values |
