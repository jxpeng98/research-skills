---
id: accounting-measurement-auditor
stage: C_design
description: "Audit accounting constructs, archival measurement choices, reporting settings, and sample filters."
inputs:
  - type: DesignSpec
    description: "Research question, setting, sample, and identification strategy"
  - type: DataDictionary
    description: "Variable definitions, source databases, filters, and transformations"
  - type: AnalysisPlan
    description: "Estimator, controls, fixed effects, clustering, and robustness checks"
outputs:
  - type: AccountingMeasurementAudit
    artifact: "analysis/accounting_measurement_audit.md"
constraints:
  - "Must distinguish the accounting construct from its empirical proxy"
  - "Must document sample filters, fiscal timing, and database item mappings"
  - "Must identify construct-validity threats before interpreting coefficients"
failure_modes:
  - "Accounting proxy does not map to the claimed construct"
  - "Sample construction removes economically important observations"
  - "Fiscal-year timing or restatement treatment undermines the design"
tools: [filesystem]
tags: [accounting, measurement, archival, reporting, construct-validity]
domain_aware: true
---

# Accounting Measurement Auditor Skill

Audit whether an accounting study's variables and sample construction can support the intended reporting, disclosure, governance, audit, tax, or earnings-quality claim.

## Purpose

Prevent construct drift between theory, accounting institution, empirical proxy, and coefficient interpretation.

## When to Use

- Before finalizing accounting variable definitions.
- Before interpreting archival accounting regressions as evidence of reporting behavior.
- When reviewer risk centers on proxy validity, sample filters, fiscal timing, database coverage, or alternative constructs.

## Inputs

- `DesignSpec`: setting, institutional mechanism, claim type, sample, and comparison group.
- `DataDictionary`: source database, item codes, filters, transformations, winsorization, fiscal timing, and missingness handling.
- `AnalysisPlan`: estimator, fixed effects, controls, clustering, and robustness checks.
- `DomainProfile`: load `skills/domain-profiles/accounting.yaml` when available.

If an input is missing, write a blocked-check note under `RESEARCH/[topic]/context/gap_notes.md`.

## Process

1. Name the target accounting construct in one sentence.
2. Map each proxy to source database, item code, transformation, fiscal timing, and expected sign.
3. Audit sample construction for filter-driven selection, database coverage, mergers, restatements, delistings, and fiscal-year alignment.
4. Check whether fixed effects, controls, and clustering match the reporting setting and identifying variation.
5. Identify alternative accounting constructs that could explain the same coefficient.
6. Produce a pass / revise / blocked verdict.

## Output Contract

Write `RESEARCH/[topic]/analysis/accounting_measurement_audit.md`:

```markdown
# Accounting Measurement Audit

## Construct Map
| Claim construct | Proxy | Source item | Transformation | Timing | Validity risk |
|---|---|---|---|---|---|

## Sample Construction
| Filter | Observations lost | Justification | Selection risk |
|---|---:|---|---|

## Reporting Setting Fit
| Design choice | Current plan | Risk | Required revision |
|---|---|---|---|

## Alternative Constructs
| Alternative explanation | Why plausible | Diagnostic or robustness check |
|---|---|---|

## Verdict
- Status:
- Required changes:
- Blocked checks:
```

## Quality Bar

- Every variable has a source, transformation, fiscal timing, and missingness rule.
- Every causal or mechanism claim distinguishes accounting measurement from causal identification and substantive interpretation.
- Every sample filter has a documented reason and selection-risk note.
