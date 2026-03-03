---
id: literature-mapper
stage: B_literature
version: "1.0.0"
description: "Build a taxonomy or map of the literature to support defensible positioning and non-chronological related work writing."
inputs:
  - type: ExtractionTable
    description: "Extracted data from included papers"
outputs:
  - type: LiteratureMap
    artifact: "literature/literature_map.md"
constraints:
  - "Must use mechanism-based cluster labels"
  - "Must identify open problems per cluster"
failure_modes:
  - "Papers too heterogeneous for meaningful clustering"
tools: [filesystem]
tags: [literature, taxonomy, mapping, related-work, clustering]
domain_aware: false
---

# Literature Mapper Skill

Build a taxonomy/map of the literature to support defensible positioning and a non-chronological related work section.

## Related Task IDs

- `B6` (literature mapping)

## Output (contract path)

- `RESEARCH/[topic]/literature/literature_map.md`

## Procedure

1. Cluster papers by approach / theory / data / application.
2. Name clusters with *mechanism-based* labels (not just “paper A/B”).
3. For each cluster: representative papers + typical methods + open problems.
4. Output a map that can be reused in `B4` related work writing.
