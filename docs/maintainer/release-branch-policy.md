# Release Branch Policy

The accepted Python-led 1.x line is now frozen. This repository uses `2.x` for
Rust-native development, keeps `release/1.x-python` as the accepted 1.x
compatibility oracle and critical-fix line, and retains `dev` as the recorded
post-release handoff branch. `main` remains the legacy stable release branch.

## Branch Roles

| Branch | Role | Allowed changes |
|--------|------|-----------------|
| `2.x` | Active Rust-native development, integration, and 2.x prerelease source | Native Rust workspace and product features, contract/resource loaders, native CLI/UI/MCP/orchestrator work, installers, tests, docs, CI, and 2.x release tooling. Python and Node may be used only as frozen oracle or build-time test inputs, never as production runtime dependencies. |
| `dev` | Accepted 1.x handoff and post-release baseline integration endpoint | A8 baseline evidence, branch governance, documentation, tests, and handoff metadata. No new 1.x product features and no Rust-native product implementation. |
| `release/1.x-python` | Accepted 1.x tag, compatibility oracle, and critical-fix-only maintenance line | Approved security or release-breakage corrections through pull requests, plus the minimum tests, release metadata, and documentation required for those corrections. No normal features. |
| `main` | Legacy stable release source | Stable release evidence and explicitly approved emergency maintenance. No normal 1.x feature development. |

Open native feature pull requests against `2.x`. Use `dev` only for the A8
handoff and cross-line governance after the final 1.x beta. Do not merge native
implementation back into `dev`, `main`, or `release/1.x-python`.

## 1.x Maintenance Governance

`release/1.x-python` points at the accepted annotated tag
`v1.19.0-beta.1` (`8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f`). It is not a
moving copy of `dev`. Because it was cut at the accepted tag, it does **not**
contain the A8 workflow-filter changes committed later on `dev`; do not claim
that those later workflow definitions are present on the frozen branch.

The repository ruleset for this line is
[ruleset 18797579](https://github.com/jxpeng98/qiongli/rules/18797579). It
requires pull requests for changes to `release/1.x-python`; direct maintenance
pushes are not the operating procedure. It also blocks deletion and
non-fast-forward updates and has no bypass actors. The server-side ruleset is
the source of enforcement, while this document records the review policy.

A 1.x maintenance pull request is allowed only when all of the following hold:

1. the change fixes a critical security issue or release breakage and names the
   exception explicitly;
2. the failure is reproduced against the accepted tag and the smallest safe
   patch is proposed through a pull request;
3. the pull request includes focused regression tests, full applicable release
   gates, and artifact or rollback evidence proportional to the change;
4. the same behavior is forward-ported to Rust 2.x, or the pull request records
   equivalence evidence showing why the Rust line is unaffected and names any
   follow-up owner;
5. the change does not add a 1.x feature or silently move the frozen oracle.

The planned 1.x support window ends **90 days after Qiongli 2 stable** unless a
later, explicit support decision supersedes it. The security and
release-breakage exception remains subject to a release-owner decision; this
window is not permission to resume feature development.

## 2.x Native Branch Governance

Current development is local: create a feature branch from local `2.x`, edit,
run affected checks, review and commit, then `git merge --ff-only` into local
`2.x`. No pull request or GitHub CI wait is required. The maintainer's instruction
covers local branch/commit/merge within the requested scope. Remote sync and
remote-rule changes are separate actions. The GitHub policy below describes the
optional remote route, not a local integration gate.

`2.x` is created from the exact clean A8 handoff commit after the normalized
1.x baseline is frozen. It owns all subsequent native implementation and 2.x
release work.

`Native CI` runs automatically for pull requests targeting `2.x`; merge pushes
do not start a duplicate run. It remains manually dispatchable for an explicit
candidate. Its required checks are:

- `Native 2.x change boundary`;
- `Rust native foundation (Linux)`;
- `Rust native foundation (macOS)`;
- `Rust native foundation (Windows)`.

For a ready source PR, run headless workspace tests on Linux, macOS and Windows
and format/CLI Clippy once on Linux. Shared native source/build and desktop
changes add Linux desktop consumer checks; dedicated CLI/MCP changes skip the
renderer. Lite compatibility covers changes to Lite, its shared runtime
dependencies and unknown/tooling inputs.
Draft pull requests defer native tests. Native CI and Evaluation Truth merge pushes
do not start a duplicate run.

For a non-runtime documentation or evidence-only pull request, native contexts
use lightweight reports. Unknown/workflow/fixture/empty changes conservatively
select all PR checks; deleted source remains source. Explicit `workflow_dispatch`
runs the full three-platform desktop, Lite, package and candidate checks.
See [CONTRIBUTING](https://github.com/jxpeng98/qiongli/blob/2.x/CONTRIBUTING.md)
for the current development loop. The required `Evaluation Truth V1` context
also runs once on each PR head; it does not require human confirmation.

`Legacy Compatibility CI` and
`Legacy Checkout Install Check` continue to run automatically for `main`,
`master`, and `dev`. Both remain manually dispatchable against a named `2.x` ref
when a specific compatibility question requires the frozen Python, Node, Rust
Lite, distribution, or checkout oracle. Their results are diagnostic and are not required checks for native 2.x work.

The dependency-free native change boundary resolves a pull request's
`github.base_ref`; a manual dispatch falls back safely to the first available
parent/root. It rejects changes to the
accepted Python/Node product paths, the versioned 1.x baseline and its schemas,
including `tooling/migration/baselines/v1.19.0-beta.1/manifest.json`, the 2.x
branch-point record, and ADRs 0201-0207. The deeper frozen-baseline guard and
asset-backed `capture --check` remain available in the manually dispatched
legacy workflow for a named compatibility investigation. New conformance
evidence uses a new versioned path rather than rewriting accepted 1.x evidence.

The active enforcement source is ruleset `18800504`, which requires pull
requests, the four native contexts above and `Evaluation Truth V1`, blocks deletion and non-fast-forward
updates, and has no bypass actors. The immutable guard is preventive only when
its workflow is required; without server-side enforcement, a direct push would
be unvalidated because merge pushes do not start `Native CI`.

Production code on `2.x` must be Rust-native and dependency-free for end users.
Frozen Python Full, Rust Lite, and Node MCPB results remain compatibility
oracles and test evidence; they are not allowed to become hidden production
dependencies.

## Verification Tiers

Use the smallest tier that matches the delivery boundary:

1. **Focused** — during implementation, run only the smallest check that can
   falsify the changed behavior. Run security, authorization, schema, path,
   ownership, and data-loss negative checks as soon as those boundaries change.
   On Apple Silicon macOS, native work may use the complete macOS workspace
   test and the third-party `cargo-xwin` commands below for early Windows x64
   compilation feedback. Using `cargo-xwin` accepts the Microsoft SDK licence
   and therefore requires explicit maintainer approval before first use.
2. **Slice** — optional remote collaboration only. Routine integration uses
   the local branch/check/commit/merge loop above; no PR or CI wait is required.
3. **Acceptance** — only for an explicit 2.x cutover or release candidate, run
   target packages, packaged-product and Lite candidate acceptance, current live
   Hosts, migration/rollback, trust/supply-chain, and claimed manual journeys.

Automatic `2.x` pull-request runs do not assemble the three target product
packages, run packaged-product acceptance, run Lite candidate acceptance, or
dispatch Community Alpha promotion, and merge pushes do not start `Native CI`.
Those jobs run only on an explicit `workflow_dispatch` candidate action. A
green Slice is integration evidence, not release authorization.

For the macOS-first native loop, run these commands from
`packages/qiongli-native/` so the pinned Rust toolchain applies:

```bash
cargo test --workspace --all-targets --all-features --locked
cargo xwin build --workspace --release --target x86_64-pc-windows-msvc --locked
cargo xwin test --workspace --no-run --all-features --target x86_64-pc-windows-msvc --locked
```

The second command produces Windows x64 PE/COFF artifacts and the third only
compiles Windows test executables. Neither is a Windows runtime pass. Run the
affected startup, persistence, and failure smoke paths in a Windows guest or
runner, and retain the ready-PR native Windows context as the Slice authority.
Windows 11 Arm with x64 emulation is useful day-to-day evidence, not native
Windows x64 hardware certification, signing, installer, or release acceptance.

## Official Plugin Linkage

The official public marketplace entry lives in `jxpeng98/skillsplace` and should point at the stable generated Qiongli plugin payload:

- Marketplace repository: `https://github.com/jxpeng98/skillsplace`
- Qiongli repository: `https://github.com/jxpeng98/qiongli`
- Stable Codex artifact: `qiongli-core-codex-plugin-<tag>.tar.gz`
- Stable Claude Code artifact: `qiongli-core-claude-plugin-<tag>.tar.gz` or `.zip`
- Stable generated payload root: `plugins/qiongli/`

The stable Skillsplace catalog entries should track `main` and stable release
tags, not `dev`. Use `2.x` for native plugin packaging tests and prerelease
validation after the A8 handoff; use `dev` only to preserve the recorded A8
baseline and governance evidence. This repository no longer carries Codex or
Claude marketplace catalog files; it owns the plugin manifests and materializes
the release payload from canonical source.

Legacy 1.x beta tags publish the `qiongli-next` testing channel instead of the
full stable marketplace matrix. Native 2.x alpha dry-runs do not publish or
build these plugin artifacts. The legacy-generated next artifacts are:

- `qiongli-next-codex-plugin-<tag>.tar.gz`
- `qiongli-next-claude-plugin-<tag>.tar.gz`
- `qiongli-next-claude-plugin-<tag>.zip`
- `qiongli-next-claude-desktop-skill-core-<tag>.zip`

The `qiongli-next` Codex and Claude Code plugin artifacts install only the `core/complete` skill package and keep the bundled Rust Lite literature MCP runtime. They do not publish subject plugin variants. The Claude plugin ZIP contains the same plugin payload as the Claude tarball for upload flows that reject `.tar.gz`. Claude Desktop testing uses the focused core ZIP plus the separate `qiongli-literature-provider-<version>.mcpb` release asset. The Skillsplace catalog may expose a separate `qiongli-next` entry for beta testing while stable `qiongli` and subject entries continue to point at stable artifacts.

This repository does not track stable or beta plugin payload directories. `plugins/qiongli/`, `plugins/qiongli-next/`, `packages/qiongli-plugin/`, and `packages/qiongli-next-plugin/` are generated shapes. Change `content/workflow/`, `content/distribution/plugins.yaml`, or `tooling/scripts/build_plugin_artifacts.py`, then materialize into a staging directory for validation.

## Platform dist refs

Codex and Claude marketplace installs from `jxpeng98/skillsplace` use Git subdirectory sources when the reviewed plugin payload is generated at release time. Keep the existing release tag, GitHub Release, PyPI, npm, and archive artifact flow unchanged, and publish separate orphan branch refs for each platform:

```text
refs/heads/codex/v<version>
refs/heads/claude/v<version>
```

## Codex dist refs

Each Codex dist ref must contain only the generated plugin payload tree needed by the Codex marketplace entry:

```text
plugins/qiongli/.codex-plugin/plugin.json
plugins/qiongli-next/.codex-plugin/plugin.json
```

Codex refs must not include `.claude-plugin/`; Claude refs carry that metadata separately.

## Claude dist refs

Each Claude dist ref must contain only the generated plugin payload tree needed by the Claude marketplace entry:

```text
plugins/qiongli/.claude-plugin/plugin.json
plugins/qiongli-next/.claude-plugin/plugin.json
```

Claude refs must not include `.codex-plugin/` or `.mcp.json`. The legacy 1.x
release postflight publishes platform dist refs after it materializes the
release staging payload and builds the existing plugin artifacts. Legacy stable
marketplace installs publish `plugins/qiongli`; legacy beta installs publish
`plugins/qiongli-next`. A native 2.x alpha dry-run never publishes these refs,
and native postflight remains blocked until target and package identities are
truthful and accepted.

Use `scripts/publish-codex-dist-ref.mjs` manually only when backfilling an existing release or intentionally repairing a dist ref from a verified staging directory:

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
node scripts/publish-codex-dist-ref.mjs --channel codex --version 1.3.0 --slug qiongli --source /tmp/qiongli-dist/plugins/qiongli
node scripts/publish-codex-dist-ref.mjs --channel claude --version 1.3.0 --slug qiongli --source /tmp/qiongli-dist/plugins/qiongli
node scripts/publish-codex-dist-ref.mjs --channel codex --version 1.5.0-beta.1 --slug qiongli-next --source /tmp/qiongli-dist/plugins/qiongli-next
node scripts/publish-codex-dist-ref.mjs --channel claude --version 1.5.0-beta.1 --slug qiongli-next --source /tmp/qiongli-dist/plugins/qiongli-next
```

The publisher validates the channel-specific manifest, bundled MCP entrypoint, portable skill package, and version fields before it writes the orphan ref. Re-run with `--force` only when intentionally replacing an existing platform `v<version>` payload.

## Development Flow

1. After A8 records the branch point, start all native feature and packaging
   work on `2.x` and open pull requests back to `2.x`.
2. Run Focused checks while editing. Keep the pull request in draft while the
   slice is moving; draft events do not expand the native matrix. Once ready,
   `Native CI` runs on the exact pull-request commit. Source-affecting changes
   must pass format, check, Clippy, workspace tests, Linux portable frontend
   checks, Lite compatibility, and the frozen change boundary. Allowlisted
   non-runtime documentation or evidence-only closeout runs only the boundary
   and lightweight required contexts. The merge push does not repeat the run.
   Apple Silicon maintainers may use the macOS workspace plus the documented
   `cargo-xwin` build/test-compilation loop before that Slice; it does not
   replace Windows runtime or required-CI evidence.
   Dispatch the legacy workflows only for a named compatibility question and
   record equivalence evidence where the migrated surface already has an
   accepted oracle.
3. Materialize legacy portable payloads only into a staging directory for
   comparison or artifact validation:

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
```

4. Keep the frozen 1.x source and baseline read-only during ordinary 2.x work.
   The immutable CI surface includes the versioned baseline directory,
   `qiongli-1x-baseline-plan.json`, `baseline-plan.schema.json`,
   `baseline-manifest.schema.json`, and `oracle-fixture.schema.json`. Run the
   applicable legacy validators manually as compatibility evidence, not as a
   required 2.x check or production dependency:

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest discover -s tests -v
```

5. Route any 1.x security or release-breakage exception to
   `release/1.x-python` under the PR-only policy above. Do not use `dev` as a
   feature-bearing 1.x release source.
6. For an explicit 2.x candidate, manually dispatch `Native CI` on the frozen
   `2.x` source so target package assembly, packaged acceptance, Lite candidate
   acceptance, and the existing exact promotion dispatch run together. Never
   use an automatic pull-request Slice as candidate or release authorization.
7. Use the B1 native preflight only as an external-staging dry-run. It now
   validates alpha syntax, the Cargo version/channel source, isolated channel
   metadata, a planned target identity, and rollback/promotion semantics. Do
   not create or publish a 2.x tag until the later native artifact, signing,
   target acceptance, updater, and release gates remove the explicit
   `publication_allowed=false` blocker.

## Stable Release Rule

The accepted `v1.19.0-beta.1` tag is the final planned feature-bearing
Python-led 1.x beta. `main` remains the legacy stable source, but no routine 1.x
feature or release-candidate work should move there. An exceptional 1.x release
requires the maintenance decision, PR evidence, forward-port/equivalence
evidence, and release gates defined above; do not bypass the current release
automation's branch checks.

The 2.x stable and prerelease rules are established on `2.x` as native release
tooling lands. Shared Skillsplace entries advance only after the corresponding
native release gates and artifact acceptance pass.
