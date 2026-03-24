---
id: fatal-flaw-detector
stage: H_submission
version: "0.2.1"
description: "Constructive desk-reject analysis identifying critical flaws that would prevent publication."
inputs:
  - type: Manuscript
    description: "Draft manuscript"
outputs:
  - type: FatalFlawAnalysis
    artifact: "revision/fatal_flaw_analysis.md"
constraints:
  - "Must categorize flaws by severity (fatal/major/minor)"
  - "Must propose specific remediation for each flaw"
failure_modes:
  - "False positive on methodological choices that are defensible"
  - "Missing domain-specific fatal flaws"
tools: [filesystem]
tags: [submission, fatal-flaw, desk-reject, quality-gate, pre-submission]
domain_aware: true
---

# Fatal Flaw Detector Skill

Constructive desk-reject analysis: identify flaws likely to cause immediate rejection and propose mitigations.

## Related Task IDs

- `H4` (fatal flaw analysis)

## Output (contract path)

- `RESEARCH/[topic]/revision/fatal_flaw_analysis.md`

## Inputs

- `paper_type`, target `venue`
- Latest `manuscript/manuscript.md`
- Any available methods/results artifacts

## Failure modes to test (examples)

- Novelty not defensible (positioning/gap unsupported)
- RQ-method mismatch or unclear contribution
- Identification/validity threats unaddressed
- Underpowered / non-credible evidence for the claim strength
- Reporting checklist gaps that signal low rigor
- Reproducibility absent (code/data availability inconsistent with claims)

## Minimal output format (`revision/fatal_flaw_analysis.md`)

```markdown
# Fatal Flaw Analysis

| flaw | why it is fatal | evidence/location | fix | feasibility |
|---|---|---|---|---|
```
