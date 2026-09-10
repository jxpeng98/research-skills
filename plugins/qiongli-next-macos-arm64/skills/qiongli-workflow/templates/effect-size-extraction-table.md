# Effect Size Extraction Table Template

<!--
Usage:
- Use this table when you plan quantitative pooling (meta-analysis).
- Keep one row per (study, outcome, timepoint, estimand).
Save to: RESEARCH/[topic]/effect_size_table.md
-->

# Effect Size Extraction Table

## Review: [Your Review Title]
## Date: [Date]

---

## A) Pooled-ready Effect Sizes (Preferred)

Fill this table with the effect size already on the **analysis scale** you will pool.

| Study ID | Citation | Outcome ID | Timepoint | Effect Measure | yi (analysis scale) | SE(yi) | 95% CI (low) | 95% CI (high) | Adjusted? | Covariates (if adjusted) | Notes |
|---------:|----------|-----------:|-----------|----------------|---------------------|--------|--------------|---------------|----------|---------------------------|------|
| 1 | | O1 | | log(OR) / log(RR) / SMD / MD / Fisher’s z | | | | | Y/N | | |
| 2 | | O1 | | | | | | | | | |

**Notes**
- `yi` must be on a consistent scale per outcome (e.g., all log(OR)).
- If you only have a 95% CI on the analysis scale: `SE ≈ (upper - lower) / (2 * 1.96)`.
- Record the “positive direction” for each outcome in `meta_analysis_plan.md`.

---

## B) Raw Data (Only If You Need to Compute Effect Sizes)

### B1) Binary Outcomes (2×2)

| Study ID | Outcome ID | Timepoint | a (treat event) | b (treat non-event) | c (ctrl event) | d (ctrl non-event) | Effect Target (OR/RR/RD) | Notes |
|---------:|-----------:|-----------|-----------------|---------------------|----------------|--------------------|---------------------------|------|
| | O1 | | | | | | OR / RR / RD | |

### B2) Continuous Outcomes (Two Groups)

| Study ID | Outcome ID | Timepoint | n_treat | mean_treat | sd_treat | n_ctrl | mean_ctrl | sd_ctrl | Target (MD/SMD) | Notes |
|---------:|-----------:|-----------|---------|------------|---------|--------|-----------|--------|------------------|------|
| | O1 | | | | | | | | MD / SMD | |

### B3) Correlations / Associations

| Study ID | Outcome ID | Timepoint | r | n | Target (Fisher’s z) | Adjusted? | Notes |
|---------:|-----------:|-----------|---|---|----------------------|----------|------|
| | O1 | | | | z | Y/N | |

---

## C) Export to CSV (Recommended for Code)

Export at minimum:
- `outcome_id`, `study_id`, `yi`, `sei`

