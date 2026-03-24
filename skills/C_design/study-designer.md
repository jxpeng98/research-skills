---
id: study-designer
stage: C_design
version: "0.1.0"
description: "Design empirical, qualitative, or mixed-methods studies including sampling strategy, instruments, procedures, analysis plans, and protocol lock-in."
inputs:
  - type: RQSet
    description: "Research questions and hypotheses"
  - type: TheoreticalFramework
    description: "Theoretical framework for operationalization"
    required: false
outputs:
  - type: DesignSpec
    artifact: "study_design.md"
  - type: AnalysisPlan
    artifact: "analysis_plan.md"
  - type: DataManagementPlan
    artifact: "data_management_plan.md"
  - type: Instruments
    artifact: "instruments/"
  - type: Preregistration
    artifact: "preregistration.md"
constraints:
  - "Must justify design choice against alternatives"
  - "Must specify design-appropriate sampling adequacy logic (power/MDE for quant; saturation/theoretical sampling for qual)"
  - "Must specify missingness handling or evidence-gap handling appropriate to the design"
failure_modes:
  - "Sampling adequacy logic is missing or mismatched to the design"
  - "Design-method mismatch with research questions"
tools: [filesystem, metadata-registry]
tags: [design, sampling, measures, analysis-plan, preregistration]
domain_aware: true
---

# Study Designer Skill

Design an empirical, qualitative, or mixed-methods study from a research question and constraints, producing a protocol-style design doc, analysis plan, and instruments skeleton.

## When to Use

- After `/find-gap` and `/build-framework`, when you want to run an empirical, qualitative, or mixed-methods study.
- When you have a topic + tentative RQs/hypotheses but no concrete methodology yet.
- When you want to write a fully qualitative paper and need the design, coding logic, and evidence plan to be explicit before drafting.

## Granularity Boundary

`study-designer` owns the design package as a whole:
- design choice rationale
- construct-to-measure mapping
- sampling and procedure logic
- analysis summary handoff
- governance and preregistration pointers

Do not split these into separate top-level skills unless a new artifact path and direct pipeline dependency are required. If the change is only about output structure, update `templates/study-design.md` or related templates instead.

## Inputs (Ask / Collect)

1. **Research question(s)** and whether the goal is *causal*, *descriptive*, *predictive*, *explanatory*, or *interpretive*
2. **Domain constraints**: data access, population access, timeline, budget, tools
3. **Unit of analysis**: individuals / teams / orgs / posts / countries / etc.
4. **Ethics constraints**: minors, sensitive data, protected classes, high-risk contexts
5. **Target venue** (optional): affects reporting norms and word limits

## Process

### Step 1: Choose Study Type (Fit-to-Question)

Pick the simplest design that can answer the RQ credibly:
- **Experiment/RCT**: when randomization feasible and causal claims are central
- **Quasi-experimental**: when causal claims matter but randomization isn’t feasible (e.g., DID/RD/IV)
- **Observational (cross-sectional/longitudinal)**: when you need associations, prevalence, trajectories
- **Qualitative**: when process, mechanism, experience, or meaning is primary (interviews, case study, ethnography, process research)
- **Mixed methods**: when you need both measurement and explanation (sequencing must be explicit)

Document why this design is the best tradeoff given constraints.

### Step 2: Define Constructs, Variables, and Operationalization

- Map constructs to measurable variables, sensitizing concepts, or qualitative code families
- Specify IV/DV/mediators/moderators/controls (if quantitative)
- Define case boundaries, focal episodes, actors, and evidence forms (if qualitative)
- Prefer validated instruments/scales; pre-specify adaptations and translation steps

### Step 3: Sampling & Recruitment Plan

- Inclusion/exclusion criteria + recruitment channels
- Sample size strategy:
  - Quant: power/MDE rationale (even if approximate)
  - Qual: saturation plan + sampling logic (purposive / theoretical / maximum variation / critical-case)
- Anticipate attrition, missingness, and inaccessible or low-quality qualitative evidence

### Step 4: Data Collection Plan

- Instruments: survey / interview guide / observation protocol / fieldnote protocol / document extraction rules
- Pilot and refinement plan
- Access plan, recording/transcription rules, and memoing expectations for qualitative work
- Data security and storage plan (align with `templates/data-management-plan.md`)

### Step 5: Analysis Plan (Pre-specify)

Create an analysis plan that is executable and auditable:
- Primary vs secondary outcomes
- Models/estimators + covariates + assumptions checks
- Multiple comparisons strategy (if many outcomes)
- Missing data handling
- Robustness/sensitivity analyses
- Subgroup analysis rules (avoid fishing; pre-specify)
- Qualitative analytic procedure where applicable:
  - coding strategy (inductive / deductive / hybrid)
  - codebook evolution rules
  - memoing and reflexivity plan
  - within-case and cross-case comparison logic
  - negative-case / rival-interpretation handling
  - stopping rule for saturation or theoretical sufficiency

### Step 6: Validity / Rigor Plan

- Quant: internal validity threats, measurement validity, common method bias, robustness
- Qual: credibility, transferability, dependability, confirmability, reflexivity, triangulation, and chain-of-evidence plan

### Step 7: Preregistration (Optional but Recommended)

Draft preregistration or protocol lock-in content and decision rules. Keep deviations tracked with dates.

## Outputs (Create/Update)

- Study design doc → `RESEARCH/[topic]/study_design.md` (use `templates/study-design.md`)
- Analysis plan → `RESEARCH/[topic]/analysis_plan.md` (use `templates/analysis-plan.md`)
- Data management plan → `RESEARCH/[topic]/data_management_plan.md` (use `templates/data-management-plan.md`)
- Instruments (optional):
  - Survey → `RESEARCH/[topic]/instruments/survey.md` (use `templates/survey-instrument.md`)
  - Interview guide → `RESEARCH/[topic]/instruments/interview_guide.md` (use `templates/interview-guide.md`)
- Prereg draft (optional) → `RESEARCH/[topic]/preregistration.md` (use `templates/preregistration-template.md`)

Design package handoff expectations:
- `study_design.md` owns the rationale and protocol spine
- `analysis_plan.md` owns estimator and inference detail
- `data_management_plan.md` owns governance and retention detail
- `instruments/` owns collection instruments
- `preregistration.md` owns prereg wording

## Notes

- This skill is not legal advice; for ethics/IRB packaging, use **ethics-irb-helper**.
- For implementation code, pair with **code-builder** (and **model-collaborator** when helpful).
- For downstream variable or dataset specifics, pair with **variable-constructor** and **dataset-finder** rather than splitting the design rationale itself.
