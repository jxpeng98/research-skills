---
id: dissertation-planner
stage: M_dissertation
description: "Plan dissertations, theses, capstones, and major projects by degree level, method type, dependencies, milestones, and risks."
inputs:
  - type: AssignmentBrief
    description: "Dissertation handbook, brief, or project requirements when available"
  - type: RQSet
    description: "Research question set when already drafted"
outputs:
  - type: DissertationPlan
    artifact: "dissertation/dissertation_plan.md"
constraints:
  - "Must calibrate expectations to degree level"
  - "Must not invent ethics approval, data access, supervisor rules, or institutional requirements"
  - "Must route research-specific work back to A-K tasks when needed"
failure_modes:
  - "Treats undergraduate dissertation as a journal manuscript"
  - "Plans data collection without ethics or access constraints"
  - "Hides missing handbook or supervisor requirements"
tools: [filesystem]
tags: [dissertation, thesis, capstone, planning, milestones]
domain_aware: true
---

# Dissertation Planner Skill

Build a dissertation or thesis project plan that is feasible at the user's degree level.

## Purpose

Write `dissertation/dissertation_plan.md` with topic, level, research problem, method type, approvals, dependencies, timeline, and fallback risks.

## When to Use

- At the start of a dissertation, thesis, capstone, or major project.
- When a proposal needs to become a full project plan.
- When degree level, handbook rules, or data feasibility need to shape the work.

## Inputs

- `AssignmentBrief`
- Optional `RQSet`, `GapAnalysis`, `DesignSpec`, supervisor notes, handbook excerpt, and user constraints.
- If the brief, degree level, handbook, approvals, access, or supervisor input
  is missing or insufficient, record the gap and block dependent milestones.

## Process

1. Record degree level and dissertation type.
2. State the working topic, research problem, and question status.
3. Identify required upstream A-K tasks.
4. Capture approvals, data/source dependencies, ethics constraints, and milestones.
5. Mark missing handbook or supervisor requirements.

## Output Contract

Write `RESEARCH/[topic]/dissertation/dissertation_plan.md`.

## Quality Bar

- [ ] Degree level calibrates contribution expectations.
- [ ] Missing requirements are explicit.
- [ ] Research tasks are delegated to A-K where appropriate.
- [ ] Risks and fallback options are concrete.
- [ ] Each finding cites supplied requirements or project evidence;
  interpretation calibrates feasibility and contribution to the degree level;
  implication names a milestone, fallback, question, or blocker.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---|---|---|
| Overclaiming novelty | Misfits undergraduate/taught master work | Calibrate to level |
| Ignoring ethics | Blocks project later | Record ethics dependencies early |
| Inventing handbook rules | Creates compliance risk | Mark rules as missing |
