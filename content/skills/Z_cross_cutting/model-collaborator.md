---
id: model-collaborator
stage: Z_cross_cutting
description: "Coordinate independent multi-agent execution, disagreement tracking, and synthesis across literature, writing, review, and code tasks."
inputs:
  - type: TaskPacket
    description: "Task specification for multi-agent research execution and cross-review"
outputs:
  - type: CollaborationTrace
    artifact: "logs/model_collab_trace.md"
constraints:
  - "Must ensure independent execution before synthesis"
  - "Must document agent disagreements explicitly"
failure_modes:
  - "Agent unavailable for scheduled collaboration"
  - "Output format incompatible across agents"
tools: [filesystem]
tags: [cross-cutting, multi-agent, collaboration, independent-review]
domain_aware: false
---

# Model Collaborator Skill

## Purpose

Coordinate bounded independent review of literature, writing, qualitative coding,
statistics, analysis code or rebuttals. Match roles to the actual available
capabilities and evidence needs, not to model or Host brand stereotypes.

## When to Use

Use for requested independent review or a research task whose gate requires it.
A quick edit or ordinary reading task normally needs one agent. Multiple roles
in one conversation are useful self-review but are not independent execution.

## Inputs

- `TaskPacket`: objective, Task ID, source artifacts, constraints and output path.
- Available Host/tool capabilities and any required reviewer independence.
- If sources, permissions or an independent reviewer are missing, record a gap note
  and complete only the supported portion. Do not invent a second review.

## Process

1. Choose the smallest useful arrangement: independent parallel reviews for
   screening or competing interpretations; draft then review for a manuscript
   or code change; one-agent self-review when independence is not required.
2. Use the user's current model configuration. Do not install another runtime,
   request API keys or replace the configured model to fill a role. Load
   `references/platform-routing.md` for native 2.x execution and recovery.
3. For registered project orchestration, use visible `qiongli_orchestrator_route`
   and follow the returned Full MCP sequence: select project/revision, run
   `qiongli_orchestration_doctor`, start, read evidence, submit the bounded
   candidate, then obtain the next handoff. A Lite preview cannot execute a run.
4. If authorized native subagents exist, give each a bounded packet with the
   same source revision, question, evidence anchors, output contract and allowed
   actions. Independent first-pass reviewers should not receive the other's
   verdict. A draft-review chain necessarily exposes the draft: describe that
   dependence honestly. Keep candidates isolated; do not permit concurrent
   canonical project writes.
5. Compare findings against sources, diagnostic results and the research method.
   Record conflicting claims, evidence for each and the resolution or remaining
   blocker. Majority agreement and high confidence are not evidence; do not
   discard a supported minority objection.
6. Return one synthesis plus source-bound candidate(s). Preserve actual run,
   revision, generation, document/handoff digests and evidence references when
   supplied by Qiongli. Candidate submission does not approve artifact apply.
   If the independent lane is unavailable, report it explicitly and offer a
   bounded self-review without marking its independent-review gate as passed.

## Output Contract

Write `RESEARCH/[topic]/logs/model_collab_trace.md` through the applicable project
write owner. Until persistence is approved, present it as a proposed trace.

The trace records task and sources, actual participants/capabilities, independence
or dependence, findings with source anchors, disagreements, resolutions and
remaining gaps. Use `templates/duo-review-report.md` when its structured review
is needed. Describe concise decision rationale, not hidden reasoning traces.
Do not invent session IDs, evidence hashes, success flags, citations, data,
sample sizes, statistical results or reviewer comments. Separate finding,
interpretation and implication; apply `references/academic-output-rubric.md`.

## Quality Bar

- Every claimed independent review has an actual separately executed output.
- Reviewers worked on the declared source revision and did not share first-pass verdicts.
- Conflicts are resolved using evidence or carried forward as explicit blockers.
- Candidate and approval bindings are preserved; no review result authorizes a write.
- Missing tools or reviewers produce a truthful partial result, not a simulated success.

## Common Pitfalls

| Pitfall | Correction |
|---|---|
| Assign expertise from a model name | Use the actual task, tools and observed capability |
| Treat sequential personas as independent reviewers | Label the work self-review and keep the unmet gate open |
| Merge by majority or confidence | Resolve against source evidence and methodological validity |
| Reuse stale context after a handoff | Re-read authorized evidence at the current revision |
| Let reviewers overwrite one another | Keep bounded candidates separate until approved integration |
