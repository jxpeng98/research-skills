---
id: publication-bias-checker
stage: E_synthesis
description: "Evaluate publication bias using funnel plots, fail-safe N, Egger's test, and trim-and-fill correction models."
inputs:
  - type: EvidenceTable
    description: "Synthesized evidence with effect sizes"
outputs:
  - type: PublicationBiasReport
    artifact: "synthesis/publication_bias.md"
constraints:
  - "Must use at least 2 complementary methods"
  - "Must report funnel plot asymmetry test"
  - "Must calibrate claims — bias checks are suggestive, not definitive"
failure_modes:
  - "Too few studies (<10) for reliable funnel plot"
  - "Effect size heterogeneity confounds bias assessment"
  - "Asymmetry due to small-study effects, not publication bias"
tools: [filesystem, stats-engine]
tags: [synthesis, publication-bias, funnel-plot, fail-safe-N, trim-and-fill]
domain_aware: false
---

# Publication Bias Checker Skill

Assess whether the meta-analytic evidence base may be distorted by selective publication, reporting, or availability of results.

## Purpose

Evaluate publication bias using funnel plots, fail-safe N, Egger's test, and trim-and-fill correction models.

## Related Task IDs

- `E3_5` (publication bias assessment)

## Output (contract path)

- `RESEARCH/[topic]/synthesis/publication_bias.md`

## When to Use

- After completing meta-analysis (E3) with ≥5 studies
- When reviewers or guidelines (PRISMA item 21) require bias assessment
- When the literature has plausible asymmetry (e.g., industry-funded studies, file-drawer risk)

## Process

### Step 1: Assess Feasibility

| Condition | Minimum Studies | Reliability |
|-----------|----------------|-------------|
| Funnel plot (visual) | k ≥ 5 | Low for small k; useful as exploratory tool |
| Egger's regression test | k ≥ 10 | Adequate power requires k ≥ 10 |
| Begg's rank test | k ≥ 10 | Less powerful than Egger's |
| Trim-and-fill | k ≥ 10 | Provides adjusted estimate |
| Fail-safe N (Rosenthal) | k ≥ 3 | Widely reported but methodologically weak |
| p-curve | k ≥ 20 significant results | Tests evidential value |
| Selection models (Vevea-Hedges) | k ≥ 20 | Most powerful; requires assumptions |

> **If k < 10**: Document that formal tests are underpowered. Report funnel plot + narrative risk assessment only. Do NOT over-interpret test p-values.

### Step 2: Generate and Interpret Funnel Plot

**Construction**:
- X-axis: effect size (standardized mean difference, log-OR, etc.)
- Y-axis: precision (typically standard error, inverted so precise studies are at top)
- Add the pooled estimate as vertical line
- Add pseudo-95% confidence limits (funnel boundary)

**Interpretation guide**:

| Pattern | Likely Cause | Implication |
|---------|-------------|-------------|
| Symmetric distribution | No obvious bias | Reassuring but not definitive |
| Gap in bottom-right | Missing small, non-significant studies | Classic publication bias pattern |
| Gap in bottom-left | Missing small negative studies | Possible selective reporting of positive results |
| Extreme outliers | Influential studies | Investigate quality; consider sensitivity analysis |
| Hollow center | All studies clustered at edges | Possible heterogeneity, not bias |

> **Critical caveat**: Funnel plot asymmetry can be caused by genuine heterogeneity, methodological differences correlated with study size, or chance. Asymmetry ≠ publication bias.

### Step 3: Run Statistical Tests

#### Egger's Regression Test
- Regress standardized effect against precision (1/SE)
- If intercept ≠ 0 (p < 0.10 conventional threshold): evidence of small-study effects
- Report: intercept, SE, p-value, and interpretation

#### Begg's Rank Correlation Test
- Rank correlation between standardized effect and variance
- Report: Kendall's τ, p-value
- Less powerful than Egger's; include for completeness if convention in the field

#### Fail-safe N (Rosenthal's Method)
- Number of null studies needed to reduce observed effect to non-significance
- **Interpretation**: If fail-safe N > 5k + 10 (where k = number of studies), the result is robust
- **Caveat**: Methodologically criticized (assumes null studies have exactly zero effect)
- Report the number but do NOT rely on it as sole evidence

### Step 4: Apply Correction Methods

#### Trim-and-Fill (Duval & Tweedie)
1. Estimate the number of "missing" studies from funnel asymmetry
2. Impute their effect sizes (mirrored around the mean)
3. Re-estimate the pooled effect including imputed studies
4. Report:
   - Number of imputed studies (left/right)
   - Original pooled estimate vs adjusted estimate
   - Change in significance

#### Selection Models (advanced; if k ≥ 20)
- Vevea-Hedges weight function model
- Specify a priori weight functions for different p-value regions
- Compare model fit with and without selection
- Report: adjusted estimate, likelihood ratio test

#### p-Curve Analysis (if sufficient significant results)
- Distribution of p-values among significant results
- Right-skewed: evidential value present
- Flat or left-skewed: evidence of p-hacking or no true effect
- Report: Z-full test, Z-half test, binomial test

### Step 5: Synthesize and Calibrate Claims

Write a synthesis paragraph following this structure:

```
1. State which methods were feasible and which were not (with sample size justification)
2. Report funnel plot pattern
3. Report statistical test results (with exact values)
4. Report trim-and-fill adjusted estimate (if applicable)
5. State whether the correction changes the conclusion
6. Calibrate: "These tests are suggestive, not definitive. Asymmetry may reflect
   [small-study effects / heterogeneity / genuine bias]."
```

> **Never state**: "there is no publication bias." Instead: "no statistically significant evidence of funnel plot asymmetry was detected, though formal tests are underpowered for k = [n]."

## Quality Bar

The publication bias assessment is **ready** when:

- [ ] Feasibility documented (k reported; underpowered tests flagged)
- [ ] Funnel plot generated and interpreted with specific pattern description
- [ ] At least 2 complementary methods applied (or justified why not feasible)
- [ ] Original vs corrected pooled estimate compared
- [ ] Claims calibrated (does not state "bias absent" or "bias present" without qualification)
- [ ] Limitations of each method acknowledged

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Over-interpreting funnel plot with k < 10 | Random variation mimics asymmetry | State explicitly that visual inspection is unreliable at small k |
| Equating asymmetry with publication bias | Heterogeneity and small-study effects also cause asymmetry | Test for heterogeneity first; consider meta-regression |
| Reporting only fail-safe N | Widely criticized; reviewers may flag as insufficient | Use as supplementary, never sole evidence |
| Not reporting adjusted estimate | Reader cannot assess practical impact | Always show original vs trim-and-fill estimate side by side |
| Ignoring outcome reporting bias | Studies may report only significant outcomes | Consider within-study selective reporting assessment |

## Minimal Output Format

```markdown
# Publication Bias Assessment

## Feasibility
- Number of studies: k = [n]
- Methods feasible: [funnel + Egger's / funnel only / all methods]
- Methods not feasible: [reason]

## Funnel Plot
[Describe symmetry pattern; embed or reference plot]

## Statistical Tests

| Test | Statistic | p-value | Interpretation |
|------|-----------|---------|----------------|
| Egger's regression | intercept = ... | p = ... | [significant / non-significant small-study effect] |
| Begg's rank | τ = ... | p = ... | ... |
| Fail-safe N | N = ... | — | [robust / not robust] per 5k+10 rule |

## Correction

| Method | Original estimate | Adjusted estimate | Imputed studies | Conclusion change? |
|--------|------------------|------------------|-----------------|-------------------|
| Trim-and-fill | ... | ... | ... | Yes / No |

## Synthesis
[Calibrated paragraph integrating all evidence]

## Limitations
- ...
```
