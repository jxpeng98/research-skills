---
description: 基于已筛选/提取/质评结果执行证据综合（叙述/定性/定量 Meta-analysis）
---

# Evidence Synthesis / Meta-analysis

Synthesize included studies into PRISMA-ready results using narrative, qualitative, and/or quantitative meta-analysis methods.

Canonical Task IDs (from the globally installed `research-paper-workflow` skill):
- `E1` synthesis strategy
- `E2` effect size table
- `E3` meta-analysis results
- `E4` certainty grading
- `E5` integrated synthesis

## Target

$ARGUMENTS

## Workflow

### Step 0: Select Project Folder

Ask the user:
> "Which `RESEARCH/[topic]/` folder should be synthesized?"
> - Existing projects: [List folders under `RESEARCH/`]
> - Create new: `RESEARCH/[new-topic]/`

Set `[topic]` accordingly.

### Step 1: Prerequisite Check

Verify these exist (create if missing, or stop and ask to run upstream workflow):
- `RESEARCH/[topic]/extraction_table.md`
- `RESEARCH/[topic]/quality_table.md`
- `RESEARCH/[topic]/notes/` (non-empty recommended)

Recommended (create if missing):
- `RESEARCH/[topic]/synthesis_matrix.md`

### Step 2: Clarify the Synthesis Request

Ask the user:
1. Which outcomes/themes should be synthesized? (all vs subset)
2. Do you want quantitative pooling (meta-analysis) where feasible? (Y/N)
3. Any required subgroups? (population/setting/design/timepoint)
4. Preferred effect measures (OR/RR/SMD/MD/Fisher’s z) and “positive direction”

### Step 3: Execute Evidence Synthesis

Use the **evidence-synthesizer** skill to:
1. Decide synthesis type per outcome (meta-analysis vs narrative vs qualitative) and justify
2. Update synthesis matrix → `RESEARCH/[topic]/synthesis_matrix.md`
3. If meta-analysis is performed:
   - Draft plan → `RESEARCH/[topic]/meta_analysis_plan.md` (use **templates/meta-analysis-plan.md**)
   - Build effect size table → `RESEARCH/[topic]/effect_size_table.md` (use **templates/effect-size-extraction-table.md**)
   - Generate results → `RESEARCH/[topic]/meta_analysis_results.md` (use **templates/meta-analysis-report.md**)
   - (Optional) Save code + plots under `RESEARCH/[topic]/analysis/` using `templates/code/statistics/`
4. Write integrated synthesis → `RESEARCH/[topic]/synthesis.md`

### Step 4: Reporting Checklist (Fast)

Confirm `synthesis.md` covers:
- What was synthesized (which studies per outcome)
- Why pooling was or wasn’t done (rationale)
- Heterogeneity exploration (if applicable)
- Sensitivity analyses (if applicable)
- Missing-results bias assessment (if applicable)
- Certainty/confidence (optional: GRADE SoF)

Begin evidence synthesis now.
