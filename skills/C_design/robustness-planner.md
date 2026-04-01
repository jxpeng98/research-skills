---
id: robustness-planner
stage: C_design
description: "Pre-specify robustness checks, sensitivity analysis, and bounds scaling linked to specific identification threats."
inputs:
  - type: DesignSpec
    description: "Study design with identification strategy"
  - type: AnalysisPlan
    description: "Primary model and assumptions"
outputs:
  - type: RobustnessPlan
    artifact: "design/robustness_plan.md"
constraints:
  - "Must link each check to a specific threat or assumption"
  - "Must pre-specify pass/fail interpretation (what changes conclusion)"
  - "Must cover both model and data robustness"
failure_modes:
  - "Robustness table is a checklist without threat-specific motivation"
  - "Too many checks without prioritization inflates researcher degrees of freedom"
  - "Qualitative robustness dismissed as 'not applicable'"
tools: [filesystem]
tags: [design, robustness, sensitivity-analysis, endogeneity, trustworthiness]
domain_aware: true
---

# Robustness Planner Skill

Pre-specify robustness checks so reviewers see that you anticipated threats — not that you searched for favorable results post-hoc.

## Purpose

Pre-specify robustness checks, sensitivity analysis, and bounds scaling linked to specific identification threats.

## Related Task IDs

- `C3_5` (robustness/sensitivity plan)

## Output (contract path)

- `RESEARCH/[topic]/design/robustness_plan.md`

## When to Use

- After analysis plan (C3) is drafted
- Before data analysis begins (prevents "garden of forking paths")
- When the identification strategy has known weaknesses
- For qualitative research: before finalizing coding and interpretation

## Process

### Step 1: Map Threats to Validity

For each research question and hypothesis, identify the key threats:

#### For Quantitative / Causal Designs

| Threat Category | Common Threats | Typical Checks |
|-----------------|---------------|----------------|
| **Omitted variables** | Unobserved confounders | Instrumental variables, bounds (Oster's δ), control function |
| **Selection bias** | Non-random treatment assignment | Propensity score matching, Heckman selection |
| **Reverse causality** | DV → IV instead of IV → DV | Granger causality, lagged IV, natural experiment |
| **Functional form** | Wrong model specification | Nonlinear terms, splines, Box-Cox transformation |
| **Measurement error** | Noisy variables attenuate estimates | Instrumental variables for measurement, reliability correction |
| **Sample composition** | Results driven by outliers or subgroup | Winsorization, jackknife, subsample analysis |
| **Temporal sensitivity** | Results depend on time window | Vary window start/end, placebo time periods |
| **Spatial/clustering** | Standard errors underestimated | Cluster at different levels, wild bootstrap |
| **Multiple testing** | Type I error inflation | Bonferroni, Holm, Benjamini-Hochberg FDR |

#### For Qualitative / Interpretive Designs

| Trustworthiness Criterion | Threat | Robustness Check |
|---------------------------|--------|-----------------|
| **Credibility** | Single-coder bias | Independent double-coding with inter-rater check |
| **Credibility** | Confirmation bias in themes | Actively seek and report disconfirming cases |
| **Credibility** | Informant bias | Triangulate across data sources (interviews + documents + observations) |
| **Transferability** | Overclaiming scope | Thick description of context; compare across cases/sites |
| **Dependability** | Process not auditable | Maintain audit trail (codebook evolution, memo trail, decision log) |
| **Confirmability** | Researcher positionality affects findings | Reflexivity statement; peer debriefing; member checking |

### Step 2: Design the Robustness Table

For each check, specify:

| # | Threat | Check | What Changes | Pass/Fail Criterion | Priority |
|---|--------|-------|-------------|---------------------|----------|
| R1 | Omitted variable bias | Oster's δ (2019) bounds | Nothing (diagnostic) | δ > 1 → robust to proportional selection | Must-run |
| R2 | Outlier sensitivity | Winsorize at 1/99% | Coefficient + significance | Sign and significance stable | Must-run |
| R3 | Functional form | Add quadratic term | Model fit + coefficient | AIC/BIC improves? Coefficient sign changes? | Should-run |
| R4 | Cluster SE | Cluster at firm vs individual | Standard errors + inference | Significance holds at both levels | Must-run |
| R5 | Alternative DV | Replace proxy with alternative measure | Coefficient magnitude | Same direction, similar magnitude | Should-run |
| R6 | Subsample stability | Drop each industry/country | Coefficient | No single group drives the result | Nice-to-have |

> **Rule of thumb**: 3–5 must-run checks, 2–3 should-run, and a few nice-to-have. More than 10 robustness checks suggest unfocused design — prioritize by threat severity.

### Step 3: Pre-specify Interpretation Rules

Before running any checks, commit to how you will interpret results:

```
If the primary result [sign + significance] holds across all must-run checks:
→ Report as "robust result"

If one must-run check fails:
→ Report which check failed, discuss why, and narrow the claim scope

If multiple must-run checks fail:
→ The primary result is not robust; discuss as suggestive evidence only

If a robustness check produces a STRONGER result:
→ Report but do not upgrade the primary claim (pre-registration discipline)
```

### Step 4: Plan Sensitivity Analyses

Beyond robustness checks, plan formal sensitivity analyses when applicable:

| Method | When to Use | Interpretation |
|--------|------------|----------------|
| **Oster (2019) δ** | Observational studies with selection concern | δ > 1: unobservables would need to be more important than observables to explain away the result |
| **Rosenbaum bounds** | Matched designs | Γ value at which significance breaks |
| **E-value** | Any observational estimate | Minimum confounder strength to explain away result |
| **Leave-one-out** | Meta-analysis with influential studies | Check if one study drives the pooled result |
| **Alternative coding** | Qualitative research | Re-code with alternative theoretical lens; compare themes |
| **Member checking** | Qualitative research | Return findings to participants; document convergence/divergence |

### Step 5: Document the Reporting Commitment

Pre-commit to reporting **all** robustness results, including failures:

- [ ] All must-run checks will be reported in the main paper (table or text)
- [ ] Should-run and nice-to-have checks will be reported in appendix/supplement
- [ ] Failed checks will be discussed with explanation, not hidden
- [ ] The robustness section will explicitly state which threat each check addresses

## Quality Bar

The robustness plan is **ready** when:

- [ ] Every check is linked to a specific threat (no unmotivated checks)
- [ ] Pass/fail criteria are pre-specified (not post-hoc)
- [ ] Must-run vs should-run vs nice-to-have prioritized
- [ ] Interpretation rules written before data analysis
- [ ] Reporting commitment documented
- [ ] For qualitative: trustworthiness procedures are specific (not just "member checking" without detail)

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Running 20+ robustness checks | Inflates degrees of freedom; looks like fishing | Prioritize 5–8 checks linked to real threats |
| "Robustness" checks that are just model variations | Varying controls without threat motivation | Each check must name the threat it addresses |
| Reporting only checks that "pass" | Selective reporting of robustness = another bias | Pre-commit to reporting all must-run results |
| Ignoring qualitative robustness | Reviewers increasingly expect trustworthiness procedures | Include disconfirming cases, triangulation, audit trail |
| No pre-specification | Robustness analysis decided after seeing results | Write robustness plan before touching data |

## Minimal Output Format

```markdown
# Robustness / Sensitivity Plan

## Threat Inventory
| Threat | Severity | Source | Addressed by |
|--------|---------|--------|--------------|

## Robustness Checks

| # | Threat | Check | Changes | Pass/Fail Criterion | Priority |
|---|--------|-------|---------|---------------------|----------|

## Sensitivity Analyses
| Method | Parameter | Threshold |
|--------|-----------|-----------|

## Interpretation Rules
- All must-run pass: ...
- One must-run fails: ...
- Multiple must-run fail: ...

## Reporting Commitment
- [ ] All must-run in main text
- [ ] All others in supplement
- [ ] Failed checks discussed with explanation
```
