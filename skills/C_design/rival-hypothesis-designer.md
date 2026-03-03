---
id: rival-hypothesis-designer
stage: C_design
version: "1.0.0"
description: "Proactively construct and address competitive theories and rival explanations to strengthen study design."
inputs:
  - type: DesignSpec
    description: "Current study design"
  - type: HypothesisSet
    description: "Hypotheses to challenge"
outputs:
  - type: RivalHypotheses
    artifact: "design/rival_hypotheses.md"
constraints:
  - "Must generate at least one rival explanation per key claim"
  - "Must propose design features to rule out rivals"
failure_modes:
  - "Unable to articulate plausible rivals for purely exploratory work"
tools: [filesystem]
tags: [design, rival-hypotheses, threats-to-validity, competing-theories]
domain_aware: false
---

# Rival Hypothesis Designer Skill

Design against alternative explanations and reviewer objections by enumerating rivals and how to rule them out.

## Related Task IDs

- `C1_5` (rival hypothesis design)

## Output (contract path)

- `RESEARCH/[topic]/design/rival_hypotheses.md`

## Procedure

1. List plausible rivals:
   - omitted variables / confounding
   - reverse causality
   - measurement artifacts
   - selection effects / attrition
2. For each rival, define:
   - observable implication
   - planned measurement/control/test
