---
description: 从研究问题到可执行的研究设计（study_design + analysis_plan + instruments + DMP + prereg 草案）
argument-hint: [研究主题或RESEARCH下topic文件夹名]
---

# Study Design (Empirical)

Design an empirical study (quant/qual/mixed) and produce protocol-style artifacts.

Canonical Task IDs from `standards/research-workflow-contract.yaml`:
- `C1` study design
- `C2` instruments
- `C3` analysis plan
- `C4` data management plan
- `C5` preregistration draft

## Topic / Project

$ARGUMENTS

## Workflow

### Step 0: Select/Create Project Folder

Ask the user:
> "Which `RESEARCH/[topic]/` folder should this design live in?"
> - Existing projects: [List folders under `RESEARCH/`]
> - Create new: `RESEARCH/[new-topic]/`

Normalize `[topic]` (lowercase, hyphens).

Ensure structure exists:
```
RESEARCH/[topic]/
├── study_design.md
├── analysis_plan.md
├── data_management_plan.md
├── preregistration.md              # optional
├── ethics_irb.md                   # optional / depends on study
├── instruments/                    # optional
│   ├── survey.md
│   ├── interview_guide.md
│   ├── consent_form.md
│   └── recruitment_script.md
└── analysis/                        # optional (code + logs)
```

### Step 1: Clarify Research Question & Constraints

Use **question-refiner** to confirm:
1. RQ(s) + goal type (causal/descriptive/predictive)
2. Unit of analysis + setting
3. Constraints (data access, population access, timeline)
4. What claims are acceptable (strong causal vs association)

### Step 2: Produce Study Design + Analysis Plan

Use **study-designer** to draft:
- `RESEARCH/[topic]/study_design.md` (use `templates/study-design.md`)
- `RESEARCH/[topic]/analysis_plan.md` (use `templates/analysis-plan.md`)
- `RESEARCH/[topic]/data_management_plan.md` (use `templates/data-management-plan.md`)
- Optional `RESEARCH/[topic]/preregistration.md` (use `templates/preregistration-template.md`)
- Optional instruments under `RESEARCH/[topic]/instruments/`
  - Survey: `templates/survey-instrument.md`
  - Interview guide: `templates/interview-guide.md`

### Step 3: Ethics Gate (If Human Data)

**STOP & CONFIRM**:
> "Does this study involve human participants or sensitive data requiring IRB/ethics review? (Y/N)"

If yes, run `/ethics-check [topic]`.

Begin study design now.
