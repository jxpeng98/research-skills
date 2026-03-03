---
id: study-designer
stage: C_design
version: "1.0.0"
description: "Design empirical studies including sampling strategy, measures, procedures, and analysis plans with preregistration support."
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
  - "Must include power analysis"
  - "Must specify missingness handling"
failure_modes:
  - "Insufficient power for planned analysis"
  - "Design-method mismatch with research questions"
tools: [filesystem, metadata-registry]
tags: [design, sampling, measures, analysis-plan, preregistration]
domain_aware: true
---

# Study Designer Skill

Design an empirical research study (quant/qual/mixed) from a research question and constraints, producing a protocol-style design doc + analysis plan + instruments skeleton.

## When to Use

- After `/find-gap` and `/build-framework`, when you want to run an empirical study.
- When you have a topic + tentative RQs/hypotheses but no concrete methodology yet.

## Inputs (Ask / Collect)

1. **Research question(s)** and whether the goal is *causal*, *descriptive*, or *predictive*
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
- **Qualitative**: when mechanism/experience/meaning is primary (interviews/fieldwork)
- **Mixed methods**: when you need both measurement and explanation (sequencing must be explicit)

Document why this design is the best tradeoff given constraints.

### Step 2: Define Constructs, Variables, and Operationalization

- Map constructs to measurable variables (or qualitative codes)
- Specify IV/DV/mediators/moderators/controls (if quantitative)
- Prefer validated instruments/scales; pre-specify adaptations and translation steps

### Step 3: Sampling & Recruitment Plan

- Inclusion/exclusion criteria + recruitment channels
- Sample size strategy:
  - Quant: power/MDE rationale (even if approximate)
  - Qual: saturation plan + sampling logic (purposive/maximum variation)
- Anticipate attrition and missingness

### Step 4: Data Collection Plan

- Instruments: survey / interview guide / observation protocol / data extraction rules
- Pilot and refinement plan
- Data security and storage plan (align with `templates/data-management-plan.md`)

### Step 5: Analysis Plan (Pre-specify)

Create an analysis plan that is executable and auditable:
- Primary vs secondary outcomes
- Models/estimators + covariates + assumptions checks
- Multiple comparisons strategy (if many outcomes)
- Missing data handling
- Robustness/sensitivity analyses
- Subgroup analysis rules (avoid fishing; pre-specify)

### Step 6: Validity / Rigor Plan

- Quant: internal validity threats, measurement validity, common method bias, robustness
- Qual: credibility/transferability/dependability/confirmability; reflexivity plan

### Step 7: Preregistration (Optional but Recommended)

Draft preregistration content and decision rules. Keep deviations tracked with dates.

## Outputs (Create/Update)

- Study design doc → `RESEARCH/[topic]/study_design.md` (use `templates/study-design.md`)
- Analysis plan → `RESEARCH/[topic]/analysis_plan.md` (use `templates/analysis-plan.md`)
- Data management plan → `RESEARCH/[topic]/data_management_plan.md` (use `templates/data-management-plan.md`)
- Instruments (optional):
  - Survey → `RESEARCH/[topic]/instruments/survey.md` (use `templates/survey-instrument.md`)
  - Interview guide → `RESEARCH/[topic]/instruments/interview_guide.md` (use `templates/interview-guide.md`)
- Prereg draft (optional) → `RESEARCH/[topic]/preregistration.md` (use `templates/preregistration-template.md`)

## Notes

- This skill is not legal advice; for ethics/IRB packaging, use **ethics-irb-helper**.
- For implementation code, pair with **code-builder** (and **model-collaborator** when helpful).

