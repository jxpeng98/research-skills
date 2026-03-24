# Stage C — Study Design & Analysis Plan (C1–C5)

This stage converts framing into an executable plan: design choices, measurement, estimands, and pre-specified analysis.

## Canonical outputs (contract paths)

- `C1` → `study_design.md`
- `C1_5` → `design/rival_hypotheses.md`
- `C2` → `instruments/`
- `C3` → `analysis_plan.md`, `design/variable_spec.md`
- `C3_5` → `design/robustness_plan.md`
- `C4` → `data_management_plan.md`, `design/dataset_plan.md`
- `C5` → `preregistration.md`

## Quality gate focus

- `Q1` (question-to-method alignment) is enforced here: every RQ/hypothesis must map to data + model + outcome, or to setting + evidence source + analytic lens for qualitative work.
- `Q4` (reproducibility baseline): document data lineage, missingness, and analysis decisions.

---

## C1 — Study Design

**Definition of done**

- Study type justified (experiment / quasi / observational / qualitative / mixed)
- Unit of analysis + sampling frame is clear
- Constructs / sensitizing concepts → measures / evidence sources → data collection procedure is specified
- Case boundaries, setting, and analytic strategy are explicit for qualitative work
- Threats to validity addressed at design time (not only in “limitations”)

**Recommended minimum sections: `study_design.md`**

```markdown
# Study Design

## Research question alignment
| RQ/Hypothesis | Construct(s) | Data source | Outcome/metric or evidence form |
|---|---|---|---|

## Design choice & rationale
- Design type:
- Identification logic (if causal):
- Analytic tradition / qualitative strategy (if qualitative):
- Why alternatives were rejected:

## Sample / data
- Population/frame:
- Inclusion/exclusion:
- Target N / saturation / case logic:

## Measures / operationalization
| Construct / sensitizing concept | Measure / protocol / evidence source | Reliability/validity or trustworthiness notes |
|---|---|---|

## Procedure
- Recruitment / collection:
- Timeline:

## Validity & risk
- Internal:
- Construct:
- External:
- Statistical conclusion:
- Credibility / transferability / dependability / confirmability (if qualitative):
```

For fully qualitative studies, `C1` should also lock:
- case selection rationale and setting boundaries
- interview / observation / document collection rules
- within-case vs cross-case logic
- reflexivity and memoing plan
- disconfirming-case / rival-interpretation plan

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
- Instrument exists and matches constructs or sensitizing concepts
- Administration protocol (timing, prompts, consent links)
- Versioning plan (if iterative)

Write under: `instruments/` (e.g., `instruments/survey.md`, `instruments/interview_guide.md`).

---

## C3 — Analysis Plan

Specify *before* results stabilize: estimands or analytic targets, models or coding logic, assumptions, missingness/evidence gaps, and inference choices.

**Definition of done**
- Primary estimand(s) or analytic targets clearly defined
- Model family or qualitative analytic procedure specified
- Missing data or evidence-gap strategy specified
- Multiple comparisons / researcher degrees of freedom / rival-interpretation degrees of freedom addressed
- Variable roles, coding, and transformation rules are frozen in `design/variable_spec.md`

Minimum structure: `analysis_plan.md`

```markdown
# Analysis Plan

## Estimands / analytic targets
- Primary:
- Secondary:
- Focal process / meaning / mechanism claims (if qualitative):

## Variables / coding frame
| Role / code family | Variable / code | Measurement / evidence source | Notes |
|---|---|---|---|

## Models / analytic procedures
- Primary model (if quantitative):
- Qualitative analytic procedure (if qualitative):
- Assumptions:
- Diagnostics / trustworthiness checks:

## Missing data / evidence gaps
- Expected missingness or thin spots:
- Handling:

## Inference / interpretation rules
- Effect sizes + uncertainty reporting:
- Quote / episode / case-selection rules for write-up (if qualitative):
- Multiple testing / rival interpretation control:

## Robustness hooks (links to C3_5)
- ...
```

Companion artifact: `design/variable_spec.md`

```markdown
# Variable / Code Specification

| Role | Variable / code | Source | Unit / coding / evidence form | Transformation | Notes |
|---|---|---|---|---|---|
```

---

## C3_5 — Robustness / Sensitivity Plan

This is where you pre-specify checks reviewers will demand.
For qualitative work, this includes rival interpretations, deviant cases, alternate coding lenses, and source triangulation.

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
- Dataset feasibility, provenance, and access constraints are captured in `design/dataset_plan.md`
- Transcript/audio/document handling rules are explicit when qualitative evidence is collected

Write into:
- `data_management_plan.md`
- `design/dataset_plan.md`

Recommended minimal structure for `design/dataset_plan.md`

```markdown
# Dataset Plan

| Dataset | Coverage | Access | Key variables | Risks |
|---|---|---|---|---|
```

---

## C5 — Preregistration Draft (Optional but valuable)

Use when the venue/community values preregistration or when you want to lock in degrees of freedom.
For qualitative work, a protocol lock-in document is still valuable even if formal preregistration is uncommon.

**Definition of done**
- Hypotheses/RQs, design, exclusion rules, and analysis plan are frozen in a prereg doc
- Deviations policy described

Write into: `preregistration.md`.
