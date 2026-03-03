---
id: prereg-writer
stage: C_design
version: "1.0.0"
description: "Generate preregistration documents for OSF/AsPredicted/ClinicalTrials.gov from study design and analysis plan."
inputs:
  - type: DesignSpec
    description: "Complete study design"
  - type: AnalysisPlan
    description: "Pre-specified analysis plan"
  - type: HypothesisSet
    description: "Testable hypotheses"
outputs:
  - type: Preregistration
    artifact: "preregistration.md"
constraints:
  - "Must include all hypotheses with directional predictions"
  - "Must specify primary and secondary outcomes before data collection"
  - "Must lock analysis plan with exact model specifications"
failure_modes:
  - "Exploratory research doesn't fit prereg template"
  - "Analysis plan too flexible to pre-specify fully"
tools: [filesystem]
tags: [design, preregistration, OSF, AsPredicted, transparency]
domain_aware: true
---

# Pre-registration Writer Skill

Generate structured preregistration documents following OSF or AsPredicted templates.

## Related Task IDs

- `C4` (preregistration)

## Output (contract path)

- `RESEARCH/[topic]/preregistration.md`

## Procedure

1. **Select template** based on study type:
   - OSF Prereg (general)
   - AsPredicted (simplified)
   - PROSPERO (systematic reviews)
   - ClinicalTrials.gov (clinical trials)
2. **Core sections**:
   - Study information (title, authors, date)
   - Hypotheses (numbered, directional)
   - Design plan (study type, blinding, randomization)
   - Sampling plan (N, power analysis, stopping rules)
   - Variables (IVs, DVs, covariates, with operationalization)
   - Analysis plan (exact model specifications, inference criteria)
   - Exploratory analyses (clearly separated from confirmatory)
3. **Lock specifications**:
   - Exact statistical models (formula notation)
   - Exclusion criteria (pre-data)
   - Multiple comparison corrections
   - Missing data handling
4. **Separation discipline**:
   - Confirmatory vs. exploratory clearly demarcated
   - Pre-data vs. post-data decisions flagged

## Minimal Output Format

```markdown
# Preregistration: [Study Title]

## 1. Hypotheses
H1: [directional prediction]

## 2. Design Plan
- Study type: ...
- Blinding: ...

## 3. Sampling Plan
- N = ... (power = .80, α = .05)

## 4. Variables
| Role | Variable | Operationalization |
|---|---|---|

## 5. Analysis Plan
- Primary model: `DV ~ IV + covariates`
- Inference: ...

## 6. Exploratory Analyses
- [clearly separated]
```
