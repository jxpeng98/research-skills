---
id: chapter-architect
stage: M_dissertation
description: "Design dissertation chapter maps, chapter status, dependencies, evidence thresholds, and word-count allocations."
inputs:
  - type: DissertationPlan
    description: "Project plan and dissertation type"
outputs:
  - type: DissertationChapterMap
    artifact: "dissertation/chapter_map.md"
constraints:
  - "Must adapt chapter structure to dissertation type"
  - "Must not duplicate A-K research workflows"
  - "Must preserve chapter dependencies and missing evidence"
failure_modes:
  - "Uses one fixed chapter model for every dissertation"
  - "Lets findings precede methods or evidence readiness"
  - "Drops supervisor or handbook constraints"
tools: [filesystem]
tags: [dissertation, chapters, architecture, word-count, dependencies]
domain_aware: true
---

# Chapter Architect Skill

Create a dissertation chapter map that makes each chapter's job, dependencies, evidence threshold, and status explicit.

## Purpose

Write `dissertation/chapter_map.md` and support `dissertation/chapter_status.md` for long-running dissertation work.

## When to Use

- After dissertation planning.
- Before drafting or revising chapters.
- When a supervisor asks for chapter structure or word allocation.

## Inputs

- `DissertationPlan`
- Optional handbook, rubric map, supervisor feedback, chapter drafts, and research artifacts.
- If the plan, handbook constraints, chapter evidence, or feedback is missing or
  insufficient, record a gap note and mark the affected chapter blocked.

## Process

1. Select chapter family for empirical, qualitative, mixed-methods, review, conceptual, design-science, or professional project.
2. Assign chapter purpose, word target, dependencies, and evidence threshold.
3. Identify which A-K workflow outputs each chapter needs.
4. Mark status and blockers.

## Output Contract

Write `RESEARCH/[topic]/dissertation/chapter_map.md`; update `RESEARCH/[topic]/dissertation/chapter_status.md` when status information is available.

## Quality Bar

- [ ] Chapter structure matches dissertation type.
- [ ] Dependencies are explicit.
- [ ] Evidence gaps are visible.
- [ ] Word allocation respects the handbook or brief.
- [ ] Each finding names the available plan or artifact; interpretation states
  the dependency at the evidence's actual strength; implication sets chapter
  status, evidence work, or a blocker.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---|---|---|
| Generic chapter list | Misses project type | Adapt chapter family |
| Hidden blockers | Causes draft drift | Track status and dependencies |
| Duplicating workflows | Creates maintenance burden | Delegate research work to A-K |
| Filled evidence gaps | Creates a false chapter plan | Never invent citations, data, handbook rules, feedback, statistics, or results |
