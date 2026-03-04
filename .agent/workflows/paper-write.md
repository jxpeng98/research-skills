---
description: 从 RESEARCH 项目资料生成完整论文草稿（outline → manuscript → claim-evidence → figures/tables → readiness）
argument-hint: [RESEARCH下topic文件夹名] [可选: paper_type empirical|sr|theory] [可选: venue]
---

# Paper Writing (Full Draft)

Draft a complete research paper manuscript from existing project artifacts, covering outline, section drafts, and integrity/compliance passes.

Canonical Task IDs from `standards/research-workflow-contract.yaml`:
- `F1` outline only
- `F3` full draft
- `F4` claim-evidence map
- `F5` figures/tables plan

## Target

$ARGUMENTS

## Workflow

### Step 0: Select Project Folder

Ask the user:
> "Which `RESEARCH/[topic]/` folder should be turned into a manuscript?"
> - Existing projects: [List folders under `RESEARCH/`]
> - Create new: `RESEARCH/[new-topic]/`

Ensure:
```
RESEARCH/[topic]/manuscript/
├── outline.md
├── manuscript.md
├── claims_evidence_map.md
└── figures_tables_plan.md
```

### Step 1: Clarify Writing Constraints

Ask:
1. Paper type: empirical / systematic review / theory / methods
2. Target venue (optional) + word/page limits
3. Double-blind? (Y/N)
4. Citation style (APA/IEEE/BibTeX) and whether `bibliography.bib` is available

### Step 2: Gather Inputs from Project Artifacts

If empirical, prefer:
- `study_design.md`, `analysis_plan.md`, `data_management_plan.md`
- Results tables/figures or summary bullet points

If systematic review, prefer:
- `synthesis.md` (+ optional `meta_analysis_results.md`)
- PRISMA artifacts if available (`prisma_checklist.md`, `screening/`, etc.)

If some are missing, **STOP & CONFIRM** whether to:
- Draft with placeholders, or
- Run upstream workflows (`/study-design`, `/synthesize`, `/lit-review`)

### Step 3: Build Outline

Use **manuscript-architect** with `templates/manuscript-outline.md` to draft:
- `RESEARCH/[topic]/manuscript/outline.md`

**STOP & CONFIRM**:
> "Outline is ready. Proceed to full draft? (Y/N)"

### Step 4: Draft Manuscript

Use **manuscript-architect** with `templates/manuscript-skeleton.md` to produce:
- `RESEARCH/[topic]/manuscript/manuscript.md`

For single-section drafting, you can still use `/academic-write`.

### Step 5: Integrity Passes

Use **manuscript-architect** to create:
- Claim–evidence map → `RESEARCH/[topic]/manuscript/claims_evidence_map.md` (use `templates/claim-evidence-map.md`)
- Figures/tables plan → `RESEARCH/[topic]/manuscript/figures_tables_plan.md` (use `templates/figures-tables-plan.md`)

### Step 6: Readiness Checks

Run the appropriate checker:
- Empirical: **reporting-checker** → `RESEARCH/[topic]/reporting_checklist.md`
- SR: **prisma-checker** → `RESEARCH/[topic]/prisma_checklist.md`

Optionally proceed to submission packaging via `/submission-prep`.

Begin paper drafting now.
