---
id: finance-identification-risk-auditor
stage: C_design
description: "Audit finance identification, risk adjustment, return construction, factor exposure, and bias controls."
inputs:
  - type: DesignSpec
    description: "Finance question, claim type, empirical design, event window, asset universe, and target venue"
  - type: DataDictionary
    description: "Return, price, accounting, holdings, transaction, analyst, or macro variables and filters"
  - type: AnalysisPlan
    description: "Expected-return model, estimator, factor controls, standard errors, and robustness checks"
outputs:
  - type: FinanceIdentificationRiskAudit
    artifact: "analysis/finance_identification_risk_audit.md"
constraints:
  - "Must distinguish alpha, risk compensation, mispricing, corporate policy, and causal mechanism claims"
  - "Must audit look-ahead bias, survivorship bias, event-date leakage, and risk-adjusted benchmark choice"
  - "Must hold undergraduate-and-above work to doctoral-level journal evidence standards"
failure_modes:
  - "Risk-adjusted return claim uses an unsuitable factor model or benchmark"
  - "Portfolio, event-study, or panel design leaks future information"
  - "Corporate finance interpretation makes causal claims without identification support"
tools: [filesystem]
tags: [finance, asset-pricing, risk-adjusted, event-study, identification]
domain_aware: true
---

# Finance Identification And Risk Auditor Skill

Audit whether a finance manuscript's empirical design can support its asset pricing, corporate finance, market microstructure, event-study, or risk claim.

## Purpose

Prevent finance results from being interpreted as alpha, mispricing, risk compensation, or causal policy effects before return construction, benchmark choice, timing, and bias controls are defensible.

## When to Use

- Before finalizing an asset pricing, corporate finance, market microstructure, event-study, or risk management design.
- Before interpreting abnormal returns, factor alphas, event-study CARs, firm policy coefficients, or liquidity measures.
- When a project started from undergraduate or master's research needs to be raised to doctoral-level journal standards.

## Inputs

- `DesignSpec`: claim type, unit, sample, asset universe, event timing, comparison group, and target venue.
- `DataDictionary`: return frequency, delisting treatment, market data source, accounting variables, event dates, and filters.
- `AnalysisPlan`: expected-return model, factor set, estimator, standard errors, clustering, and robustness checks.
- `DomainProfile`: load `skills/domain-profiles/finance.yaml` when available.

If any input is missing, write a blocked-check note under `RESEARCH/[topic]/context/gap_notes.md`.

## Process

1. Classify the finance claim as descriptive, asset pricing, risk-adjusted performance, event study, corporate finance, market microstructure, or causal.
2. Name the return construction, benchmark model, factor set, and event window where applicable.
3. Audit look-ahead bias, survivorship bias, delisting returns, stale prices, event-date leakage, and overlapping observations.
4. Check whether standard errors, clustering, Newey-West corrections, or Fama-MacBeth steps match the data structure.
5. Identify the strongest alternative risk or mechanism explanation.
6. Produce a pass / revise / blocked verdict.

## Output Contract

Write `RESEARCH/[topic]/analysis/finance_identification_risk_audit.md`:

```markdown
# Finance Identification And Risk Audit

## Claim Classification
- Claim:
- Classification:
- Required benchmark:

## Return And Risk Construction
| Component | Current plan | Risk | Required revision |
|---|---|---|---|

## Bias Controls
| Bias | Status | Evidence | Gap |
|---|---|---|---|

## Inference Plan
| Design element | Standard-error or test plan | Concern |
|---|---|---|

## Alternative Explanations
- Risk-based alternative:
- Microstructure alternative:
- Corporate policy alternative:

## Verdict
- Decision: pass / revise / blocked
- Required changes:
```

## Quality Bar

- Every risk-adjusted claim names the benchmark model and factor set.
- Return construction handles delisting, survivorship, timing, and look-ahead bias.
- Event studies define event dates, estimation windows, event windows, benchmark models, and confounding-event rules.
- Corporate finance causal claims separate identification from risk-adjusted association.
- The final verdict holds undergraduate-and-above projects to doctoral-level journal evidence standards.
