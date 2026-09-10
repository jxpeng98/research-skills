# Quality Gate Report

## Gate Metadata

- task_id:
- solo_role:
- reviewer:
- date:

## Gate Status

```yaml
gates:
  Q1:
    status: BLOCKED
    evidence: []
    semantic_checks:
      - check_id: q1_rq_method_alignment
        status: BLOCKED
        finding: "No RQ-method-outcome evidence has been supplied yet."
        evidence_refs: []
    blocking_issues:
      - issue: "Research question, method, data source, outcome, and analysis strategy are not yet cross-referenced."
        required_action: "Add or update RESEARCH/[topic]/study_design.md with an RQ-method-outcome matrix."
  Q2:
    status: BLOCKED
    evidence: []
    semantic_checks:
      - check_id: q2_claim_evidence_traceability
        status: BLOCKED
        finding: "No claim-evidence ledger evidence has been supplied yet."
        evidence_refs: []
    blocking_issues:
      - issue: "Central manuscript claims are not yet mapped to evidence anchors."
        required_action: "Add or update RESEARCH/[topic]/evidence/claim-evidence-ledger.csv and manuscript claim map."
  Q3:
    status: BLOCKED
    evidence: []
    semantic_checks:
      - check_id: q3_reporting_completeness
        status: BLOCKED
        finding: "No reporting checklist evidence has been supplied yet."
        evidence_refs: []
    blocking_issues:
      - issue: "Required reporting checklist, disclosures, or waiver evidence is missing."
        required_action: "Add or update RESEARCH/[topic]/reporting_checklist.md and submission statements."
  Q4:
    status: BLOCKED
    evidence: []
    semantic_checks:
      - check_id: q4_reproducibility_baseline
        status: BLOCKED
        finding: "No reproducibility baseline evidence has been supplied yet."
        evidence_refs: []
    blocking_issues:
      - issue: "Data, code, environment, command, or rerun evidence is missing."
        required_action: "Add or update RESEARCH/[topic]/code/reproducibility_audit.md and relevant environment records."
```

## Structured Evidence Reference Examples

```yaml
evidence_ref_examples:
  Q1:
    - artifact: RESEARCH/[topic]/study_design.md
      anchor: rq-method-outcome-matrix
      supports: "Shows whether each RQ maps to data, measurement, method, outcome, and analysis strategy."
      claim_id: rq-method-outcome-coverage
      diagnostic_id: q1_rq_method_alignment
  Q2:
    - artifact: RESEARCH/[topic]/evidence/claim-evidence-ledger.csv
      anchor: central-claims
      supports: "Shows whether each central claim maps to source evidence, analysis output, citation anchor, or a gap note."
      claim_id: central-claims
      diagnostic_id: q2_claim_evidence_traceability
  Q3:
    - artifact: RESEARCH/[topic]/reporting_checklist.md
      anchor: required-items
      supports: "Shows whether required reporting items, disclosures, and waivers are complete for the paper type and venue."
      claim_id: reporting-completeness
      diagnostic_id: q3_reporting_completeness
  Q4:
    - artifact: RESEARCH/[topic]/code/reproducibility_audit.md
      anchor: rerun-baseline
      supports: "Shows whether inputs, code, environment, commands, outputs, and rerun limits are documented."
      claim_id: reproducibility-baseline
      diagnostic_id: q4_reproducibility_baseline
```

## Passed Gates

- None.

## Failed Gates

- None.

## Blocked Verification

- None.

## Next Actions

- None.
