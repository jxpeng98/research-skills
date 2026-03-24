---
id: manuscript-architect
stage: F_writing
version: "0.2.2"
description: "Build coherent paper structure from outline to section-level drafts with claim-evidence integrity and analysis-depth checking."
inputs:
  - type: RQSet
    description: "Research questions"
  - type: EvidenceTable
    description: "Synthesized evidence"
  - type: DesignSpec
    description: "Study design (for empirical or qualitative papers)"
    required: false
outputs:
  - type: ManuscriptOutline
    artifact: "manuscript/outline.md"
  - type: Manuscript
    artifact: "manuscript/manuscript.md"
  - type: ClaimGraph
    artifact: "manuscript/claims_evidence_map.md"
  - type: FiguresTablesPlan
    artifact: "manuscript/figures_tables_plan.md"
constraints:
  - "Must maintain claim-evidence alignment across sections"
  - "Must follow venue-specific sectioning requirements"
failure_modes:
  - "Evidence gaps discovered during writing"
  - "Scope creep from unfocused claims"
tools: [filesystem]
tags: [writing, manuscript, outline, drafting, claim-evidence]
domain_aware: true
---

# Manuscript Architect Skill

Build and revise a full research paper manuscript across stages: outline → section drafts → coherence pass → compliance checks → submission-ready package.

## When to Use

- You want to draft a paper from an existing `RESEARCH/[topic]/` project folder (empirical study or systematic review).
- You want to draft a fully qualitative paper from interview, case, ethnographic, or document-based materials without defaulting back to a quantitative structure.
- You have partial materials (notes, tables, analysis plan, results) and want a coherent manuscript.

## Granularity Boundary

Keep these as embedded writing subflows inside `manuscript-architect` unless they own a separate contract artifact:

- story spine construction
- section-level drafting
- paragraph ordering and transitions
- results anchoring against tables/figures
- section-to-section coherence passes

If the need is only "write a better intro / abstract / discussion paragraph", update this skill or its templates before introducing another top-level writing skill.

## Analytical Depth Contract

These depth rules apply across empirical, qualitative, mixed-methods, theory, and methods papers. Do not collapse the discussion into generic business-style implications when the domain is elsewhere.

- Every substantive paragraph must do more than describe; it should advance at least two of:
  - mechanism or process
  - tension, comparison, or contradiction
  - alternative explanation
  - boundary condition
  - implication
- Qualitative writing must interpret themes into analytic claims about mechanism, meaning, or scope. Do not stop at theme labels or illustrative quotes.
- If the evidence cannot support a deeper inference, narrow the claim instead of simulating depth with abstract wording.

## Inputs (Ask / Collect)

1. Paper type: **empirical** / **qualitative** / **systematic review** / **theory** / **methods** (default: empirical)
2. Target venue (optional): scope + formatting + double-blind requirements
3. Current artifacts available in `RESEARCH/[topic]/`:
   - Empirical: `study_design.md`, `analysis_plan.md`, results tables/figures, code outputs
   - Qualitative: `study_design.md`, `analysis_plan.md`, interview/case notes, coding memos, evidence tables, process diagrams, transparency appendix notes
   - SR: `synthesis.md`, `meta_analysis_results.md`, `prisma_checklist.md`
4. Citation style (APA/IEEE/BibTeX) and reference source (`bibliography.bib` preferred)

## Process

### Step 0: Set Manuscript Folder

Create/ensure:
```
RESEARCH/[topic]/manuscript/
├── outline.md
├── manuscript.md
├── claims_evidence_map.md
└── figures_tables_plan.md
```

Primary template references:
- `templates/manuscript-outline.md`
- `templates/manuscript-skeleton.md`
- `templates/claim-evidence-map.md`
- `templates/figures-tables-plan.md`

### Step 1: Define Contribution & Story

Produce:
- 1–2 sentence **core contribution**
- 3–5 bullet **novelty/importance** points
- Main **thesis** (what the reader should believe after reading)
- 1 short note on **why the gap persisted until now**

### Step 2: Build Outline (Paper-first, not section-first)

Use `templates/manuscript-outline.md` to:
- Fix section headings and subheadings
- Allocate word budget (rough)
- Attach the key evidence to each planned paragraph

### Step 3: Draft Sections Iteratively

Draft in order:
1. Introduction (problem → gap → why the gap persisted → contribution → roadmap)
2. Related work / literature positioning (theme-, tension-, or mechanism-based; not serial paper summary)
3. Methods (design, site/case access, sampling, data sources, measures/protocols, analytic procedure, reflexivity, ethics)
4. Results / Findings (tables/figures, themes, dimensions, process model, heterogeneity, null findings, negative cases; avoid interpretation creep and quote dumping)
5. Discussion (mechanisms, rival interpretations, boundary conditions, comparison to prior work, theoretical contribution, practical implications)
6. Limitations / threats to validity (explicit, not hidden)
7. Conclusion
8. Title/Abstract/Keywords (last)

Use **citation-formatter** to keep citations consistent.

For each drafted section, ask:
- What is the analytic job of this section?
- What does the reader learn beyond the surface description?
- Which alternative explanation or counter-pattern needs to be confronted here?
- Where should the claim be narrowed?

### Step 4: Claim–Evidence Integrity Pass

Use `templates/claim-evidence-map.md`:
- Every non-trivial claim must have: evidence + location + citation (or be marked as speculation)
- Remove or hedge claims that out-run evidence
- Tag mechanism statements that are evidenced vs. hypothesized
- Ensure RQs ↔ Methods ↔ Results alignment

### Step 5: Figures/Tables Pass

Use `templates/figures-tables-plan.md`:
- Ensure every figure/table has a purpose and is referenced in text
- Ensure captions are standalone and reproducible

### Step 6: Compliance / Readiness Pass

Run the right checker:
- Empirical: **reporting-checker** (CONSORT/STROBE/COREQ/etc.)
- Qualitative: **reporting-checker** (SRQR / COREQ / venue-specific transparency)
- Systematic review: **prisma-checker**

Optionally draft:
- Data/code availability statement
- Ethics statement (if human data)

### Step 7: Outputs

- Outline → `RESEARCH/[topic]/manuscript/outline.md`
- Draft manuscript → `RESEARCH/[topic]/manuscript/manuscript.md` (use `templates/manuscript-skeleton.md`)
- Claim-evidence map → `RESEARCH/[topic]/manuscript/claims_evidence_map.md`
- Figures/tables plan → `RESEARCH/[topic]/manuscript/figures_tables_plan.md`

Companion artifacts that should be referenced, not re-modeled as standalone writing skills:
- `manuscript/results_interpretation.md`
- `manuscript/effect_interpretation.md`
- `manuscript/tables/`
- `manuscript/figures/`
