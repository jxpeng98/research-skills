# Coverage Matrix

Defines recommended **minimum Task ID coverage** by `paper_type` for a submission-ready workflow.

## Capability Matrix (at-a-glance)

| Capability | Empirical | Qualitative | Systematic Review | Methods | Theory |
|---|---|---|---|---|---|
| Framing (RQ + contribution) | Required | Required | Required | Required | Required |
| Theoretical framing / gap | Optional | Required | Optional | Optional | Required |
| Search + screening artifacts | Optional | Optional | Required | Optional | Optional |
| Extraction + quality appraisal | Optional | Optional | Required | Optional | Optional |
| Study design + analysis plan | Required | Required | Optional | Required | Optional |
| Data readiness (sampling + instruments + evidence plan) | Required | Required | Optional | Recommended | Optional |
| Ethics + governance | Required | Required | Optional | Optional | Optional |
| Synthesis / meta-analysis | Optional | Optional | Required | Optional | Optional |
| Manuscript drafting | Required | Required | Required | Required | Required |
| Compliance checks | Required | Required | Required | Required | Required |
| Submission + revision kit | Required | Required | Required | Required | Required |
| Proofread & de-AI | Required | Required | Required | Required | Required |
| Code/repro support | Recommended | Optional | Optional | Required | Optional |

## Recommended Minimum Task Coverage

### `empirical`

- Framing: `A1`, `A1_5`, `A2`, `A5`
- Literature: `B2`, `B6`, `B4` (related work)
- Design: `C1`, `C2` (if needed), `C3`, `C3_5`, `C4`
- Data: `I3`
- Ethics: `D1` and/or `D2` + `D3` (as applicable)
- Writing: `F1`, `F3`, `F4`, `F5`, `F6`
- Compliance: `G1`, `G3`, `G4`
- Proofread: `J1`, `J2` (required if AI-assisted); `J3`, `J4` (recommended)
- Submission/revision: `H1` (and `H3/H4` strongly recommended pre-submission)
- Code support: `I4` (recommended) + `I5–I8` (if code is a core contribution)

### `qualitative`

- Framing: `A1`, `A1_5` (hypotheses, propositions, or sensitizing concepts), `A2`, `A3`, `A4`, `A5`
- Literature: `B2`, `B6`, `B4` (related work)
- Design: `C1`, `C2`, `C3`, `C1_5`, `C4`
- Ethics: `D1` and/or `D2` + `D3` (as applicable for human/sensitive data)
- Writing: `F1`, `F3`, `F4`, `F5`, `F6`
- Compliance: `G1` (SRQR / COREQ), `G3`, `G4`
- Proofread: `J1`, `J2` (required if AI-assisted); `J3`, `J4` (recommended)
- Submission/revision: `H1` (and `H3/H4` strongly recommended pre-submission)
- Code support: `I4` optional when qualitative analysis relies on reproducible transcript processing or coding exports

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
- Data/code pipeline: `I3` (recommended when evaluation depends on reproducible data assembly)
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
- Qualitative interviews/focus groups: **COREQ**
- Broader qualitative studies: **SRQR**
- Prediction/diagnostic: **TRIPOD**

## Out of Scope by Default

- Field-specific legal requirements
- Institution-specific IRB portal submission
- Paywalled full-text retrieval guarantees
