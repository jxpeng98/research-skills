---
id: hypothesis-generator
stage: A_framing
description: "Translate research questions into hypotheses, propositions, or sensitizing concepts with mechanisms, rival explanations, and boundary conditions."
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
  - "Each confirmatory hypothesis must specify direction and expected sign when applicable"
  - "Must include at least one rival explanation or alternative interpretation per key claim"
failure_modes:
  - "Research questions are exploratory but no propositions or sensitizing concepts are produced"
tools: [filesystem]
tags: [framing, hypothesis, propositions, mechanisms]
domain_aware: false
---

# Hypothesis Generator Skill

Translate research questions into testable hypotheses, qualitative working propositions, or sensitizing concepts with mechanisms and boundary conditions.

## Related Task IDs

- `A1_5` (hypothesis generation)

## Output (contract path)

- `RESEARCH/[topic]/framing/hypothesis.md`

## Inputs

- `framing/research_question.md`
- Optional: `theoretical_framework.md`

## Procedure

1. Map each RQ to 1–3 hypotheses, propositions, or sensitizing concepts.
2. For each hypothesis:
   - specify direction and expected sign (if quantitative)
   - or specify the focal process, meaning, or mechanism to investigate (if qualitative)
   - articulate mechanism (why)
   - list boundary conditions (when it may not hold)
   - propose operationalizations (IV/DV) or evidence forms (cases/episodes/quotes/documents)
3. Add at least one rival explanation or rival interpretation per key claim (feeds `C1_5`).
