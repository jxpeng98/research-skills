---
id: econ-identification-auditor
stage: C_design
description: "Audit economics identification strategies, threats, assumptions, and robustness requirements."
inputs:
  - type: DesignSpec
    description: "Study design and identification strategy"
  - type: AnalysisPlan
    description: "Estimator, variables, and planned robustness checks"
outputs:
  - type: IdentificationAudit
    artifact: "analysis/identification_audit.md"
constraints:
  - "Must distinguish causal claims from descriptive claims"
  - "Must name the identifying variation and comparison group"
  - "Must surface blocked diagnostics instead of inventing evidence"
failure_modes:
  - "Identification strategy unsupported by available variation"
  - "Estimator does not match treatment timing or assignment mechanism"
  - "Robustness plan omits the main threat to validity"
tools: [filesystem]
tags: [economics, identification, causal-inference, robustness]
domain_aware: true
---

# Economics Identification Auditor Skill

Audit the identification argument before economics results are written or interpreted.

## Purpose

Check whether an economics paper's causal or structural claim is supported by its design, data, estimator, and robustness plan.

## When to Use

- Before finalizing an empirical strategy section.
- Before interpreting DID, IV, RD, synthetic control, matching, panel, or structural estimates as causal.
- When reviewer risk centers on omitted variables, selection, timing, spillovers, weak instruments, or specification search.

## Inputs

- `DesignSpec`: treatment, outcome, unit, timing, assignment mechanism, comparison group, and identifying assumption.
- `AnalysisPlan`: estimator, controls, fixed effects, clustering, robustness checks, and planned tables/figures.
- `DomainProfile`: load `skills/domain-profiles/economics.yaml` when available.

If any input is missing, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and list the blocked audit checks.

## Process

1. Classify the claim as descriptive, causal reduced-form, structural, predictive, or theory-only.
2. Name the identifying variation in one sentence.
3. Match the design to the economics method template from `skills/domain-profiles/economics.yaml`.
4. Audit assumptions, diagnostics, estimator fit, standard-error plan, and robustness checks.
5. Identify the strongest alternative explanation and the evidence needed to rule it out.
6. Produce a pass / revise / blocked verdict.

## Output Contract

Write `RESEARCH/[topic]/analysis/identification_audit.md`:

```markdown
# Identification Audit

## Claim Type
- Claim:
- Classification:

## Identifying Variation
- Source:
- Comparison group:
- Timing:

## Assumptions
| Assumption | Evidence available | Gap or risk |
|---|---|---|

## Diagnostics and Robustness
| Required check | Status | Notes |
|---|---|---|

## Threats
- Main threat:
- Alternative explanations:
- Spillover or anticipation risk:

## Verdict
- Decision: pass / revise / blocked
- Required changes:
```

## Quality Bar

- [ ] The audit states the estimand and identifying variation.
- [ ] The estimator matches the treatment timing and data structure.
- [ ] The standard-error and clustering plan is explicit.
- [ ] The main validity threat has a targeted diagnostic or a stated gap.
- [ ] The final verdict does not upgrade unsupported evidence into a causal claim.
