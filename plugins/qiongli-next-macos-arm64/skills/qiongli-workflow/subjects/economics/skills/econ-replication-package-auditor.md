---
id: econ-replication-package-auditor
stage: I_code
description: "Audit economics replication packages, reproducibility materials, data access, and disclosure risks."
inputs:
  - type: Manuscript
    description: "Draft manuscript with reported estimates, tables, figures, and disclosure statements"
  - type: AnalysisCode
    description: "Analysis scripts, execution order, environment files, seeds, and generated outputs"
  - type: DataDictionary
    description: "Variable definitions, sample construction rules, data sources, and access constraints"
outputs:
  - type: ReplicationAudit
    artifact: "analysis/replication_audit.md"
constraints:
  - "Must map every reported empirical result to code, data, and sample definitions"
  - "Must separate restricted, licensed, generated, and shareable data materials"
  - "Must flag missing disclosure, environment, seed, and execution-order evidence"
failure_modes:
  - "Reported estimates cannot be traced to executable scripts or table sources"
  - "Treatment, outcomes, controls, fixed effects, clusters, or weights diverge from the manuscript"
  - "Restricted or licensed data are bundled without access constraints or disclosure"
tools: [filesystem]
tags: [economics, reproducibility, replication, data-code, disclosure]
domain_aware: true
---

# Economics Replication Package Auditor Skill

Audit the economics replication package before submission or release.

## Purpose

Check whether an economics manuscript's empirical results can be traced from manuscript claims to scripts, data, tables, figures, and disclosure materials.

## When to Use

- Before submitting an economics paper with empirical results and code.
- Before depositing a replication package, reproducibility archive, or journal data appendix.
- When reviewer, editor, or journal policy risk centers on data access, undisclosed degrees of freedom, or irreproducible tables.

## Inputs

- `Manuscript`: reported estimates, results tables, figures, sample definitions, data availability statement, and disclosure language.
- `AnalysisCode`: scripts, notebooks, environment files, random seeds, execution order, generated outputs, and table-building code.
- `DataDictionary`: variable definitions, source files, data licenses, access restrictions, and generated-data documentation.
- `DomainProfile`: load `skills/domain-profiles/economics.yaml` when available.

If any input is missing, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and list the blocked replication checks.

## Process

1. Match every reported estimate to a script, table source, and sample definition.
2. Confirm that treatment, outcome, controls, fixed effects, clusters, and weights match the manuscript.
3. Check that restricted data, licensed data, generated data, and shareable data are separated in the package.
4. Verify that random seeds, environment files, and execution order are documented.
5. Map tables and figures to scripts and generated outputs.
6. Flag undisclosed researcher degrees of freedom and robustness checks that cannot be reproduced.
7. Produce a pass / revise / blocked verdict.

## Output Contract

Write `RESEARCH/[topic]/analysis/replication_audit.md`:

```markdown
# Replication Audit

## Reproducibility Status
- Decision: pass / revise / blocked
- One-command rerun available: yes / no / partial

## Missing Files
- Scripts:
- Data:
- Environment:
- Documentation:

## Script-to-Table Map
| Manuscript result | Script or notebook | Data source | Sample definition | Status |
|---|---|---|---|---|

## Specification Match
| Result | Treatment | Outcome | Controls | Fixed effects | Clusters | Weights | Risk |
|---|---|---|---|---|---|---|---|

## Data Access Constraints
| Data source | Access type | Package handling | Required disclosure |
|---|---|---|---|

## High-Risk Discrepancies
- Discrepancy:
- Affected result:
- Required fix:

## Required Fixes Before Submission
- Fix:
- Owner:
- Evidence needed:
```

## Quality Bar

- [ ] Every reported estimate maps to a script, table source, and sample definition.
- [ ] Treatment, outcome, controls, fixed effects, clusters, and weights match the manuscript.
- [ ] Restricted, licensed, generated, and shareable data materials are clearly separated.
- [ ] Random seeds, environment files, and execution order are documented.
- [ ] The audit flags unreproducible robustness checks and undisclosed researcher degrees of freedom.
