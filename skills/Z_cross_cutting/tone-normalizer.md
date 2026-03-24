---
id: tone-normalizer
stage: Z_cross_cutting
version: "0.1.0"
description: "Cross-cutting tone normalization for any text output — rebuttals, emails, IRB documents, informal drafts. Canonical skill at G_compliance/tone-normalizer.md."
inputs:
  - type: AnyArtifact
    description: "Any text requiring tone normalization (manuscript, rebuttal, email, IRB)"
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
tags: [cross-cutting, tone, academic-writing, editing, conciseness]
domain_aware: false
---

# Tone Normalizer (Cross-Cutting)

> **Canonical version**: `skills/G_compliance/tone-normalizer.md`
>
> This is the cross-cutting variant for use beyond compliance stage — rebuttals, emails, IRB documents, informal drafts, and any text output from any stage.

Rewrite text into concise, objective academic tone and reduce reviewer-triggering overclaim.

## When to Use This vs G_compliance Version

| Use This (Z_cross_cutting) | Use G_compliance |
|---|---|
| Rebuttal response letters | Manuscript compliance pass |
| Email drafts to collaborators | Pre-submission tone audit |
| IRB consent forms | Reporting guideline check |
| Conference abstracts | Journal-specific requirements |
| Grant proposals | Full manuscript normalization |

## Procedure

1. Identify high-risk patterns:
   - absolute claims ("prove", "guarantee", "always")
   - vague intensifiers ("very", "extremely", "significant" without stats)
   - citation-free prior-work claims
2. Rewrite with calibrated language:
   - "suggest", "is consistent with", "we observe"
3. Enforce style rules:
   - keep paragraphs single-purpose
   - define acronyms once
   - remove filler transitions where possible

## Output Format

```markdown
# Tone Normalization Log

| location | original | issue | rewrite |
|---|---|---|---|
```
