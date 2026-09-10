---
id: economics-accounting-bridge-auditor
stage: H_submission
description: "Audit economics-accounting bridge drafts for identification, accounting construct validity, disclosure institutions, fiscal timing, and composite reviewer risk."
inputs:
  - type: DesignSpec
    description: "Research question, estimand, identifying variation, accounting construct, empirical proxy, sample, and outcome window"
  - type: LiteratureMap
    description: "Economics, accounting, disclosure, audit, reporting, and capital-market literatures"
  - type: Manuscript
    description: "Draft argument, design, measurement, findings, contribution framing, and reviewer-risk discussion when available"
outputs:
  - type: EconomicsAccountingBridgeAudit
    artifact: "analysis/economics_accounting_bridge_audit.md"
constraints:
  - "Must connect economics identification standards to accounting construct and proxy validity"
  - "Must name the disclosure, audit, reporting, or fiscal institution that creates the bridge setting"
  - "Must align fiscal timing with capital-market or firm outcome windows before claim calibration"
failure_modes:
  - "Identification is strong but the accounting construct or proxy is underspecified"
  - "Accounting disclosure setting is described without an estimand or identifying variation"
  - "Contribution framing satisfies one field while leaving the other field's reviewer standard implicit"
tools: [filesystem]
tags: [economics-accounting, identification, measurement, disclosure, fiscal-window, reviewer-risk]
domain_aware: true
---

# Economics-Accounting Bridge Auditor Skill

Audit whether an economics-accounting manuscript connects economic identification, accounting construct measurement, disclosure or reporting institutions, and outcome timing tightly enough for reviewers in both fields.

## Purpose

Prevent a bridge project from passing as either generic economics with accounting labels or generic accounting with causal language. The audit should make the estimand, proxy, institutional setting, and contribution burden explicit before final writing or submission.

## When to Use

- Before finalizing the theory, empirical design, introduction, or discussion for an economics-accounting bridge project.
- When the paper combines an economics-style estimand or identifying variation with accounting disclosures, audit settings, reporting mandates, fiscal-year panels, or construct validity claims.
- When reviewer risk centers on measurement validity, timing, sample construction, venue fit, or whether the contribution is legible to both fields.

## Inputs

- `DesignSpec`: estimand, identifying variation, accounting construct, empirical proxy, fiscal timing, sample filters, source-item mapping, outcome window, and target venue.
- `LiteratureMap`: economics identification standards, accounting disclosure/reporting/audit literatures, and adjacent capital-market or firm-outcome evidence.
- `Manuscript`: introduction, theory, methods, results, robustness, contribution framing, and limitations when available.
- `DomainProfile`: load `skills/domain-profiles/economics.yaml` when available and pair it with the active economics-accounting runtime manifest.

If the estimand, accounting construct, or institutional disclosure/reporting setting is missing, write a blocked-check note under `RESEARCH/[topic]/context/gap_notes.md` and do not invent the bridge mechanism.

## Process

1. State the bridge claim in one sentence.
2. Map the estimand, identifying variation, accounting construct, empirical proxy, and outcome window.
3. Identify the disclosure, audit, reporting, fiscal, or capital-market institution that makes the setting accounting-specific.
4. Check source-item mapping, sample filters, fiscal-year timing, event timing, and capital-market or firm outcome alignment.
5. Separate identification strength, measurement validity, institutional interpretation, and contribution framing.
6. Compare the current framing against economics and accounting reviewer standards.
7. Produce a pass / revise / blocked verdict.

## Output Contract

Write `RESEARCH/[topic]/analysis/economics_accounting_bridge_audit.md`:

```markdown
# Economics-Accounting Bridge Audit

## Claim Under Review
- Focal bridge claim:
- Estimand:
- Target venue or reviewer standard:
- Claim strength:

## Identification-Measurement Map
| Element | Current statement | Evidence | Gap |
|---|---|---|---|
| Identifying variation | | | |
| Accounting construct | | | |
| Empirical proxy | | | |
| Disclosure or reporting institution | | | |
| Outcome window | | | |

## Timing And Sample Checks
| Risk | Current status | Required check |
|---|---|---|
| Fiscal-year timing | | |
| Event or disclosure window | | |
| Source-item mapping | | |
| Sample filters | | |
| Capital-market or firm outcome timing | | |

## Reviewer Risk
- Economics reviewer objection:
- Accounting reviewer objection:
- Required narrowing:
- Required evidence:

## Verdict
- Decision: pass / revise / blocked
- Required changes:
```

## Quality Bar

- The audit ties accounting construct validity to the estimand and identifying variation.
- Disclosure, audit, reporting, or fiscal institutions are named as research-setting features, not generic context.
- Fiscal timing and outcome windows are aligned before causal or market-response claims are strengthened.
- Economics and accounting contribution claims are evaluated separately before they are combined.
- The verdict narrows overclaims before submission or final writing.
