---
id: limitation-auditor
stage: H_submission
description: "Identifies and structures study constraints, ensuring rigorous self-critique without undermining the manuscript's core contribution."
inputs:
  - type: DraftManuscript
    description: "The full manuscript draft including methods and discussion"
  - type: MethodologicalFramework
    description: "The core research design and constraints"
outputs:
  - type: LimitationSection
    artifact: "tools/limitations_audit.md"
  - type: MitigationStrategy
    artifact: "tools/limitation_mitigations.md"
constraints:
  - "Must map limitations directly to research design flaws or data constraints"
  - "Must include a mitigation or justification for each limitation"
failure_modes:
  - "Listing generic limitations (e.g., 'small sample size' without context)"
  - "Undermining the entire study rather than contextualizing its boundary conditions"
tools: [filesystem]
tags: [submission, limitations, rigorous, self-critique, boundary-conditions]
domain_aware: true
---

# Limitation Auditor Skill

Identifies and structures study constraints, ensuring rigorous self-critique without undermining the manuscript's core contribution.

## Purpose

To systematically review the research design, execution, and findings to identify valid limitations. It then frames these limitations constructively, explaining their impact on the conclusions and how future research can address them, establishing boundary conditions for the study's claims.

## When to Use

Use during the late drafting or pre-submission phase when the Methods and Discussion sections are complete, to ensure the manuscript demonstrates scholarly humility and self-awareness before peer reviewers point out the flaws.

## Expected Inputs

- `RESEARCH/[topic]/manuscript_fragments/03_methods.md`
- `RESEARCH/[topic]/manuscript_fragments/05_results.md`
- `RESEARCH/[topic]/manuscript_fragments/06_discussion.md`

## Process

### Step 1: Scan for Methodological Vulnerabilities
Analyze the Methods section for standard vulnerabilities:
- **Design:** Cross-sectional vs. longitudinal, observational vs. experimental
- **Data/Sample:** Selection bias, attrition, non-representativeness, sample size
- **Measurement:** Self-report bias, construct validity, recall bias
- **Analytical:** Unobserved confounding, endogeneity, assumptions of statistical models

### Step 2: Extract Specific Claims and Boundary Conditions
- Review the Discussion section.
- Identify the strongest claims made.
- Determine the boundary conditions: *Under what circumstances might these claims not hold true?* (e.g., specific populations, cultural contexts, time periods).

### Step 3: Frame Constructively (The "Flaw-Impact-Mitigation" Structure)
For each identified limitation, structure the audit:
1. **The Flaw:** Explicitly state the limitation.
2. **The Impact:** Explain how this limitation affects the certainty or generalizability of the results. Avoid catastrophic language; use precise, measured terms.
3. **The Mitigation/Justification:** Why is the paper still valuable despite this? Did the authors perform robustness checks? Is it an unavoidable constraint of the field?
4. **Future Directions:** Clearly specify how subsequent studies could overcome this constraint.

### Step 4: Output Generation
Create the `limitations_audit.md` mapping out the limitations, and provide a polished text block (`limitation_mitigations.md`) that can be copy-pasted directly into the manuscript's Discussion/Conclusion section.

## Quality Bar

- [ ] Limitations are specific to the study, not generic boilerplate
- [ ] Language is measured, avoiding fatalistic self-sabotage
- [ ] Every limitation is paired with a mitigation or boundary condition
- [ ] Future research directions are logically linked to the stated limitations

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Boilerplate Flaws | Claiming "sample size was small" without explaining the statistical power impact | Relate sample size explicitly to effect sizes or sub-group analyses |
| Self-Sabotage | Writing limitations that implicitly say the study is worthless | Focus on *boundary conditions*—where do the findings apply, and where do they stop? |
| Missing Mitigations | Pointing out a flaw but forgetting to defend the methodological choices made | Always include the *why* (e.g., "Although cross-sectional, this approach was necessary to...") |
