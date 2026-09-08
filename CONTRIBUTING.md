# Contributing to Qiongli

Start with the
[Qiongli 2 master roadmap](docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md).
It owns product direction, ordering, and the current execution horizon. The
[program ledger](docs/superpowers/roadmaps/qiongli-program-ledger-v1.json) owns
task state and accepted evidence. The current implementation plan selects a
bounded user outcome from that roadmap; existing `.trellis/` records remain
project knowledge without requiring Trellis skills or a task lifecycle.

## Development and integration

Default loop: create a local branch from `2.x` → edit → affected **Focused**
checks → review and commit → merge locally into `2.x` → next increment.
No PR, remote CI wait, post-merge suite, package acceptance or phase sign-off
is required for routine development. Integrate one complete behavior at a time.

The main Agent can implement, review and check directly. Reuse the current short
plan; routine fixes need no new task, PRD, journal or duplicate evidence report.
Record tests and remaining gaps once at the integration boundary. Source progress
can continue while a separate live Host, package or program acceptance claim is
pending. Acceptance status is not a prerequisite for independent implementation.

Run only affected tests while editing. Reuse results until their code,
dependencies or inputs change; commits and local merges do not themselves
require rerunning them. Security, authorization, schema, path, ownership and
data-loss changes retain their immediate negative checks.

The maintainer's local-development instruction covers branch creation, scoped
commits and local merges for the requested outcome. Follow the
[delivery checklist](.github/delivery-checklists.md) without asking at each step.
Use `git merge --ff-only <branch>` into local `2.x`; if it has advanced, merge
`2.x` into the feature branch, resolve conflicts and check the affected combined
behavior first. Run the fast frozen-source guard before merging; reuse existing
local test results when their inputs are unchanged.

Push, remote-rule changes, research-data access and publication remain separately
scoped actions. Existing GitHub protection applies only when synchronizing with
the remote and does not block local development.

Keep changes in their canonical boundary:

- academic content: `content/`;
- native Qiongli 2 source: `packages/qiongli-native/`;
- stable command wrappers: `scripts/`, with implementations in `tooling/scripts/`;
- generated plugin/package payloads: regenerate through the supported
  materialization workflow; never edit generated copies directly.

## Optional remote verification

A PR is created only when explicitly requested for remote collaboration; use the
[optional template](.github/pull_request_template.md). The existing PR-triggered
CI remains available with these scopes, but is not a local merge prerequisite:

- CLI: headless workspace tests on Linux, macOS and Windows; format and CLI
  Clippy once on Linux. `qiongli-ui`'s GUI tests are excluded from headless work.
- Desktop: shared native source/build or desktop/frontend changes add Linux
  desktop consumer tests and Clippy. Dedicated CLI/MCP paths skip the renderer.
- Lite: changes to Lite, its shared runtime dependencies or unknown/tooling
  inputs add compatibility tests; CLI app-only work skips the separate package.
- Non-runtime docs/process/evidence use lightweight native contexts. Unknown,
  workflow, fixture and empty changes conservatively include desktop and Lite.

The remote ruleset and context names remain unchanged; local merges do not
trigger them. Do not open a PR or change remote protection merely to complete
local development. Full desktop matrices, packages, live Hosts and capacity
measurements belong to a named candidate, not every extraction increment.

## Builds

For the retained desktop target, use [Local Desktop Development and Packaging](docs/development/local-desktop-build.md)
for the macOS, Windows, and Linux build loop. Record the exact source and named
target. Cross-compilation proves compilation only; it does not claim target-native
runtime, signing, installer, or release acceptance.

## Releases

Release work starts only from an explicit release task after merge to `2.x`.
Follow the Release section of the delivery checklist and run the existing owner:

```bash
./scripts/release_ready.sh --cli-github --version <version> --staging-dir <new-external-dir>
```

CI supplies evidence. A named human release decision authorizes publication, and
a separate decision authorizes announcement.

The CLI lane builds no App and requires no Community Alpha signing key. It
qualifies macOS ARM64 GitHub assets and local npm/wheel installs; publication uses
an immutable tag and `gh release`, followed by public download verification.
Registry uploads and managed-product activation remain separate. See
[ADR 0219](docs/architecture/decisions/0219-cli-github-release-distribution.md).
Omit `--cli-github` only for the retained legacy/desktop diagnostic lanes.
