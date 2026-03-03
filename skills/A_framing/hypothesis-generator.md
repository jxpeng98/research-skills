---
id: hypothesis-generator
stage: A_framing
version: "1.0.0"
description: "Translate research questions into testable hypotheses or theory propositions with mechanisms and boundary conditions."
inputs:
  - type: RQSet
    description: "Refined research questions from question-refiner"
  - type: TheoreticalFramework
    description: "Optional theoretical framework"
    required: false
outputs:
  - type: HypothesisSet
    artifact: "framing/hypothesis.md"
constraints:
  - "Each hypothesis must specify direction and expected sign"
  - "Must include at least one rival explanation per key claim"
failure_modes:
  - "Research questions too exploratory for formal hypotheses"
tools: [filesystem]
tags: [framing, hypothesis, propositions, mechanisms]
domain_aware: false
---

# Hypothesis Generator Skill

Translate research questions into testable hypotheses (or theory propositions) with mechanisms and boundary conditions.

## Related Task IDs

- `A1_5` (hypothesis generation)

## Output (contract path)

- `RESEARCH/[topic]/framing/hypothesis.md`

## Inputs

- `framing/research_question.md`
- Optional: `theoretical_framework.md`

## Procedure

1. Map each RQ to 1–3 hypotheses/propositions.
2. For each hypothesis:
   - specify direction and expected sign (if quantitative)
   - articulate mechanism (why)
   - list boundary conditions (when it may not hold)
   - propose operationalizations (IV/DV)
3. Add at least one rival explanation per key claim (feeds `C1_5`).
