---
id: dissertation-readiness-checker
stage: M_dissertation
description: "Check dissertation final readiness and defense preparation against chapter completeness, evidence alignment, formatting, integrity, and unresolved risks."
inputs:
  - type: DissertationPlan
    description: "Dissertation project plan"
  - type: DissertationChapterMap
    description: "Chapter map and status"
outputs:
  - type: DissertationReadinessReport
    artifact: "dissertation/final_readiness.md"
constraints:
  - "Must not promise supervisor approval, marks, or degree outcome"
  - "Must flag missing chapters, unresolved feedback, weak evidence, and policy gaps"
  - "Must separate final readiness from viva or defense preparation"
failure_modes:
  - "Treats a formatted document as academically ready"
  - "Ignores unresolved supervisor feedback"
  - "Produces defense questions unsupported by the draft"
tools: [filesystem]
tags: [dissertation, readiness, defense, viva, final-check]
domain_aware: true
---

# Dissertation Readiness Checker Skill

Check whether a dissertation or thesis is ready for final submission or defense preparation.

## Purpose

Write `dissertation/final_readiness.md` and, when requested, support `dissertation/defense_prep.md`.

## When to Use

- Before final dissertation submission.
- After major supervisor feedback has been addressed.
- When the user asks for viva or defense preparation.

## Inputs

- `DissertationPlan`
- `DissertationChapterMap`
- Optional supervisor feedback log, chapter drafts, formatting rules, and integrity notes.
- If chapters, feedback, formatting rules, or evidence are missing or
  insufficient, record a gap note and keep final readiness blocked.

## Process

1. Check chapter completeness and dependencies.
2. Check research question, method, evidence, and discussion alignment.
3. Check citation coverage, formatting constraints, AI-policy notes, and unresolved risks.
4. Produce defense preparation only from supplied dissertation content.

## Output Contract

Write `RESEARCH/[topic]/dissertation/final_readiness.md`; write `RESEARCH/[topic]/dissertation/defense_prep.md` when defense preparation is requested.

## Quality Bar

- [ ] Missing chapters and unresolved feedback are visible.
- [ ] Evidence and method alignment are checked.
- [ ] Integrity and formatting constraints are preserved.
- [ ] Defense preparation does not invent dissertation content.
- [ ] Each finding points to a supplied chapter or requirement; interpretation
  states the readiness risk without predicting approval; implication names the
  required revision, evidence, confirmation, or blocker.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---|---|---|
| Formatting-only readiness | Misses academic risks | Check claims and evidence |
| Grade or approval promise | Unsupported | Use readiness language |
| Invented viva answers | Misrepresents work | Use supplied draft only |
