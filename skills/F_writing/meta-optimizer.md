---
id: meta-optimizer
stage: F_writing
version: "0.2.1"
description: "Optimize abstract, title, and keywords for indexing, discoverability, SEO, and reader engagement."
inputs:
  - type: Manuscript
    description: "Draft manuscript with abstract"
outputs:
  - type: MetaOptimization
    artifact: "manuscript/meta_optimization.md"
constraints:
  - "Must preserve academic tone while improving accessibility"
  - "Must test against venue word limits"
failure_modes:
  - "Over-optimization sacrificing precision for engagement"
  - "Keyword stuffing detected by reviewers"
tools: [filesystem]
tags: [writing, abstract, title, SEO, indexing, keywords]
domain_aware: true
---

# Meta Optimizer Skill

Optimize title, abstract, and keywords for indexing, discoverability, and reviewer scanning—without misrepresenting claims.

## Related Task IDs

- `F6` (meta-optimization)

## Output (contract path)

- `RESEARCH/[topic]/manuscript/meta_optimization.md`

## Inputs

- Target venue (optional)
- Current abstract + title draft
- 3–8 core constructs/keywords from Stage A/B

## Procedure

1. Title: encode construct + setting + method + contribution (as appropriate).
2. Abstract: ensure it contains (problem → gap → method → result → implication).
3. Keywords: mix author terms + common index terms; avoid stuffing.
4. Consistency check: every claim in abstract must appear in `claims_evidence_map.md`.

## Minimal output format (`manuscript/meta_optimization.md`)

```markdown
# Meta Optimization

## Title candidates (ranked)
1. ...

## Abstract revision
- Original:
- Revised:

## Keywords
- ...
```
