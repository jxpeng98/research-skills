# Project instructions

## Keep the project aligned

- Codex is the primary product-development and verification Host. Other Agents
  adapt the same CLI/Skill/MCP contracts; they do not define parallel product
  workflows. Preserve their compatibility and the user's configured models.
- The [master roadmap](docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md)
  owns direction, dependencies and the current execution horizon.
- The [program ledger](docs/superpowers/roadmaps/qiongli-program-ledger-v1.json)
  owns status and accepted evidence; its
  [current index](docs/superpowers/roadmaps/qiongli-current-program-index.md)
  is generated. Never infer acceptance from a checkbox, passing local test or merge.
- Read the current bounded plan linked from the roadmap and only the relevant
  package/layer specs under `.trellis/spec/` before changing their contracts.
  Accepted ADRs own architecture; supersede them instead of rewriting history.
- Existing `.trellis/tasks/`, `.trellis/workspace/` and `.trellis/spec/` preserve
  plans, decisions and evidence. They are ordinary project knowledge, not a
  mandatory task engine. Do not reinstall Trellis skills, hooks or agents unless
  the user asks.

## Deliver small, complete increments

1. Inspect the working diff and trace the behavior to its existing owner.
2. State the user outcome and the smallest useful change. Reuse an existing plan;
   create one short plan only when the scope needs it. Routine fixes need no new
   task directory, PRD, context manifest, phase approval or journal entry.
3. Implement within the user's requested scope and run the closest meaningful
   checks. Continue through routine fixes without asking for approval again.
4. Review the final diff; update affected contracts and record checks, remaining
   gaps and the next increment once at the integration boundary. Reuse unchanged
   local results; do not repeat suites before commit, push or after merge.

The main Agent may implement and check directly. Delegate only bounded independent
work with clear file ownership when it helps; no mandatory role chain or channel.
Parallel work shares a stable interface and one integration outcome. A blocked
external validation lane does not stop independent offline development.

## Preserve the product boundaries

- Follow [CONTRIBUTING.md](CONTRIBUTING.md): develop on a local feature branch,
  run affected checks, review and commit, then merge locally into `2.x`.
  No PR or remote CI wait is required. Qualify packages at a named candidate.
- Keep input validation, permission checks, compatibility and data-loss negative
  cases. Qiongli project writes still use existing preview/approval/CAS owners.
- Preserve unrelated working changes and user data. Material scope changes or
  external/destructive actions need authority for that action; ordinary coding
  authorization does not imply publication or access to private research data.
- The maintainer authorizes local branch creation, scoped commits and local
  merges for requested development; proceed without per-step confirmation.
  Push, remote-rule changes and publication require their own scope. Never
  auto-commit unrelated bookkeeping or rerun full CI merely because of a merge.
- Program acceptance and external/manual evidence gate their readiness claims,
  not independent CLI implementation. No phase-by-phase human sign-off is needed.
