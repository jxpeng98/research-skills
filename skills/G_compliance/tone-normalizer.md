---
id: tone-normalizer
stage: G_compliance
version: "1.0.0"
description: "Ruthlessly cut fluff, transition words, hedging, and absolute claims to produce concise, objective academic tone."
inputs:
  - type: Manuscript
    description: "Draft text requiring tone normalization"
outputs:
  - type: ToneNormalization
    artifact: "compliance/tone_normalization.md"
constraints:
  - "Must flag every hedge word and absolute claim"
  - "Must preserve technical precision while improving readability"
failure_modes:
  - "Over-cutting removes necessary nuance"
  - "Domain-specific hedging conventions not respected"
tools: [filesystem]
tags: [compliance, tone, academic-writing, editing, conciseness]
domain_aware: false
---

# Tone Normalizer Skill

Rewrite text into concise, objective academic tone and reduce reviewer-triggering overclaim.

## Related Task IDs

- `G4` (tone normalization)

## Output (contract path)

- `RESEARCH/[topic]/compliance/tone_normalization.md`

## Procedure

1. Identify high-risk patterns:
   - absolute claims (“prove”, “guarantee”, “always”)
   - vague intensifiers (“very”, “extremely”, “significant” without stats)
   - citation-free prior-work claims
2. Rewrite with calibrated language:
   - “suggest”, “is consistent with”, “we observe”
3. Enforce style rules:
   - keep paragraphs single-purpose
   - define acronyms once
   - remove filler transitions where possible

## Minimal output format (`compliance/tone_normalization.md`)

```markdown
# Tone Normalization Log

| location | original | issue | rewrite |
|---|---|---|---|
```
