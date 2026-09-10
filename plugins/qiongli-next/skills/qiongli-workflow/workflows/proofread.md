---
description: 学术语言与终稿校对 — Improve clarity, scholarly voice, attribution and consistency while preserving claims and evidence
---

# Scholarly Proofreading Workflow

Use this workflow for requested scholarly language edits and formal Stage J
checks. A complete pre-submission pass follows drafting and applicable compliance
checks; a supplied passage can be corrected directly.

## When to use

- You want clearer prose with accurate attribution and the author's intended voice
- You need a final language-level proofread pass

## Task IDs

| Task ID | Purpose |
|---|---|
| `J1` | Language-pattern audit — flag concrete clarity and repetition problems |
| `J2` | Scholarly voice revision — improve selected passages without meaning drift |
| `J3` | Source-overlap check — inspect available sources and attribution |
| `J4` | Final proofread — grammar, consistency, flow |

These are stable task/artifact names, not evidence of authorship detection.
J1 flags repetitive, vague or unsupported language; it cannot establish who wrote
the text. J2 improves expression while preserving meaning, citations and required
AI disclosure. J3 compares only the available sources; do not invent detector or
similarity scores, claim a complete plagiarism search, or optimize for evasion.
For a small passage, perform the requested edit directly; the full stage output
set below applies to a formal J-stage run.
Choose J4 for grammar/consistency, J2 for requested expression changes, J1 for a
pattern diagnosis and J3 for source comparison. Mentioning proofreading does not
schedule J1 → J2 → J3 → J4. Existing source/claim problems remain visible even
when they are outside a language-only edit.

## Academic Boundary Review

Before formal checkpoint outputs, inspect the existing boundary and request.
Use `boundary-interviewer` only for an unresolved consequential decision. A
missing boundary file alone does not block a direct language edit. Broadening
locked research claims requires a new boundary decision with a revisit trigger.

Preserve meaning, claim strength, citations and required AI disclosure.

## Quick Start

```
Proofread this paragraph for grammar; keep its claims and citations unchanged.
Execute Task J3 on RESEARCH/[topic] to compare the draft with the supplied sources.
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
2. **Reviewer** checks whether the requested language issues are resolved
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
