# Evidence Ledger Contract

The evidence ledger is the canonical claim-to-support register for scholarly outputs.

## Canonical Paths

- Markdown overview: `RESEARCH/[topic]/evidence/evidence-ledger.md`
- CSV ledger: `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv`

## Required CSV Columns

`claim_id,claim_text,claim_type,evidence_type,source_id,source_location,artifact_path,confidence,limitations,status`

## Allowed Values

Allowed `claim_type`:

- `finding`
- `interpretation`
- `implication`
- `method_assumption`
- `limitation`
- `speculation`

Allowed `evidence_type`:

- `paper`
- `dataset`
- `analysis_result`
- `theory`
- `artifact`
- `gap_note`

## Rules

- Every central manuscript claim must have one ledger row.
- Supported claims must include `source_id`, `source_location`, and `artifact_path`.
- Unsupported claims must use `evidence_type=gap_note` and must not be converted into invented citations.
- Claim IDs must be stable across manuscript, submission, rebuttal, and presentation artifacts.
- Confidence labels should be `high`, `medium`, or `low`; explain limitations even when confidence is high.
