---
id: publication-bias-checker
stage: E_synthesis
version: "0.1.0"
description: "Evaluate publication bias using funnel plots, fail-safe N, Egger's test, and trim-and-fill correction models."
inputs:
  - type: EvidenceTable
    description: "Synthesized evidence with effect sizes"
outputs:
  - type: PublicationBiasReport
    artifact: "synthesis/publication_bias.md"
constraints:
  - "Must use at least 2 complementary methods"
  - "Must report funnel plot asymmetry test"
failure_modes:
  - "Too few studies (<10) for reliable funnel plot"
  - "Effect size heterogeneity confounds bias assessment"
tools: [filesystem, stats-engine]
tags: [synthesis, publication-bias, funnel-plot, fail-safe-N, trim-and-fill]
domain_aware: false
---

# Publication Bias Checker Skill

Assess missing-results bias / publication bias for meta-analysis where feasible and document limitations.

## Related Task IDs

- `E3_5` (publication-bias-check)

## Output (contract path)

- `RESEARCH/[topic]/synthesis/publication_bias.md`

## Procedure

1. Check feasibility (k too small → bias tests low power; document this).
2. Run/report appropriate checks (depending on data availability):
   - funnel plot interpretation notes
   - Egger/Begg tests (if applicable)
   - trim-and-fill / selection model discussion (if applicable)
3. Calibrate claims: bias checks are suggestive, not definitive.
