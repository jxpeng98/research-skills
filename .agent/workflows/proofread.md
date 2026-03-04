---
description: AI 去痕 / 降重 / 终审校对（多 AI 协作）— Scan for AI fingerprints, rewrite flagged passages with human voice, check similarity, and final proofread
---

# Proofread & De-AI Workflow

Use this workflow after manuscript drafting (Stage F) and compliance checks (Stage G), **before** submission (Stage H).

## When to use

- Your manuscript was substantially drafted or revised with AI assistance
- You want to reduce AI-detection scores before submission
- You need a final language-level proofread pass

## Task IDs

| Task ID | Purpose |
|---|---|
| `J1` | AI fingerprint scan — detect AI-generated text patterns |
| `J2` | Human-voice rewrite — rewrite flagged passages |
| `J3` | Similarity & originality check — reduce text overlap |
| `J4` | Final proofread — grammar, consistency, flow |

## Quick Start

```
Execute Task J1 on RESEARCH/[topic] to scan the manuscript for AI-generated patterns.
Then run J2 to rewrite high-severity passages using human writing voice.
```

## Multi-AI Collaboration (Recommended)

For best results, use the orchestrator's `task-run --triad` mode:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id J2 \
  --paper-type [type] \
  --topic [topic] \
  --cwd . \
  --triad
```

This runs three agents in a loop:
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

- Task definitions: `references/workflow-contract.md`
- Stage playbook: `references/stage-J-proofread.md`
- Contract: `standards/research-workflow-contract.yaml`
