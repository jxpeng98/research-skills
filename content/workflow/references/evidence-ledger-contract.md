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

- Every central manuscript claim must have at least one ledger row. Keep one row
  per claim/source/location; multiple sources reuse the same claim ID, exact
  atomic claim text and claim type. An exact repeated row adds no new evidence.
- Supported claims must include `source_id`, `source_location`, and `artifact_path`.
- Unsupported claims must use `evidence_type=gap_note` and must not be converted into invented citations.
- Claim IDs must be stable across manuscript, submission, rebuttal, and presentation artifacts.
- Reuse those IDs and claim text in `manuscript/claims_evidence_map.md`; do not
  allocate a second ID simply because the artifact or stage changes. Conflicting
  meanings under one ID need an explicit repair, not a silent merge.
- For `evidence_type=paper`, use the established citekey as `source_id`. The graph
  joins that exact paper identity to literature and manuscript records; it does
  not resolve titles or DOI aliases. Keep other source types distinct.
- Use `supported` only after checking the source-to-claim match; uncertain support
  stays `needs_evidence`. A citation or a proposed model relation is not proof.
- The graph's support anchor binds claim, source, source location and artifact.
  Reordering CSV rows preserves that edge identity. Opening it resolves the
  matching record at the current bound revision; limitations remain in that row.
- Confidence labels should be `high`, `medium`, or `low`; explain limitations even when confidence is high.
