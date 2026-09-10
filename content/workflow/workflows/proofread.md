---
description: 学术语言与终稿校对 — Improve clarity, scholarly voice, attribution and consistency while preserving claims and evidence
---

# Scholarly Proofreading Workflow

Use this workflow after manuscript drafting (Stage F) and compliance checks (Stage G), **before** submission (Stage H).

## When to use

- Your manuscript was substantially drafted or revised with AI assistance
- You want clearer prose with accurate attribution and the author's intended voice
- You need a final language-level proofread pass

## Task IDs

| Task ID | Purpose |
|---|---|
| `J1` | AI fingerprint scan — detect AI-generated text patterns |
| `J2` | Human-voice rewrite — rewrite flagged passages |
| `J3` | Similarity & originality check — reduce text overlap |
| `J4` | Final proofread — grammar, consistency, flow |

These are stable task/artifact names, not evidence of authorship detection.
J1 flags repetitive, vague or unsupported language; it cannot establish who wrote
the text. J2 improves expression while preserving meaning, citations and required
AI disclosure. J3 compares only the available sources; do not invent detector or
similarity scores, claim a complete plagiarism search, or optimize for evasion.
For a small passage, perform the requested edit directly; the full stage output
set below applies to a formal J-stage run.

## Academic Boundary Review

Before drafting this stage's checkpoint outputs, use `boundary-interviewer` when `context/boundary_review.md` is missing, stale, or contradicted by the current task. Continue within the locked boundary when the artifact already answers the stage question. Narrowing is allowed; broadening requires a new boundary review entry with a revisit trigger.

For proofread work, lock meaning-preservation and final claim wording boundaries before style, similarity, or AI-detection edits.

## Quick Start

```
Execute Task J1 on RESEARCH/[topic] to scan the manuscript for AI-generated patterns.
Then run J2 to rewrite high-severity passages using human writing voice.
```

## Independent Review When Needed

Use one agent for ordinary proofreading. When independent review is requested
or required by the task, read `skills/Z_cross_cutting/model-collaborator.md`.
If Full MCP is visible, call `qiongli_orchestrator_route`, select the registered
project revision, pass `qiongli_orchestration_doctor`, and start host-driven
Full MCP orchestration with `executionMode: "triad"`. The active Codex or
Claude host executes each bounded handoff and returns it through
`qiongli_orchestration_submit`.

Triad describes three roles, not proof of three independent models. Only an
actually available, authorized separate reviewer can provide independent review;
sequential roles in the active conversation remain self-review:
1. **Drafter** rewrites flagged passages
2. **Reviewer** re-scans for residual AI patterns
3. **Auditor** verifies scientific accuracy is preserved

## Outputs

All outputs are written to `RESEARCH/[topic]/proofread/`:

- `ai_detection_report.md` — flagged passages with pattern types and severity
- `humanized_manuscript.md` — full revised manuscript with human voice
- `similarity_report.md` — overlap analysis and rewrite suggestions
- `proofread_checklist.md` — final corrections log and style decisions

## References

Detailed task definitions and stage playbooks are in the globally installed
`qiongli-workflow` skill (auto-loaded by your AI coding tool).
Key references within the skill:
- `references/workflow-contract.md` — canonical task & output definitions
- `references/stage-J-proofread.md` — stage playbook
