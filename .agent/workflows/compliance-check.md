---
description: 执行投稿前的学术依从性检查与风格审查（Reporting, PRISMA, Integrity, Tone）
---

# Compliance Check Workflow (G-Stage)

Run a thorough compliance and polish pass on a complete or near-complete manuscript. This ensures the study adheres to field-specific reporting guidelines, maintains logical consistency, and adopts the correct academic tone.

Canonical Task IDs from the `research-paper-workflow` skill:
- `G1` Reporting completeness
- `G2` PRISMA compliance
- `G3` Cross-section integrity
- `G4` Tone & Style Normalization

## Project Context

$ARGUMENTS

## Workflow Steps

### Step 0: Select Project Folder
Ask the user:
> "Which `RESEARCH/[topic]/` folder contains the manuscript we are checking?"

### Step 1: Select Checking Scope
Ask the user what kind of compliance checks they want to run (they can pick multiple):

1. **G1 Reporting Completeness Check**
   - Matches study type (e.g., RCT → CONSORT, Observational → STROBE, Qualitative → SRQR).
   - *Requires*: `manuscript/manuscript.md`
   - *Uses skill*: `reporting-checker`

2. **G2 PRISMA Compliance (Systematic Reviews Only)**
   - Item-by-item PRISMA 2020 verification.
   - *Requires*: `search_log.md`, `extraction_table.md`, `manuscript/manuscript.md`
   - *Uses skill*: `prisma-checker`

3. **G3 Cross-Section Integrity Check**
   - Verifies that RQs match Methods, and Methods match Results (claim-evidence mapping).
   - *Requires*: `manuscript/manuscript.md`
   - *Uses skill*: `manuscript-architect`

4. **G4 Tone & Style Normalization**
   - Strips subjective fluff, fixes over-claiming, and applies an objective academic voice.
   - *Requires*: `manuscript/manuscript.md`
   - *Uses skill*: `tone-normalizer`

### Step 2: Execution & Output
For each selected check, run the associated skill and output into the canonical paths:
- G1 → `RESEARCH/[topic]/reporting_checklist.md`
- G2 → `RESEARCH/[topic]/prisma_checklist.md`
- G3 → `RESEARCH/[topic]/manuscript/claims_evidence_map.md`
- G4 → `RESEARCH/[topic]/compliance/tone_normalization.md`

Begin execution by asking for the project topic and desired checks.
