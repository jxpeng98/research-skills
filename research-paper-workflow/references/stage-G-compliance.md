# Stage G — Polishing & Compliance (G1–G4)

This stage is “submission hardening”: reporting completeness, PRISMA checks, cross-section consistency, and tone calibration.

## Canonical outputs (contract paths)

- `G1` → `reporting_checklist.md`
- `G2` → `prisma_checklist.md`
- `G3` → `manuscript/claims_evidence_map.md` (updated)
- `G4` → `compliance/tone_normalization.md`

## Quality gate focus

- `Q2` (claim-evidence traceability): enforced via `G3`.
- `Q3` (reporting completeness): enforced via `G1` and `G2`.

---

## G1 — Reporting Completeness (non-PRISMA)

**Definition of done**
- A guideline is selected and declared (by design type):
  - RCT → CONSORT
  - Observational → STROBE
  - Diagnostic prediction → TRIPOD
  - Qualitative → COREQ / SRQR
- Checklist items are mapped to manuscript locations
- Missing items are converted into concrete edit tasks

Write into: `reporting_checklist.md`.

---

## G2 — PRISMA Compliance (Systematic Review)

**Definition of done**
- PRISMA 2020 checklist is completed with manuscript locations
- Counts reconcile across `search_log.md`, `screening/`, `extraction_table.md`, and `synthesis.md`
- Deviations from protocol are documented (if any)

Write into: `prisma_checklist.md`.

---

## G3 — Cross-section Integrity Check

**Definition of done**
- Abstract claims appear in the claim–evidence map and are supported
- RQs ↔ Methods ↔ Results alignment holds (no “method drift”)
- Discussion claims are calibrated to evidence quality

Write/update: `manuscript/claims_evidence_map.md`.

---

## G4 — Tone & Style Normalization

Goal: remove reviewer triggers (overclaim, vagueness, hype).

**Definition of done**
- Overclaim language removed (“prove”, “guarantee”, “always”)
- Hedging calibrated to evidence (“suggest”, “is consistent with”)
- Paragraph-level topic sentences are explicit

Write into: `compliance/tone_normalization.md` with:
- a list of high-risk sentences
- suggested rewrites
- global style rules adopted (tense, voice, acronym policy)

