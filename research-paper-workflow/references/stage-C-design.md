# Stage C — Study Design & Analysis Plan (C1–C5)

This stage converts framing into an executable plan: design choices, measurement, estimands, and pre-specified analysis.

## Canonical outputs (contract paths)

- `C1` → `study_design.md`
- `C1_5` → `design/rival_hypotheses.md`
- `C2` → `instruments/`
- `C3` → `analysis_plan.md`
- `C3_5` → `design/robustness_plan.md`
- `C4` → `data_management_plan.md`
- `C5` → `preregistration.md`

## Quality gate focus

- `Q1` (question-to-method alignment) is enforced here: every RQ/hypothesis must map to data + model + outcome.
- `Q4` (reproducibility baseline): document data lineage, missingness, and analysis decisions.

---

## C1 — Study Design

**Definition of done**

- Study type justified (experiment / quasi / observational / qualitative / mixed)
- Unit of analysis + sampling frame is clear
- Constructs → measures → data collection procedure is specified
- Threats to validity addressed at design time (not only in “limitations”)

**Recommended minimum sections: `study_design.md`**

```markdown
# Study Design

## Research question alignment
| RQ/Hypothesis | Construct(s) | Data source | Outcome/metric |
|---|---|---|---|

## Design choice & rationale
- Design type:
- Identification logic (if causal):
- Why alternatives were rejected:

## Sample / data
- Population/frame:
- Inclusion/exclusion:
- Target N / saturation logic:

## Measures / operationalization
| Construct | Measure | Reliability/validity notes |
|---|---|---|

## Procedure
- Recruitment / collection:
- Timeline:

## Validity & risk
- Internal:
- Construct:
- External:
- Statistical conclusion:
```

---

## C1_5 — Rival Hypotheses / Alternative Explanations

Goal: reduce reviewer “endogeneity / confounding / alternative mechanism” objections proactively.

**Definition of done**
- 3–8 plausible rivals listed
- For each rival: how it would create the same pattern, and what you will measure/control to rule it out

Suggested table: `design/rival_hypotheses.md`

```markdown
| Rival | Mechanism | Observable implication | Design control / test |
|---|---|---|---|
```

---

## C2 — Instruments

Applies to surveys, interviews, coding schemes, rubrics, measurement protocols.

**Definition of done**
- Instrument exists and matches constructs
- Administration protocol (timing, prompts, consent links)
- Versioning plan (if iterative)

Write under: `instruments/` (e.g., `instruments/survey.md`, `instruments/interview_guide.md`).

---

## C3 — Analysis Plan

Specify *before* you see results: estimands, models, assumptions, missingness, and inference choices.

**Definition of done**
- Primary estimand(s) clearly defined
- Model family specified with covariates and functional forms
- Missing data strategy specified
- Multiple comparisons / researcher degrees of freedom addressed

Minimum structure: `analysis_plan.md`

```markdown
# Analysis Plan

## Estimands / targets
- Primary:
- Secondary:

## Variables
| Role | Variable | Measurement | Notes |
|---|---|---|---|

## Models
- Primary model:
- Assumptions:
- Diagnostics:

## Missing data
- Expected missingness:
- Handling:

## Inference
- Effect sizes + uncertainty reporting:
- Multiple testing:

## Robustness hooks (links to C3_5)
- ...
```

---

## C3_5 — Robustness / Sensitivity Plan

This is where you pre-specify checks reviewers will demand.

**Definition of done**
- A prioritized list of robustness checks linked to specific threats
- Clear pass/fail interpretation (what would change your conclusion)

Write into: `design/robustness_plan.md`.

---

## C4 — Data Management Plan (DMP)

Treat as a reproducibility + ethics artifact.

**Definition of done**
- Storage, access control, retention, and sharing plan
- De-identification linkage (D3) if human/sensitive data
- Code/data availability statement draft inputs (for D2/H1)

Write into: `data_management_plan.md`.

---

## C5 — Preregistration Draft (Optional but valuable)

Use when the venue/community values preregistration or when you want to lock in degrees of freedom.

**Definition of done**
- Hypotheses/RQs, design, exclusion rules, and analysis plan are frozen in a prereg doc
- Deviations policy described

Write into: `preregistration.md`.

