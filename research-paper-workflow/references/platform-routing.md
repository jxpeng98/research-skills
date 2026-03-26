# Platform Routing

Use this mapping to keep behavior consistent across tools.

## Claude Code

- `A1–A5` -> `/paper` (master router picks framing tasks)
- `A3` -> `/build-framework`
- `A4` -> `/find-gap`
- `B1` -> `/lit-review`
- `B2` -> `/paper-read`
- `B4` -> `/academic-write related-work [topic]`
- `C1–C5` -> `/study-design`
- `D1–D3` -> `/ethics-check`
- `E1–E5` -> `/synthesize`
- `F2` -> `/academic-write [section] [topic]`
- `F3` -> `/paper-write`
- `G1–G4` -> `/submission-prep` (reporting checks)
- `H1` -> `/submission-prep`
- `H2` -> `/rebuttal`
- `H3–H4` -> `/paper` (peer-review simulation, fatal-flaw)
- `I1–I8` -> `/code-build`
- `J1–J4` -> `/proofread`
- `K1–K4` -> `/academic-present`

## Codex

- Use `$research-paper-workflow`
- Provide: `paper_type`, `task_id`, `topic`, and optional `venue`
- Follow artifact paths from workflow contract
- For multi-agent execution, apply `standards/mcp-agent-capability-map.yaml`:
  - use `primary_agent` for draft
  - use `review_agent` for independent check
  - use `fallback_agent` when primary fails
- For proofread tasks (`J1`–`J4`), recommend `--triad` mode for iterative de-AI
- For presentation tasks (`K1`–`K4`), specify backend: `slidev`, `beamer`, or `pptx`

## Gemini

- Prompt pattern:
  `Execute Task {ID} for paper_type {paper_type} in RESEARCH/{topic} and produce contract outputs.`
- Keep task IDs and output file names unchanged
- For proofread: `Execute Task J2 for paper_type {paper_type} in RESEARCH/{topic} using multi-agent collaboration to de-AI rewrite.`
- For presentation: `Execute Task K1 for paper_type {paper_type} in RESEARCH/{topic} and prepare a conference talk using slidev backend.`
