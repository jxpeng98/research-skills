---
id: code-review
stage: I_code
version: "1.0.0"
description: "Secondary model reviews code logic, security, statistical validity, and domain-specific correctness."
inputs:
  - type: AnalysisCode
    description: "Code to review"
  - type: DomainProfile
    description: "Domain checklist for review"
    required: false
outputs:
  - type: CodeReview
    artifact: "code/code_review.md"
constraints:
  - "Must check statistical correctness against domain checklist"
  - "Must verify random seed handling and reproducibility"
failure_modes:
  - "Reviewer lacks domain expertise for statistical validation"
  - "False positives on acceptable coding patterns"
tools: [filesystem]
tags: [code, review, security, statistical-validity, domain-checklist]
domain_aware: true
---

# Code Review Skill

Independent review of research code for correctness, reproducibility, and statistical validity.

## Related Task IDs

- `I8` (code review)

## Output (contract path)

- `RESEARCH/[topic]/code/code_review.md`

## Domain Integration

When `--domain` is specified, load the corresponding `skills/domain-profiles/*.yaml` and apply:
- Domain-specific **common_pitfalls** as mandatory review items
- Domain-specific **stats_diagnostics** as validation checkpoints
- Domain-specific **method_templates.checklist** for each detected method

## Review Checklist

### Correctness
- Does the implementation match the method/spec (I5)?
- Are edge cases handled explicitly?
- Are there silent failure modes (NaNs, empty slices, wrong joins)?
- Do loops/vectorizations match mathematical definitions exactly?

### Statistical validity
- Are estimands and standard errors computed correctly?
- Are assumptions checked (or at least flagged)?
- Are multiple comparisons / leakage risks addressed (if relevant)?
- Is the effect size reported alongside p-values?

### Reproducibility
- Fixed seeds where appropriate
- Deterministic pipelines documented (versions, configs)
- Clear rerun instructions in `code/documentation/`
- Container config (Docker/Singularity) if applicable

### Security / data safety (as applicable)
- No secrets in code
- Safe file I/O paths
- Privacy constraints respected (D3)

## Domain-Specific Review Rules

### Economics / Econometrics
- [ ] Clustered SE at correct level (not observation-level for panel)
- [ ] No look-ahead bias in feature construction
- [ ] Staggered DID uses robust estimator (not naïve TWFE)
- [ ] First-stage F-stat reported for IV
- [ ] Pre-treatment balance table present

### Finance
- [ ] Look-ahead bias check (no future data in features)
- [ ] Survivorship bias documented (delisted firms handled)
- [ ] Transaction costs / market impact acknowledged in backtests
- [ ] Overlapping observations use Newey-West correction

### Psychology / Behavioral
- [ ] SEM fit indices reported (CFI, RMSEA, SRMR)
- [ ] Reliability reported (α and ω, not just α)
- [ ] Mediation uses bootstrap CI (not Baron & Kenny)
- [ ] Common method bias assessed
- [ ] Scale translation procedure documented (if applicable)

### Biomedical / Clinical
- [ ] PH assumption tested (Schoenfeld) for Cox models
- [ ] EPV ≥ 10 in logistic regression
- [ ] CONSORT flow diagram present for RCT
- [ ] Multiple imputation used (not single imputation)
- [ ] ITT as primary analysis (per-protocol as sensitivity)

### Education
- [ ] Nested structure modeled (HLM, not naïve OLS)
- [ ] Pre-test controlled (ANCOVA, not gain scores)
- [ ] IRT fit statistics within acceptable range
- [ ] No student leakage across CV folds

### CS / AI
- [ ] Results averaged over ≥ 3 random seeds (report mean ± std)
- [ ] Ablation study included
- [ ] Fair baseline comparison (same hyperparameter budget)
- [ ] Data contamination check for LLM evaluations
- [ ] Compute cost reported (GPU hours, model size)
- [ ] No train/test leakage

### Political Science / Sociology
- [ ] Survey weights applied correctly
- [ ] Post-treatment variables not used as controls
- [ ] Topic model validated (semantic coherence + hand-coding)
- [ ] Replication data/code deposited

### Epidemiology
- [ ] DAG drawn and adjustment set justified
- [ ] E-value computed for unmeasured confounding
- [ ] Immortal time bias ruled out
- [ ] Informative censoring assessment present

### Ecology / Environmental
- [ ] Pseudo-replication ruled out
- [ ] Spatial autocorrelation tested (Moran's I on residuals)
- [ ] Zero-inflation addressed if count data
- [ ] SDM: spatial block cross-validation used

## Minimal review format (`code/code_review.md`)

```markdown
# Code Review

## Summary

## Domain: [domain]

## High-severity findings
1. ...

## Medium / low findings
- ...

## Domain-specific checklist
- [ ] Item 1 (from domain profile)
- [ ] Item 2
- ...

## Reproducibility notes
- ...

## Suggested fixes (ordered)
1. ...
```
