# Meta-analysis Results Template

<!--
Usage:
- Use this template to write up quantitative synthesis results (and link to code/plots).
- Aligns with PRISMA 2020 results items 20b–20d and 21–22 as applicable.
Save to: RESEARCH/[topic]/meta_analysis_results.md
-->

# Meta-analysis Results

## Review: [Your Review Title]
## Date: [Date]

---

## 1) Overview

Summarize:
- Outcomes synthesized quantitatively (n outcomes)
- Total studies and participants contributing (per outcome)
- Model choice (fixed/random) and key assumptions

---

## 2) Per-outcome Results

Repeat this section for each outcome.

### Outcome [O1]: [Name]

**Contributing studies:** k = [#]  
**Total participants:** N = [#] (if applicable)  
**Effect measure:** [log(OR)/SMD/etc]  
**Model:** [Random-effects (REML/DL) / Fixed-effect]  

#### Pooled estimate
- Pooled effect: [estimate]
- 95% CI: [low, high]
- p-value (if reported): [ ]

#### Heterogeneity
- Q(df) = [ ], p = [ ]
- I² = [ ]%
- τ² = [ ]

#### Visualizations
- Forest plot: [path or link]
- Funnel plot (if applicable): [path or link]

#### Subgroup / Meta-regression (if any)
- Subgroup definitions:
- Results:

#### Sensitivity analyses
- Leave-one-out summary:
- Excluding high RoB:
- Alternative estimator/model:

#### Risk of bias due to missing results (publication bias)
- Assessment performed: [Y/N]
- Method(s): funnel / Egger / trim-fill
- Summary and interpretation:

#### Certainty of evidence (optional)
- GRADE/CERQual judgement: [High/Moderate/Low/Very low]
- Rationale summary:

---

## 3) Cross-outcome Summary Table

| Outcome ID | k | Effect Measure | Model | Pooled Effect | 95% CI | I² | τ² | Notes |
|-----------:|---:|----------------|-------|---------------|--------|----|----|------|
| O1 | | | | | | | | |
| O2 | | | | | | | | |

---

## 4) Interpretation (Plain Language)

Explain magnitude and practical meaning; note limitations and heterogeneity.

---

## 5) Reproducibility

- Data source: `effect_size_table.md` (+ optional CSV)
- Code: `analysis/` (R/Python scripts)
- Parameters used (τ² estimator, CI method, etc.)

