# Platform Routing

Use this mapping to keep behavior consistent across tools.

## Claude Code

- `A3` -> `/build-framework`
- `A4` -> `/find-gap`
- `B1` -> `/lit-review`
- `B2` -> `/paper-read`
- `B4` -> `/academic-write related-work [topic]`
- `C1-C5` -> `/study-design`
- `D1` -> `/ethics-check`
- `E1-E5` -> `/synthesize`
- `F2` -> `/academic-write [section] [topic]`
- `F3` -> `/paper-write`
- `H1` -> `/submission-prep`
- `H2` -> `/rebuttal`
- `I1-I3` -> `/code-build`

## Codex

- Use `$research-paper-workflow`
- Provide: `paper_type`, `task_id`, `topic`, and optional `venue`
- Follow artifact paths from workflow contract
- For multi-agent execution, apply `standards/mcp-agent-capability-map.yaml`:
  - use `primary_agent` for draft
  - use `review_agent` for independent check
  - use `fallback_agent` when primary fails

## Gemini

- Prompt pattern:
  `Execute Task {ID} for paper_type {paper_type} in RESEARCH/{topic} and produce contract outputs.`
- Keep task IDs and output file names unchanged
