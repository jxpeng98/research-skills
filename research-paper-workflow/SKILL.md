---
name: research-paper-workflow
description: Standardized end-to-end workflow for academic paper production across Codex, Claude Code, and Gemini. Use when a user needs to choose a paper type (empirical, systematic review, methods, theory), select a workflow stage, and produce consistent artifacts under RESEARCH/[topic]/ with explicit task IDs, quality gates, and submission-ready outputs.
---

# Research Paper Workflow

Run a model-agnostic paper workflow using shared Task IDs and artifact contracts.

## Quick Start

1. Ask for `paper_type`: `empirical`, `systematic-review`, `methods`, or `theory`.
2. Ask for `task_id` from the contract (for example `F3` or `G1`).
3. Execute the task and write outputs to `RESEARCH/[topic]/` using the exact file paths.
4. Apply quality gates before submission tasks (`H1`, `H2`).

## Required Behavior

- Use the canonical task and output definitions in `references/workflow-contract.md`.
- Keep stage labels and task IDs unchanged across models.
- If a requested output is missing prerequisites, create a gap note and ask whether to:
  1. continue with placeholders, or
  2. run the prerequisite task first.
- Keep claims, methods, and evidence aligned (run integrity checks for stage `G`).

## References

- Task model + outputs: `references/workflow-contract.md`
- Platform routing map: `references/platform-routing.md`
- Coverage matrix: `references/coverage-matrix.md`
- Stage playbooks:
  - `references/stage-A-framing.md`
  - `references/stage-B-literature.md`
  - `references/stage-C-design.md`
  - `references/stage-D-ethics.md`
  - `references/stage-E-synthesis.md`
  - `references/stage-F-writing.md`
  - `references/stage-G-compliance.md`
  - `references/stage-H-submission.md`
  - `references/stage-I-code.md`
