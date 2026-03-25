---
id: evidence-synthesizer
stage: E_synthesis
description: "Synthesize evidence narratively, qualitatively, or quantitatively (meta-analysis) with PRISMA-aligned reporting."
inputs:
  - type: ExtractionTable
    description: "Extracted study data"
  - type: QualityTable
    description: "Quality assessment results"
outputs:
  - type: EvidenceTable
    artifact: "synthesis.md"
  - type: SynthesisMatrix
    artifact: "synthesis_matrix.md"
constraints:
  - "Must choose synthesis method based on data heterogeneity"
  - "Must produce structured evidence table with claim-evidence-strength mapping"
failure_modes:
  - "Excessive heterogeneity prevents pooling"
  - "Insufficient studies for quantitative synthesis"
tools: [filesystem, stats-engine]
tags: [synthesis, meta-analysis, narrative, evidence-table, PRISMA]
domain_aware: false
---

# Evidence Synthesizer Skill

Synthesize evidence across included studies into PRISMA-ready results. Supports:
- Narrative synthesis (when pooling is inappropriate)
- Qualitative thematic synthesis
- Quantitative meta-analysis (fixed/random effects)

## When to Use

Use after you have:
- Included studies decided (`screening/full_text.md`)
- Core data extracted (`extraction_table.md` + `notes/`)
- Quality assessed (`quality_table.md`)

## Inputs (Expected Artifacts)

- `RESEARCH/[topic]/extraction_table.md`
- `RESEARCH/[topic]/quality_table.md`
- `RESEARCH/[topic]/notes/` (per-paper notes)
- Optional but recommended:
  - `RESEARCH/[topic]/synthesis_matrix.md` (theme × paper matrix)
  - `RESEARCH/[topic]/effect_size_table.md` (for meta-analysis; use template below)

## Step 0: Choose the Synthesis Strategy (Per Outcome)

Many reviews are **mixed**: one outcome may be meta-analyzable while another needs narrative synthesis.

Use **meta-analysis** when all are true:
- ≥2 studies report the *same (or convertible)* effect measure for the same outcome
- Outcome definition + timepoint are comparable (or can be harmonized)
- Population/intervention/comparator are comparable enough (clinical/methodological heterogeneity is not extreme)

Use **qualitative thematic synthesis** when:
- Primary evidence is qualitative (interviews, ethnography, open-ended responses), or
- You are synthesizing themes/mechanisms/experiences rather than effect estimates

Use **narrative synthesis** when:
- Study designs/outcomes are too heterogeneous to pool, or
- Reporting is insufficient for effect-size conversion, or
- Only one eligible quantitative study exists for an outcome

Document the rationale in your Methods (PRISMA items 12–15, 13d–13f).

## Step 1: Define Outcomes, Grouping Rules, and “Unit of Synthesis”

For each outcome/theme:
1. **Outcome definition** (what exactly is measured)
2. **Eligible measures** (e.g., OR/RR, Hedges g/SMD, correlation r)
3. **Timepoint window** (e.g., post-intervention, 3–6 months)
4. **Grouping variables** (population, setting, intervention type, study design)
5. **Unit of analysis** (per participant / cluster / repeated measures handling)

Create (or fill) a synthesis plan using `templates/meta-analysis-plan.md` even if you end up doing narrative synthesis.

## Step 2A: Quantitative Meta-analysis Workflow

### A1) Build an Effect Size Table

Create `RESEARCH/[topic]/effect_size_table.md` using:
- `templates/effect-size-extraction-table.md`

Guidelines:
- Prefer **adjusted** estimates when the causal model demands it (and report which covariates).
- Do **not** mix incomparable estimands (e.g., odds ratio vs risk ratio) unless you convert carefully and justify.
- If multiple effect sizes per study exist, pre-specify selection rules (e.g., primary endpoint; longest follow-up; intention-to-treat).

### A2) Effect Measure Selection (Common Choices)

- Binary outcomes: log(RR), log(OR), risk difference
- Continuous outcomes: mean difference (MD), standardized mean difference (Hedges g)
- Associations: Fisher’s z (from correlation r), standardized regression coefficients (with caution)

Record the choice per outcome (PRISMA item 12).

### A3) Statistical Model Choice

Default: **random-effects** (heterogeneity is the rule in most applied fields).

Document:
- Fixed vs random effects (and why)
- Between-study variance estimator (e.g., DL/REML)
- Confidence interval method (normal vs Hartung-Knapp, if used)

### A4) Heterogeneity Assessment

Report both:
- Qualitative: clinical + methodological heterogeneity (design, measures, populations)
- Quantitative: τ² and **I²** (interpretation is context-dependent)

If heterogeneity is substantial, consider:
- Subgroup analysis (pre-specified)
- Meta-regression (if enough studies; avoid overfitting)
- Sensitivity analysis (see below)

### A5) Small-study Effects / Publication Bias (When Plausible)

Options (choose appropriately; note limitations):
- Funnel plot (visual)
- Egger’s regression test (typically needs ~10+ studies)
- Trim-and-fill (exploratory)

### A6) Sensitivity / Robustness Analyses

Typical set:
- Leave-one-out
- Exclude high risk-of-bias studies
- Alternative τ² estimator or fixed vs random
- Alternative effect-size extraction choice (e.g., adjusted vs unadjusted)

### A7) Certainty of Evidence (Optional but Recommended)

If relevant, produce:
- `RESEARCH/[topic]/grade_sof.md` using `templates/grade-summary-of-findings.md`

### A8) Outputs (Meta-analysis)

Minimum outputs:
- Meta-analysis plan → `RESEARCH/[topic]/meta_analysis_plan.md`
- Effect size table → `RESEARCH/[topic]/effect_size_table.md`
- Results write-up → `RESEARCH/[topic]/meta_analysis_results.md` (use template)
- Integrate a concise summary into `RESEARCH/[topic]/synthesis.md`

Optional outputs:
- Analysis code (R/Python) in `RESEARCH/[topic]/analysis/`
  - `templates/code/statistics/meta_analysis_metafor.R`
  - `templates/code/statistics/meta_analysis_random_effects.py`

## Step 2B: Qualitative Thematic Synthesis Workflow

1. **Decide synthesis approach** (e.g., thematic synthesis, framework synthesis, meta-ethnography) and justify.
2. **Code extracted findings** from included studies (start inductive, then consolidate).
3. **Build descriptive themes** (what is reported).
4. **Derive analytic themes** (interpretation/mechanisms).
5. **Link themes to context** (population/setting) and study quality.
6. **Assess confidence** (optional): use a lightweight CERQual-style judgement (Methodological limitations, Coherence, Adequacy, Relevance).

Outputs:
- Theme table + narrative → integrate into `RESEARCH/[topic]/synthesis.md`
- (Optional) theme evidence table stored in `RESEARCH/[topic]/synthesis_matrix.md`

## Step 2C: Narrative Synthesis Workflow (When Not Pooling)

Use structured narrative synthesis (avoid pure “study-by-study” summaries):
1. **Group studies** by pre-defined categories (design, population, intervention, outcome measure).
2. **Standardize reporting**: direction of effect, magnitude buckets, certainty by quality grade (A–E).
3. **Tabulate** key findings and risk of bias alongside them.
4. **Explore heterogeneity** qualitatively (why results differ).
5. **Assess robustness**: check if conclusions change when excluding low-quality evidence.

Outputs:
- Tables + narrative in `RESEARCH/[topic]/synthesis.md`
- Update `RESEARCH/[topic]/synthesis_matrix.md` to support transparency

## Templates to Use

- `templates/meta-analysis-plan.md`
- `templates/effect-size-extraction-table.md`
- `templates/meta-analysis-report.md`
- `templates/synthesis-matrix.md`
- `templates/grade-summary-of-findings.md`

## PRISMA Alignment (Quick Map)

- Methods: effect measures, synthesis methods, heterogeneity, sensitivity (Items 12–15, 13d–13f)
- Results: statistical synthesis + heterogeneity + sensitivity (Items 20b–20d)
- Bias due to missing results (Items 14, 21)
- Certainty of evidence (Items 15, 22)

