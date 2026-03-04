# Coverage Matrix

Defines recommended **minimum Task ID coverage** by `paper_type` for a submission-ready workflow.

## Capability Matrix (at-a-glance)

| Capability | Empirical | Systematic Review | Methods | Theory |
|---|---|---|---|---|
| Framing (RQ + contribution) | Required | Required | Required | Required |
| Theoretical framing / gap | Optional | Optional | Optional | Required |
| Search + screening artifacts | Optional | Required | Optional | Optional |
| Extraction + quality appraisal | Optional | Required | Optional | Optional |
| Study design + analysis plan | Required | Optional | Required | Optional |
| Ethics + governance | Required | Optional | Optional | Optional |
| Synthesis / meta-analysis | Optional | Required | Optional | Optional |
| Manuscript drafting | Required | Required | Required | Required |
| Compliance checks | Required | Required | Required | Required |
| Submission + revision kit | Required | Required | Required | Required |
| Proofread & de-AI | Required | Required | Required | Required |
| Code/repro support | Optional | Optional | Required | Optional |

## Recommended Minimum Task Coverage

### `empirical`

- Framing: `A1`, `A1_5`, `A2`, `A5`
- Literature: `B2`, `B6`, `B4` (related work)
- Design: `C1`, `C2` (if needed), `C3`, `C3_5`, `C4`
- Ethics: `D1` and/or `D2` + `D3` (as applicable)
- Writing: `F1`, `F3`, `F4`, `F5`, `F6`
- Compliance: `G1`, `G3`, `G4`
- Proofread: `J1`, `J2` (required if AI-assisted); `J3`, `J4` (recommended)
- Submission/revision: `H1` (and `H3/H4` strongly recommended pre-submission)
- Code support: `I4` (recommended) + `I5–I8` (if code is a core contribution)

### `systematic-review`

- Framing: `A1`, `A2`, `A5`
- Pipeline: `B1` (or decomposed into `B1_5`, `B2`, `B3` + screening/extraction/quality artifacts)
- Synthesis: `E1`, `E2` (if pooling), `E3` (if pooling), `E3_5`, `E4` (optional), `E5`
- Writing: `F1`, `F3`, `F4`, `F5`, `F6`
- Compliance: `G2` (PRISMA), plus `G3`, `G4`
- Proofread: `J1`, `J2` (required if AI-assisted); `J3`, `J4` (recommended)
- Submission/revision: `H1`, `H2` (post-review), `H3/H4` recommended pre-submission

### `methods`

- Framing: `A1`, `A1_5`, `A2`, `A3`, `A5`
- Literature: `B2`, `B6`, `B4`
- Design/evaluation: `C1`, `C3`, `C3_5`, `C4` (as applicable)
- Writing: `F1`, `F3`, `F4`, `F5`, `F6`
- Compliance: `G1`, `G3`, `G4`
- Proofread: `J1`, `J2` (required if AI-assisted); `J3`, `J4` (recommended)
- Code support: `I5`, `I6`, `I7`, `I8` (typically required); `I4` recommended
- Submission/revision: `H1`, `H3/H4` recommended pre-submission

### `theory`

- Framing: `A1`, `A2`, `A3`, `A4`, `A5`
- Literature: `B2`, `B6`, `B4`
- Design: `C*` optional (only if theory is supported by empirical/archival validation)
- Writing: `F1`, `F3`, `F4`, `F5`, `F6`
- Compliance: `G1`, `G3`, `G4`
- Proofread: `J1`, `J2` (required if AI-assisted); `J3`, `J4` (recommended)
- Submission/revision: `H1`, `H3/H4` recommended pre-submission

## Reporting Guideline Reminder

Select the right guideline in `G1/G2` (venue may require one explicitly):

- Systematic review: **PRISMA 2020** (`G2`)
- RCT: **CONSORT**
- Observational: **STROBE**
- Qualitative: **COREQ** / **SRQR**
- Prediction/diagnostic: **TRIPOD**

## Out of Scope by Default

- Field-specific legal requirements
- Institution-specific IRB portal submission
- Paywalled full-text retrieval guarantees
