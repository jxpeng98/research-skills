---
description: Qiongli unified research workflow entrypoint for Claude Desktop, Claude Code, and plugin installs.
---

# Qiongli Unified Workflow Router

Use this workflow when the user invokes Qiongli directly, asks for the main
Qiongli entry, or gives a natural academic research request without choosing a
specific stage command.

## Context

$ARGUMENTS

## Routing Contract

Read `references/platform-routing.md` and `references/workflow-contract.md`
before choosing a route. Do not treat this file as a separate workflow contract;
it is a thin entrypoint that delegates to the existing canonical workflows.

## Route Selection

1. If the user supplied a Task ID such as `B1`, `F3`, or `H2`, map it through
   `references/workflow-contract.md` and route to the matching workflow.
2. If the request is broad, vague, or starts a new paper project, route to
   `workflows/paper.md`.
3. If the request asks for a whole project lifecycle, roadmap, or journal-fit
   path, route to `workflows/paper-lifecycle.md`.
4. If the request clearly matches a stage-specific workflow, route directly:
   - literature review or search protocol -> `workflows/lit-review.md`
   - paper, DOI, PDF, citation, or notes reading -> `workflows/paper-read.md`
   - research gap discovery -> `workflows/find-gap.md`
   - theory, constructs, or framework -> `workflows/build-framework.md`
   - manuscript section writing -> `workflows/academic-write.md`
   - evidence synthesis or meta-analysis -> `workflows/synthesize.md`
   - full manuscript assembly -> `workflows/paper-write.md`
   - study design, variables, robustness, or preregistration ->
     `workflows/study-design.md`
   - ethics, IRB, consent, or deidentification -> `workflows/ethics-check.md`
   - reporting, submission package, or journal materials ->
     `workflows/submission-prep.md`
   - reviewer response or revision -> `workflows/rebuttal.md`
   - analysis code, notebooks, replication, or reproducibility ->
     `workflows/code-build.md`
   - proofreading, de-AI rewriting, or final copyedit ->
     `workflows/proofread.md`
   - academic slides or presentations -> `workflows/academic-present.md`
5. If the route is still ambiguous, ask one blocking academic question with a
   recommended answer and rationale, then route to `workflows/paper.md`.

## Required Behavior

- Preserve the original user request as routing context when delegating.
- State which workflow file will be used before executing it.
- Load and follow the delegated workflow file as the source of truth for task
  order, artifacts, prerequisites, and quality gates.
- Keep Qiongli capability modes explicit: skill/plugin routing is available by
  default; provider-connected literature search requires the literature MCP or
  MCPB; project orchestration requires the native Full MCP server and runs each
  bounded handoff in the active Codex or Claude host.
