# Meta-analysis / Evidence Synthesis Plan Template

<!--
Usage:
- Use this document to pre-specify how you will synthesize outcomes (meta-analysis, narrative, qualitative).
- Aligns with PRISMA 2020: items 12–15 and 13d–13f.
Save to: RESEARCH/[topic]/meta_analysis_plan.md
-->

# Evidence Synthesis Plan

## Review: [Your Review Title]
## Topic Folder: `RESEARCH/[topic]/`
## Date: [YYYY-MM-DD]

---

## 1) Outcomes and Synthesis Type

| Outcome ID | Outcome Definition | Primary Timepoint | Eligible Designs | Planned Synthesis | Rationale |
|-----------:|--------------------|-------------------|------------------|------------------|-----------|
| O1 | | | | Meta-analysis / Narrative / Qualitative | |
| O2 | | | | Meta-analysis / Narrative / Qualitative | |

**Direction Convention:** Define what “positive” means for each outcome (e.g., lower score = better).

---

## 2) Study Grouping Rules (Per Outcome)

Define how studies will be grouped for each synthesis:
- Population subgroups:
- Setting/context:
- Intervention/exposure types:
- Comparator types:
- Measurement instruments:
- Follow-up windows:

---

## 3) Data Preparation Rules

### Effect size selection (within a study)
- Primary endpoint selection rule:
- Multiple timepoints rule:
- Multiple eligible measures rule:
- Multiple reports of same study (dedup/merge rule):

### Multi-arm / Cluster / Repeated Measures
- Multi-arm trials handling:
- Cluster designs handling (ICC / effective sample size):
- Repeated measures handling (change scores, correlation assumptions):

### Handling missing or incomplete reporting
- Contact authors? (Y/N)
- Derive SE from CI/p-values? (rules below)
- Imputation rules (if any):

**SE from 95% CI (on the analysis scale):**
- `SE ≈ (upper - lower) / (2 * 1.96)`

---

## 4) Meta-analysis Specification (If Applicable)

### Effect measures (PRISMA item 12)
For each outcome, specify the *analysis scale*:
| Outcome ID | Effect Measure | Analysis Scale | Notes / Conversions |
|-----------:|----------------|----------------|---------------------|
| O1 | OR / RR / SMD / MD / Fisher’s z / etc. | log-scale / raw / z | |

### Model choice and estimators
- Fixed vs random effects (default: random):
- Between-study variance (τ²) estimator: DL / REML / Paule-Mandel / other
- CI method: normal / Hartung-Knapp / other

### Heterogeneity (PRISMA item 13e)
- Statistics to report: I², τ², Q
- Thresholds for action (contextual):
- Planned subgroup analyses:
- Planned meta-regression (only if enough studies; specify covariates):

### Sensitivity analyses (PRISMA item 13f)
Pre-specify:
- Leave-one-out:
- Exclude high risk-of-bias:
- Alternative τ² estimator:
- Alternative effect-size choice (adjusted vs unadjusted):

### Small-study effects / missing results (PRISMA item 14)
Planned assessments (choose as appropriate):
- Funnel plot:
- Egger’s test (requires ~10+ studies; interpret cautiously):
- Trim-and-fill (exploratory):

---

## 5) Certainty / Confidence in Evidence (Optional)

If using GRADE (recommended for clinical/health-style questions):
- Outcomes assessed:
- Downgrade/upgrade rules:
- Planned Summary of Findings table: `RESEARCH/[topic]/grade_sof.md`

---

## 6) Software / Reproducibility

- Data source: `RESEARCH/[topic]/effect_size_table.md` (and optional CSV export)
- Code location: `RESEARCH/[topic]/analysis/`
- Tools:
  - R: `metafor`, `meta`
  - Python: pandas/numpy + custom random-effects implementation

---

## 7) Deviations / Amendments Log

| Date | Change | Rationale |
|------|--------|-----------|
| | | |

