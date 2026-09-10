# Project instructions

## Direction and sources

- Deliver the CLI-first research workflow through native CLI, Plugin/Skills and
  MCP. Codex is the primary development and verification Host; other Agents adapt
  the shared contracts. Preserve compatibility and the user's configured models.
  The retained Desktop is maintenance scope, not the current delivery target.
- The [master roadmap](docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md)
  owns direction and the execution horizon; read its current bounded plan.
  The [program ledger](docs/superpowers/roadmaps/qiongli-program-ledger-v1.json)
  owns task status and accepted evidence. Keep changing progress there and in the
  plan, not duplicated here. Local checks or merges do not establish acceptance.
- Accepted ADRs own architecture; supersede them rather than rewriting history.
  Read relevant `.trellis/spec/` contracts when changing their package/layer.
  Retained `.trellis/` records are project knowledge, not a mandatory task engine;
  do not reinstall Trellis skills, hooks or agents unless requested.

## Development and integration

- Follow [CONTRIBUTING.md](CONTRIBUTING.md): local feature branch → affected
  checks → review and scoped commits → local fast-forward merge into `2.x`.
  These local actions are authorized for requested work without per-step approval;
  no PR or remote CI wait is required.
- Inspect the diff and trace behavior to its existing owner. State the smallest
  useful outcome and reuse the current plan; routine fixes need no new task/PRD,
  context manifest or journal. Edit canonical sources, not generated copies.
- Run the closest meaningful checks, retaining validation, permission,
  compatibility and data-loss negative cases. Reuse results until code,
  dependencies or inputs change; commit/merge alone does not warrant a rerun.
- Review the final diff, update affected contracts, and record checks, remaining
  gaps and the next increment once at integration. External evidence gates
  readiness claims, not independent development.
- The main Agent may implement and check directly. Delegate only useful, bounded
  independent work with clear file ownership and a shared interface.
- Preserve unrelated changes and user data. Qiongli project writes use the
  existing preview/approval/CAS owners. Material scope changes, private research
  access, destructive actions, push, remote-rule changes and publication require
  authority for that action; local coding authorization does not imply it.
