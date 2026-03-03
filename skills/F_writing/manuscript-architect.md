# Manuscript Architect Skill

Build and revise a full research paper manuscript across stages: outline → section drafts → coherence pass → compliance checks → submission-ready package.

## When to Use

- You want to draft a paper from an existing `RESEARCH/[topic]/` project folder (empirical study or systematic review).
- You have partial materials (notes, tables, analysis plan, results) and want a coherent manuscript.

## Inputs (Ask / Collect)

1. Paper type: **empirical** / **systematic review** / **theory** / **methods** (default: empirical)
2. Target venue (optional): scope + formatting + double-blind requirements
3. Current artifacts available in `RESEARCH/[topic]/`:
   - Empirical: `study_design.md`, `analysis_plan.md`, results tables/figures, code outputs
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

### Step 1: Define Contribution & Story

Produce:
- 1–2 sentence **core contribution**
- 3–5 bullet **novelty/importance** points
- Main **thesis** (what the reader should believe after reading)

### Step 2: Build Outline (Paper-first, not section-first)

Use `templates/manuscript-outline.md` to:
- Fix section headings and subheadings
- Allocate word budget (rough)
- Attach the key evidence to each planned paragraph

### Step 3: Draft Sections Iteratively

Draft in order:
1. Introduction (problem → gap → contribution → roadmap)
2. Related work / literature positioning (theme-based)
3. Methods (design, data, measures, analysis plan; ethics statement if needed)
4. Results (tables/figures + uncertainty; avoid interpretation creep)
5. Discussion (mechanisms, comparison to prior work, implications)
6. Limitations / threats to validity (explicit, not hidden)
7. Conclusion
8. Title/Abstract/Keywords (last)

Use **citation-formatter** to keep citations consistent.

### Step 4: Claim–Evidence Integrity Pass

Use `templates/claim-evidence-map.md`:
- Every non-trivial claim must have: evidence + location + citation (or be marked as speculation)
- Remove or hedge claims that out-run evidence
- Ensure RQs ↔ Methods ↔ Results alignment

### Step 5: Figures/Tables Pass

Use `templates/figures-tables-plan.md`:
- Ensure every figure/table has a purpose and is referenced in text
- Ensure captions are standalone and reproducible

### Step 6: Compliance / Readiness Pass

Run the right checker:
- Empirical: **reporting-checker** (CONSORT/STROBE/COREQ/etc.)
- Systematic review: **prisma-checker**

Optionally draft:
- Data/code availability statement
- Ethics statement (if human data)

### Step 7: Outputs

- Outline → `RESEARCH/[topic]/manuscript/outline.md`
- Draft manuscript → `RESEARCH/[topic]/manuscript/manuscript.md` (use `templates/manuscript-skeleton.md`)
- Claim-evidence map → `RESEARCH/[topic]/manuscript/claims_evidence_map.md`
- Figures/tables plan → `RESEARCH/[topic]/manuscript/figures_tables_plan.md`

