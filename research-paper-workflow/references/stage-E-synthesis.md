# Stage E — Evidence Synthesis / Meta-analysis (E1–E5)

This stage produces defensible synthesis: narrative, qualitative, and/or quantitative pooling.

## Canonical outputs (contract paths)

- `E1` → `meta_analysis_plan.md`
- `E2` → `effect_size_table.md`
- `E3` → `meta_analysis_results.md`
- `E3_5` → `synthesis/publication_bias.md`
- `E4` → `grade_sof.md`
- `E5` → `synthesis.md`, `synthesis/qualitative_data_dictionary.md`, `synthesis/thematic_codebook.md`

Supporting artifact (core for transparency):
- `synthesis_matrix.md`
- `synthesis/qualitative_data_dictionary.md` and `synthesis/thematic_codebook.md` when the synthesis includes qualitative evidence

## Quality gate focus

- `Q2` (claim-evidence traceability): synthesis statements must map back to extracted evidence.
- `Q4` (reproducibility baseline): analytic choices and transformations are documented.

---

## E1 — Synthesis Strategy / Meta-analysis Plan

**Definition of done**
- For each outcome: choose synthesis type and justify (pool vs narrative vs qualitative)
- Pre-specify effect measure(s), model(s), heterogeneity handling, and sensitivity plan
- Document exclusion rules for pooling (e.g., incomparable measures)

Minimum structure: `meta_analysis_plan.md`

```markdown
# Meta-analysis / Synthesis Plan

## Outcomes & synthesis choice
| Outcome | Synthesis type | Rationale |
|---|---|---|

## Effect measures
- Primary:
- Conversions / standardizations:

## Model
- Fixed vs random:
- Estimator:

## Heterogeneity
- Metrics (I², τ²):
- Subgroup / moderators (if any):

## Sensitivity / robustness
- Leave-one-out:
- Influence diagnostics:
- Alternative priors/models:

## Missing-results bias
- Planned checks (E3_5):
```

---

## E2 — Effect Size Table (Pool-ready)

**Definition of done**
- One row per study × outcome (or per comparison arm)
- Effect size + uncertainty computed or derivable
- Clear mapping to the source paper section/table/figure

Recommended minimal columns: `effect_size_table.md`

```markdown
| study_id | citekey | outcome | effect_type | effect | se_or_ci | n_treat | n_ctrl | notes | source_location |
|---|---|---|---|---:|---|---:|---:|---|---|
```

Rule of thumb: if a future-you cannot recompute the value from the paper + notes, it’s not reproducible.

---

## E3 — Meta-analysis Execution & Write-up

**Definition of done**
- Pooled estimates per outcome with uncertainty
- Heterogeneity reported and interpreted
- Sensitivity analyses reported
- Figures/tables referenced (forest plot, etc.) if generated
- If magnitude interpretation will feed the manuscript discussion, key translations can be carried forward into `manuscript/effect_interpretation.md`

Write into: `meta_analysis_results.md`.

Include:
- model choice and rationale
- handling of multi-arm trials / dependency (if applicable)
- decision log for exclusions

---

## E3_5 — Publication Bias / Missing-Results Bias

**Definition of done**
- At least one bias check appropriate for the dataset (not always possible with small k)
- Interpretation is cautious (bias checks are low power)

Write into: `synthesis/publication_bias.md`.

Recommended sections:
- funnel plot notes (if applicable)
- Egger/Begg test notes (if applicable)
- trim-and-fill / selection model discussion (if applicable)

---

## E4 — Certainty Grading (GRADE-style)

**Definition of done**
- One Summary-of-Findings row per critical outcome
- Reasons for downgrading/upgrading are explicit

Write into: `grade_sof.md`.

---

## E5 — Integrated Synthesis (Narrative + Quant)

**Definition of done**
- `synthesis.md` conclusions are supported by extraction + quality tables
- `synthesis_matrix.md` makes the narrative traceable (themes × papers)
- Qualitative synthesis tracks constructs and theme definitions in `synthesis/qualitative_data_dictionary.md` and `synthesis/thematic_codebook.md`
- Limitations and certainty statements are calibrated (avoid overclaim)
- Downstream manuscript drafting can reuse concise interpretation notes from E3/E5 in `manuscript/results_interpretation.md`

Suggested structure: `synthesis.md`

```markdown
# Synthesis

## Included studies overview
- N included:
- Designs:

## Findings by outcome/theme
### Outcome/theme 1
- What the evidence suggests:
- Strength/uncertainty:
- Key supporting studies:

## Heterogeneity & moderators (if relevant)

## Risk of bias / certainty summary

## Implications & gaps
```
