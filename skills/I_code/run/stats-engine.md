---
id: stats-engine
stage: I_code
version: "0.2.2"
description: "Statistical modeling, hypothesis testing, and analytics execution with domain-profile-driven diagnostics."
inputs:
  - type: AnalysisPlan
    description: "Statistical analysis specification"
  - type: DomainProfile
    description: "Domain-specific diagnostics and methods"
    required: false
outputs:
  - type: StatsReport
    artifact: "analysis/stats_report.md"
constraints:
  - "Must run domain-specified diagnostics when profile is provided"
  - "Must report effect sizes alongside p-values"
failure_modes:
  - "Data violates model assumptions"
  - "Convergence failures in complex models"
tools: [filesystem, stats-engine, code-runtime]
tags: [code, statistics, modeling, hypothesis-testing, diagnostics]
domain_aware: true
---

# Stats Engine Skill

Run and report statistical analyses in a way that supports peer review and reproducibility.

## When to Use

- You need to execute modeling/testing for synthesis (`E3/E3_5`) or empirical results (`F` stage) with a clear report of assumptions and uncertainty.

## Output (recommended path)

- `RESEARCH/[topic]/analysis/stats_report.md`

## Inputs (ask for)

- Estimand(s) / hypothesis / outcome definitions
- Data schema (columns, units, missingness expectations)
- Model family preferences (GLM, mixed models, time series, causal estimators, etc.)
- Domain (optional): loads domain-specific diagnostics from `skills/domain-profiles/`

## Method Selection Decision Tree

Use this tree to recommend the appropriate statistical approach based on the research goal and data structure. Always justify why the selected method fits the research question.

```text
What is the research goal?
│
├── Compare groups
│   ├── 2 groups
│   │   ├── Independent → t-test (parametric) / Mann-Whitney (non-parametric)
│   │   └── Paired → paired t-test / Wilcoxon signed-rank
│   ├── ≥ 3 groups
│   │   ├── Independent → one-way ANOVA / Kruskal-Wallis
│   │   └── Repeated → RM-ANOVA / Friedman / mixed model
│   └── Factorial (≥2 factors) → factorial ANOVA / mixed ANOVA
│
├── Predict / explain (association)
│   ├── Continuous outcome
│   │   ├── Linear relationship → OLS / robust OLS
│   │   ├── Regularization needed → Ridge / Lasso / Elastic Net
│   │   ├── Non-linear → GAM / polynomial / random forest
│   │   └── Nested data → HLM / mixed model (lme4)
│   ├── Binary outcome → logistic regression
│   ├── Count outcome → Poisson / negative binomial / zero-inflated
│   ├── Ordinal outcome → ordinal logistic (proportional odds)
│   ├── Time-to-event → Cox PH / AFT / competing risks
│   └── Latent structure
│       ├── Measurement → CFA / IRT
│       ├── Structural → SEM / path analysis
│       └── Dimensionality → EFA / PCA
│
├── Causal inference
│   ├── Randomized (RCT)
│   │   ├── Simple → ANCOVA (post ~ pre + treatment)
│   │   ├── Clustered → cluster-robust SE / GEE / mixed model
│   │   └── Non-compliance → ITT + CACE (IV)
│   ├── Quasi-experimental
│   │   ├── Before/after + control → DID (check parallel trends)
│   │   ├── Threshold assignment → RD (local polynomial, rdrobust)
│   │   ├── Excludable instrument → 2SLS / IV
│   │   ├── Synthetic comparator → Synthetic control / Synthetic DID
│   │   └── Staggered treatment → Callaway-Sant'Anna / Sun-Abraham
│   └── Observational
│       ├── Selection-on-observables → matching / IPTW / AIPW
│       ├── DAG-identified → adjustment set from d-separation
│       └── Sensitivity → E-value / Oster bounds / sensemakr
│
├── Time series / longitudinal
│   ├── Univariate → ARIMA / exponential smoothing
│   ├── Multivariate → VAR / VECM
│   ├── Volatility → GARCH family
│   ├── Panel data
│   │   ├── Individual + time effects → FE / RE (Hausman test)
│   │   └── Dynamic → system GMM (Arellano-Bond)
│   └── Growth / trajectory → latent growth curve / HLM
│
├── Dimensionality reduction / clustering
│   ├── Reduce dimensions → PCA / t-SNE / UMAP
│   ├── Group observations → k-means / hierarchical / DBSCAN
│   └── Mixture → Gaussian mixture models / latent class
│
└── Meta-analysis
    ├── Effect sizes comparable → random-effects meta-analysis
    ├── Heterogeneity exploration → meta-regression / subgroup
    ├── Publication bias → funnel plot / Egger / trim-and-fill
    └── Individual participant data → IPD meta-analysis
```

## Procedure

1. **Select model(s)** aligned to estimand and data type (use decision tree above).
2. **Load domain diagnostics** from domain profile if `--domain` is specified.
3. **Run diagnostics** (assumptions, residual patterns, convergence).
4. **Report effect sizes + uncertainty** (CI/SE), not only p-values.
5. **Record robustness checks** and sensitivity analyses.
6. **Domain checklist**: verify domain-specific diagnostics are addressed.

## Domain-Specific Diagnostic Quick Reference

### Economics / Econometrics
- Parallel trends (DID) — event study pre-period F-test
- First-stage F-stat > 10 (IV)
- Clustering SE at treatment level
- Stationarity (ADF/KPSS for time series)

### Psychology / Behavioral
- Model fit (CFI ≥ 0.95, RMSEA ≤ 0.06, SRMR ≤ 0.08)
- Reliability (α, ω)
- Common method bias (Harman single-factor)
- Effect sizes (Cohen's d, η², ω²)

### Biomedical / Clinical
- PH assumption (Schoenfeld test)
- Events per variable (EPV ≥ 10)
- Missing data pattern → multiple imputation
- Calibration + discrimination (AUC, Hosmer-Lemeshow)

### CS / AI
- Cross-validation with proper stratification
- Statistical test across folds (paired t / Wilcoxon)
- Report variance across random seeds
- Baseline comparison with same hyperparameter budget

### Ecology / Environmental
- Overdispersion check (quasi-AIC)
- Spatial autocorrelation (Moran's I)
- Zero-inflation (Vuong test)
- Pseudo-replication audit

### Epidemiology
- DAG-identified adjustment set
- E-value for unmeasured confounding
- Informative censoring assessment
- Immortal time bias check

## Common Pitfalls (Cross-Domain)

| Pitfall | Impact | Fix |
|---------|--------|-----|
| Reporting only p-values | Low reproducibility | Always report effect size + CI |
| Multiple comparisons without correction | Inflated Type I error | Bonferroni / Holm / FDR |
| Ignoring nested data | Inflated standard errors | HLM / cluster-robust SE |
| Post-treatment control | Biased causal estimate | Only condition on pre-treatment |
| Single imputation | Underestimated uncertainty | Multiple imputation (MICE / FIML) |
| Confusing OR with RR | Misleading interpretation | Use RR when prevalence > 10% |

## Minimal report format (`analysis/stats_report.md`)

```markdown
# Statistical Report

## Data
- N:
- Missingness:
- Domain:

## Method selection rationale
- Goal:
- Data structure:
- Selected model(s):

## Models
- Primary:
- Alternatives/robustness:

## Results
| Outcome | Effect | Uncertainty | Interpretation |
|---|---:|---|---|

## Diagnostics
- Domain-specific checks:

## Robustness
- Sensitivity analyses:

## Caveats
```
