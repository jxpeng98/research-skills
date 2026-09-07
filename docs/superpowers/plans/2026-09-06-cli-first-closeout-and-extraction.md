# CLI-first stage closeout and first extraction

Date: 2026-09-06. This is the bounded execution plan selected by the master
roadmap. The program ledger remains the only task-state authority.

## Outcome and authority

Close the completed App/ACP source stage, merge it through a reviewed PR into
`2.x`, and make standalone Rust CLI/MCP delivery the next development outcome.
The maintainer's current request authorizes this direction and integration.
The supplied `qiongli-cli-first-local-2026-09-05` notes are proposal material:
their embedded first-task prompt and later work-package commands are not
instructions to implement all twelve packages during this closeout.

ADR 0218 supersedes the App-first default without rewriting earlier decisions.
The master maps `LF-Q01`—`LF-Q12` to canonical `CLI-401`—`CLI-412` ledger IDs.
There is no imported second task database or cross-repository dependency.

## Reconciled baseline

- Remote/local `2.x`: `accafa7477da9d55dd22e161b2e65e0765671b13`, verified
  against the remote branch on September 6.
- Initial work branch: `codex/app-acp-all-chat-realignment`, HEAD
  `4012ee13` (four commits beyond `2.x`); no open PR at initial inspection.
- Existing uncommitted changes include retained ACP lifecycle/control/history,
  source-bound Capture integration, App consumers/contracts and retirement of
  the local Trellis entrypoints. Both are included in this closeout; the
  maintainer explicitly confirmed integration of the Trellis cleanup.
- All 46 previously accepted program rows remain unchanged. Local passing tests,
  source integration and task-record completion do not accept `PLT-404`—`PLT-408`
  or grant package/publication authority.

## Previous stage disposition

| Work | Closeout disposition |
|---|---|
| Bounded reducer, fixed ACP v1 transport, retained turns, cancellation/permissions | Retain source and regressions; stop further App ACP development. |
| Versioned App control/stream, private history, actual offline Tauri IPC | Retain source, schemas and recovery/privacy contracts. |
| Selected excerpts/method, editable candidates, Capture/consolidation and digest guards | Retain existing owners and offline evidence; inspect for reuse from CLI/MCP. |
| Trellis skills/hooks/mandatory task flow removal | Integrate the existing cleanup and regression check under explicit maintainer authorization; retain specs, task history, AGENTS.md and product safety checks. |
| Real ACP authentication, isolation, resume, packaged adapters, App multi-Agent journey and user comparison | Deferred and unaccepted; no source-presence or merge-based readiness claim. |
| Research wire v2 consumer-transition gate | Remains open; retained v1 schemas/history are not silently upgraded. |

The former task is closed as a **source-stage closeout**, with its existing plan
and review retained in place. Program acceptance and GUI retirement are separate.

## CLI dependency and behavior audit

Paths below are relative to `packages/qiongli-native/` unless stated otherwise.
This inventory traces current source; it does not claim an independent CLI build.

| Entry / owner | Existing service and coupling | Next disposition / checks |
|---|---|---|
| `apps/qiongli/src/main.rs` | Empty arguments call `run_desktop_application`; `ProductAction` includes `LaunchDesktop` and a desktop candidate session. | CLI entry must show help without a window; retain explicit desktop entry and verify exit codes. |
| `apps/qiongli/Cargo.toml` | Normal `tauri`, `tauri-plugin-opener`, `rfd`, `qiongli-ui`; build `tauri-build`; test Tauri. `custom-protocol` does not separate these. | Split normal/build/dev selection and cfg boundaries together. |
| `apps/qiongli/build.rs`, `src/lib.rs` | Verified qlpack, release authority, source identity and Companion generation precede unconditional Tauri construction; MSVC manifest embedding also covers test binaries. Lib mixes exports and desktop modules. | Preserve embedded verification and exports; isolate only presentation construction. |
| `src/command.rs`, `native_cli.rs` | Help/config/doctor/project/Capture/Graph/portfolio dispatch already exists. `app snapshot` and artifact/integration observations call shared functions in `desktop.rs`. | Reuse dispatch and contracts; do not delete `app` commands merely because of their name. Existing CLI and golden tests are the comparison oracle. |
| `src/desktop.rs`, `desktop/tauri_adapter.rs` | Most DesktopService logic is shared; launch functions, adapter registration and `rfd::FileDialog` are graphical. | Retain service owner and DTO meanings; gate or move window/file-dialog code. |
| `src/managed_operation.rs`, `cli_install.rs` | Digest-bound preview/apply and lifecycle transactions reuse `desktop::verify_running_packaged_product`. | Build separation must retain current trust refusal; independent package authority is `CLI-403`, not a bypass in extraction. |
| `src/candidate_cli.rs` | Signed candidate/native preview, apply, verify and remove already exist; normal lifecycle uses managed plans. | Reuse payload/registration transactions in `CLI-403`; do not claim a consumer installer already exists. |
| `src/mcp.rs`, `orchestration_control.rs` | Lite/Full stdio and host handoff reuse runtime/execution/project owners and approval/CAS. | Preserve protocol-only stdout and existing negative tests; qualify human approval across processes in `CLI-404`. |
| `src/update_cli.rs`, `macos_update_stage.rs`, `native_update_replace.rs` | Existing package/version/rollback owners have target-specific App assumptions. | Keep current contract during extraction; independent root/version switching belongs to `CLI-403`. |
| `crates/qiongli-project`, `qiongli-execution`, `qiongli-content` | Project/Library/Capture/Graph/export/recovery, task/checkpoint candidates and locked embedded content already exist. | Reuse these owners; same-device claims/review are later `CLI-406`—`CLI-408`. |
| `crates/qiongli-ui/Cargo.toml` | Normal GUI features are disabled by the main package; GUI dev dependencies still exist for that crate's own tests. | Test the selected CLI dependency graph; do not use a whole-workspace test as proof of CLI isolation. |
| Native CI and desktop packaging scripts | Current Slice still covers the mixed product and frontend. | Keep existing required contexts for this closeout. Add CLI-specific build evidence with extraction, while preserving the desktop maintenance lane. |

Actual offline `cargo metadata --no-deps` and `cargo tree -p qiongli -e normal,build`
confirmed `tauri 2.11.5`, `tauri-build 2.6.3`, `rfd 0.17.2` and `qiongli-ui`
in the selected main-package graph. No selected independent target exists yet.

## Next single PR: CLI-402 / LF-Q02

Start after review of this audit. Produce one CLI build/entry separation using
the existing package and a narrow desktop feature unless the actual module split
requires a separate target. Keep the executable name `qiongli`; no new service
crate, storage format, provider integration or standalone installer is required
for this first slice.

Expected files: native `Cargo.toml`/`Cargo.lock` as needed;
`apps/qiongli/Cargo.toml`, `build.rs`, `src/main.rs`, `src/lib.rs`,
`src/command.rs`, `src/application.rs`, `src/desktop.rs` and
`src/desktop/tauri_adapter.rs`. Gate App-only `all_chat_control/history/research`
consumers and related examples/tests only where the compiler demonstrates the
dependency. Preserve pure DTO/schema generation and mixed service callers.
Update the existing CLI/build contract and relevant CI/build invocation owners
in the same PR; do not rename all modules or rewrite CLI commands.

Completion checks for that slice:

1. Selected CLI normal/build/test dependency graph excludes the GUI stack;
   compile/test without frontend output or GUI development libraries.
2. No-argument/help/version/JSON/MCP stdio behavior is covered by the existing
   command/integration tests plus one actual process smoke; no window launches.
3. Existing project/Capture/Graph and managed-install negative paths preserve
   semantics, especially unverified-package and missing/stale approval refusals.
4. Explicit desktop selection still compiles. Resources/Companion locks and
   release-authority checks remain in the non-GUI build steps.

Rollback is a source revert of this extraction PR with the preserved desktop
lane; no user-project migration or public asset replacement is involved.
Independent package trust/install/rollback follows in `CLI-403`; real Host
approval and the complete research journey follow in `CLI-404`/`CLI-405`.

## Closeout checks and remaining evidence

Fresh checks on the preserved source and the direction changes:

| Command / scope | Result |
|---|---|
| `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-execution -p qiongli-project --locked --offline --quiet` | 115 execution + 180 project tests passed; 1 project test explicitly ignored. Initial sandbox attempt failed only at the ACP fixture's forbidden `ps` call; the unrestricted local rerun passed. |
| `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --lib all_chat --locked --offline --quiet` | 6 passed, including actual offline Tauri IPC, history/recovery, read-view/candidate and source-drift checks. |
| `pnpm --dir packages/qiongli-app-api test` / `check` | 38 passed; TypeScript check passed. |
| `pnpm --dir packages/qiongli-desktop test` / `check` / `build` | 254 passed, 1 skipped; zero Svelte errors/warnings; production bundle 1991.4 KiB and development-fixture exclusion passed. |
| `pnpm docs:build` | Passed after replacing the roadmap's out-of-site `.trellis` relative link with the exact GitHub source permalink. The first run reproduced the preview build's dead-link error; syntax-highlighting/chunk-size warnings are non-fatal. |
| `cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check` | Passed. |
| `cargo clippy --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-execution -p qiongli --all-targets --locked --offline -- -D warnings -A clippy::chunks_exact_to_as_chunks` | Passed with the previously recorded Rust-toolchain lint exception. |
| `python3 tooling/scripts/update_program_roadmap.py --check`; architecture, public-schema and authorization validators | Passed: 249 ordered tasks, 18 current ADRs, unchanged frozen decisions and safety policies. |
| `python3 -m unittest tests.test_program_roadmap tests.test_arc_201_adrs tests.test_frozen_2x_architecture_baseline tests.test_public_schema_policy tests.test_data_lifecycle_policy tests.test_project_development_policy tests.test_authorization_policy` | 62 passed together, including the Trellis-cleanup regression check. Obsolete App-default prose assertions were updated for ADR 0218 before the passing rerun. |
| Initial-file hash comparison / accepted-row comparison / `git diff --check` | Initial retained source matched the working snapshot; the subsequent CI portability fixes are recorded below. All 46 accepted rows are unchanged; whitespace check passed. |

Review covered direction/dependency consistency, retained development-only entry
guards, approval/source-digest owners, private recovery, strict consumers and
the existing check evidence. This is not an independent human CODEOWNER approval;
the recorded `GOV-413` blocker remains unchanged. No old open PR or stage-specific
GitHub issue existed to close at inspection; unrelated roadmap epics stay open.

Required protected PR checks and the eventual merge identify their exact head
on GitHub. They remain integration evidence, not program or package acceptance.
The first full Windows Slice at `398c90be` failed before running any test:
the new Tauri mock IPC callers exposed `STATUS_ENTRYPOINT_NOT_FOUND` because
the default resource build embeds the Common Controls v6 manifest only in the
application binary. The runner image matches the successful baseline.
Following the [upstream Tauri fix](https://github.com/tauri-apps/tauri/issues/13419),
the native build now embeds the same manifest in all MSVC targets, including lib
tests, with Tauri's duplicate application-manifest emission disabled. Existing
mock IPC tests remain enabled; the fresh required Windows Slice is the runtime
regression gate. This does not establish package or release acceptance.
The local rerun passed all 6 mock IPC/history/research tests, format and affected
all-target Clippy; the XML is well-formed and matches the pinned Tauri default
byte-for-byte. The 62 policy checks passed again after this build fix.

The next Windows Slice at `6caeabab` started successfully and ran 150 app tests:
145 passed, 4 failed and 1 was ignored. Three golden comparisons differed only
by CRLF; a Windows-style Git filter reproduced changes to 15 of the 16 new
schema/golden files. The IPC test also supplied the Unix-only local origin,
which Windows correctly rejected. The fix pins these schema/fixture paths to
LF through the existing `.gitattributes` owner and derives both mock IPC origins
from their actual Webviews. Byte comparisons and production permission checks
remain strict. The required matrix must rerun against this corrected head.
After the fix, Git's `core.autocrlf=true` checkout filter preserved all 16 files
byte-for-byte; the 6 affected local tests, Rust format and whitespace checks
passed again.

The maintainer separately confirmed committing and merging the pre-existing
Trellis cleanup on September 6. AGENTS.md and CONTRIBUTING.md own the simplified
development flow; specs, task history and manual history utilities remain.
Product authorization policy, required review/check rules and their 17 tests
are unchanged. Automatic bookkeeping commits remain disabled.
Historical detailed checks remain in the old ACP implementation plan. Real model
login, private research data, native package qualification, release, tag and
announcement were not run. `CLI-401` remains an audit/integration item pending
program acceptance; the next implementation scope is the bounded `CLI-402` above.

## CLI-402 implementation — September 6

The maintainer requested the next increment after closeout PR #181 merged as
`376108eb008f40fdb2b558d50c2245d770db8879`. Work is on
`codex/cli-mcp-build-separation`. CLI-402 is active, not program-accepted.

The existing package now defaults to CLI/MCP. Optional `desktop` gates Tauri,
its build script resources, rfd, IPC adapters, and the existing thin desktop
launcher. `custom-protocol` selects `desktop`, preserving the existing App
packaging commands and desktop-enabled empty-argument launch. CLI-only empty
arguments print help; explicit UI launch fails with the existing startup error.
Pure schemas and shared service/approval owners remain compiled. The headless
picker declines selection, while explicit path-based CLI operations retain
their existing validation. No new crate, installer, storage, or service was added.

Embedded content, release authority, source identity and Companion construction
remain unconditional. Tauri test support is optional with the desktop dependency,
so selected-package CLI tests cannot pull the GUI back through dev-dependencies.
Only actual Tauri IPC portions of mixed All Chat tests are feature-gated; pure
schema, source-drift, recovery and Capture tests still run without the desktop.

Native CI now checks the CLI normal/build/dev graph and runs CLI library,
command-process and MCP-process tests before installing Tauri prerequisites or
building Svelte. Existing all-feature three-platform checks and packaged App
build commands remain. Local validation uses Rust 1.98.1 on macOS; CI retains
Rust 1.97.0. Remote Linux/Windows execution has not been performed for this work.

Fresh local validation (CLI-402 working diff):

- CLI dependency graph excludes Tauri, rfd, egui/eframe, wry/tao/winit and
  GTK/WebKit; the same assertion is in CI.
- CLI-only all-target Clippy passed with the already documented local
  `-A clippy::chunks_exact_to_as_chunks` exception.
- Explicit `custom-protocol` all-target desktop check passed.
- 79 branch/release automation and roadmap policy tests passed.
- `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --all-targets --no-default-features --locked --offline`: 267 passed, 3 explicitly ignored across 28 test binaries/examples. The ignores are manual capacity measurement and real Codex/Claude installation checks. This includes 193 library tests, 32 CLI process tests (including empty-argument help and UI refusal), and 7 copied-binary MCP stdio tests.
- `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --features custom-protocol --lib all_chat --locked --offline`: all 6 desktop IPC/history/research tests passed.
- Rust format, whitespace, generated roadmap checks passed; all 46 accepted ledger rows are unchanged.

Final diff review covered both feature selections, desktop launcher requirements,
schema/test availability, unconditional trusted resource construction and the
CI step order. The source default changes from GUI to CLI; downstream desktop
builders must select `desktop` or `custom-protocol`. Existing supported
packaging invocations already select the latter. No public schema or
approval/CAS behavior changed. This is local implementation evidence; no new
commit, push, merge, package publication or remote CI result is claimed.

The next integration boundary is review and exact-head CI for CLI-402.
CLI-403 remains the separate trusted package/install/rollback increment; real
Host authorization and research journeys remain CLI-404/405. Compilation and
local tests do not supply their acceptance evidence.

## CLI-402 integration — September 7

The maintainer requested the next step: commit the reviewed CLI-402 change,
open a PR targeting `2.x`, and run its exact-head required CI. The source
comparison remains `376108eb008f40fdb2b558d50c2245d770db8879`; no upstream
changes or additional working-tree scope appeared at the pre-commit check.
The PR records the frozen head and live check evidence. Program acceptance,
merge and release remain separate from this integration check.

## CLI-403 first increment — installed CLI package baseline

PR #182 was reviewed with no actionable findings and merged to `2.x` as
`c25fb2c1171f43455223eb7409aad8b1ec41a187` on September 7. Its tested head
`7dafcbcde3948e031e0c5dcdb4cd62333498fda9` passed Native CI `34109163479`
and Evaluation Truth `34109146661`. This closes its source integration,
without inferring program or release acceptance.

The next bounded increment reuses `native_candidate_acceptance.rs`, native
artifact/archive composition, signed candidate verification and managed payload
transactions. Its build explicitly selects no default features. It now checks
help/window refusal on the extracted CLI and runs CLI/Lite MCP directly from each
verified installed payload, including a second MCP process after shutdown.
All runtime processes keep the existing empty PATH and isolated homes outside
the checkout. Existing digest/partial-approval/conflict refusals, compensation,
uninstall and user-data canaries remain.

This is development verification with ephemeral test signing keys, not a public
package or real Host qualification. The candidate is a development working-tree build based on the merge above;
its source label is the merge commit, with the acceptance harness changes and
a cfg-only unused-import correction on `codex/cli-native-package-baseline`.
It is not an immutable release-source qualification.

Remaining CLI-403 scope: standalone product authority for managed operations
still assumes a Desktop manifest/App binding; define its legitimate independent
identity before extending that verifier. Qualify update/version switching,
rollback and live-process pinning separately. Source builds must remain
inspection-only where product authority is required. No `verified=true` shortcut
or new installer has been added. CLI-404/405 real human Host approval and research
journeys remain later work.

Fresh local validation on macOS:
- The existing `native_candidate_acceptance` runner completed with
  `candidate-acceptance-passed`, using an isolated private output directory,
  ephemeral in-memory signing keys and no external clients.
- All 21 reported checks passed, including the four added CLI/installed-runtime
  checks. `publication_allowed=false`; real Codex/Claude, displayed window and
  production signing gates remain `not-run`.
- Evidence: `<external-output>/acceptance-evidence.json`, SHA-256
  `c2d781b7495ec29effa78f2dbfadf4380c6308e7f57b58f2244e63aad221b38a`.
- Five harness tests and 59 roadmap/release-automation tests passed.
- The acceptance-harness Clippy check passed with the existing local Rust 1.98
  `chunks_exact_to_as_chunks` exception; the corrected release build completed
  without the unused `ResearchContext` import warning.
- The first fixture run under `/private/tmp` was correctly refused because
  explicitly approved installation targets reject writable ancestors. The
  successful run used a user-owned private directory; no path validation was
  relaxed. This is distinct from the passed package/runtime checks.
- Linux/Windows execution of this new harness increment has not run; the
  preceding CLI-402 CI is not reused as evidence for this changed harness.

Integration follows the maintainer's updated local workflow: reviewed scoped
commit on `codex/cli-native-package-baseline`, then fast-forward into local
`2.x`. The frozen-source guard passed against `2.x`; unchanged focused results
above are reused. No remote synchronization or release is part of this increment.
CLI-403 remains active; the next increment is independent product authority,
reusing signed native artifact verification without weakening source-build refusal.


## Development-flow simplification — September 7

The maintainer requested faster App-to-CLI extraction with fewer tests, gates
and manual confirmations. This changes delivery tooling, not product authority
or accepted program rows. Existing CLI-403 source/evidence edits are preserved.

The live `2.x` ruleset has five required contexts and zero required approving
reviews; an impossible independent human review is not an integration gate.
Native CI already has no merge-push trigger. The duplicate post-merge Evaluation
Truth run at `c25fb2c1` took 13 seconds; the main excess is the ready-PR matrix:
CLI tests plus desktop setup, check, Clippy, all-feature tests and a duplicated
mobility test on each platform, plus Lite compatibility even for app-only edits.

The revised loop is edit → one affected check → ready PR → current-head CI →
authorized merge → next increment. Reuse unchanged local results and one scoped
integration instruction. Remove mandatory draft/freeze and per-stage approval
ceremonies, duplicate evidence fields, and Markdown layout validation. Preserve
negative tests, protected checks, truthful review state and publication scope.

CI now runs headless workspace tests on three systems (excluding GUI test
roots), Linux-only format/CLI Clippy, affected Linux desktop consumers, and
Lite compatibility only for Lite, its shared runtime dependencies or unknown/
tooling inputs. Shared native
source/build changes conservatively select desktop consumers; dedicated CLI/MCP
paths skip the renderer. Remove the redundant `cargo check`, standalone mobility
rerun and merge-push Evaluation Truth. Full desktop matrices, capacity, packages
and real Hosts remain candidate work. No remote ruleset changes are needed.

Validation:
- 55 focused flow/evaluation/authorization tests passed; affected classifier and
  authorization checks were rerun after the final routing/document edits.
- Shell syntax, YAML/trigger parsing, authorization validation, generated roadmap
  consistency and whitespace checks passed. Required remote contexts are unchanged.
- The selected headless workspace graph contains no GUI dependencies, and every
  selected test target compiled on macOS without frontend setup (50.16 seconds).
- Runtime execution reported 242 passes and 3 explicit ignores before stopping
  at the MCP fixture's sandbox-denied `TcpListener::bind("127.0.0.1:0")`. The
  focused unsandboxed `--test mcp_stdio --no-default-features --locked --offline`
  rerun passed all 7 tests. Later workspace test binaries were not run; this is
  not a full local workspace pass.
- Remote Linux/Windows execution and elapsed-time savings are unverified for
  this diff. macOS/Windows desktop regressions now surface at candidate validation
  rather than every PR; three-platform CLI regressions remain required.

No commit, push, merge, remote ruleset mutation, package or publication was
performed for this workflow change.
Next increment remains CLI-403 independent package authority/install/rollback;
its external acceptance does not block independent CLI development.


## Local integration correction — September 7

The maintainer accepted the simplification and explicitly selected development
on local feature branches followed by direct local merge into `2.x`, with no PR.
This supersedes the ready-PR requirement in the preceding audit. The current
instruction authorizes scoped local branch creation, commits and merges; no
push, remote ruleset change or publication is included.

Default loop: local feature branch → edit → affected check → reviewed commit →
`git merge --ff-only` into local `2.x` → next increment. If the base advances,
merge it into the feature branch and check only the affected combined behavior.
Run the existing fast frozen-source guard before merge. No new wrapper, task
engine, approval receipt ceremony or CI job is needed.

Local Git integration uses affected checks and a reviewed diff under the
maintainer's instruction. The versioned authorization policy still governs
protected remote merge with its existing PR/CODEOWNER evidence; local merge does
not authorize that action. Research/publication boundaries and the remote ruleset
snapshot are unchanged. Optional PR workflows
continue to exist for explicitly requested remote collaboration or candidates.
The preceding CLI-403 working changes remain outside this workflow commit.

Validation: 40 focused authorization, branch and development-policy tests passed;
authorization policy, generated roadmap and whitespace validation passed. Existing
CI/classifier evidence from the preceding audit is reused; no Rust code changes
are part of this increment, so the native suites are not repeated.

Integration branch: `codex/local-first-development`; target: local `2.x` through
fast-forward merge after the frozen-source guard. The isolated worktree contains
only workflow/policy changes. CLI-403 source, ledger updates and evidence remain
uncommitted in the original worktree. Remote rules, push and publication are
outside this action. Next development remains the CLI-403 package boundary.

## CLI-403 second increment — extracted payload release verification

Base: local `2.x` at `dd4bc808`; branch: `codex/cli-extracted-release-trust`.
The native release owner can now verify an approved extracted artifact directory
without its original archive. `verify_extracted_artifact` reuses the existing
release signature/key/generation/time checks and launch-grant verifier. The native
artifact owner revalidates the actual file tree; its manifest, binary and resource
digests must match the signed envelope and caller's expected artifact identity.
The archive verifier retains its archive-byte checks and shares the trust checks.

This is the bounded verification primitive for independent CLI trust. It returns
the existing scoped launch grant, not a forged Desktop product or write approval.
No wire schema, installer, dependency or persisted receipt format changes. Existing
source-build restrictions remain. Next: preserve and freshly verify signed
candidate provenance at the installed root, bind the running executable, then
connect the existing managed-operation owner. Updates/rollback remain later scope.

Validation on macOS:
- `cargo test ... -p qiongli --test native_portable_archive --locked --offline`:
  1 integration test passed (135.48 seconds). Existing archive/installation and
  CLI/MCP checks passed alongside the new checks with both archives moved away,
  untrusted key, invalid release/launch signatures, early/expired/stale releases,
  stale launch grant, wrong scope/channel/version, signed manifest mismatch and
  binary tampering. Restoring the binary passed a fresh verification.
- `cargo test ... -p qiongli-platform --lib native_release::tests --locked --offline`:
  3 tests passed. Affected integration-target Clippy passed with the existing
  local Rust 1.98 `chunks_exact_to_as_chunks` exception.
- Formatting, whitespace and generated roadmap consistency passed. Final review
  against local `2.x` found no actionable findings. Linux/Windows execution,
  production signing, running-process authority and real Hosts are not qualified.

Integrate the scoped commit locally through the frozen-source guard and fast-forward
merge. Reuse these results; no remote synchronization or publication. CLI-403 and
all accepted ledger rows retain their existing status.

## Continuing objective — complete the first-stage integration

The maintainer requested continued work through the first-stage integration.
This means the standalone CLI baseline in ADR 0218 and CLI-401 through CLI-405,
with CLI-410 qualification preparation: independent build/resources/install/trust,
window-free Host integration and verified human approval, a controlled research
write, restart/recovery and same-device handoff. Integration is local `2.x` under
the current workflow. Publication and additional collaboration remain separately
scoped; passing primitives does not complete this objective.

Outstanding evidence includes independent product-operation routing, update and
rollback, human approval without a Qiongli window, the real Host journey and the
declared package/platform support scope. Existing implementation and accepted
evidence will be reused only where their scope matches these requirements.

## CLI-403 third increment — persisted candidate identity

Base: `307fd56b`; branch: `codex/cli-installed-candidate-identity`.
Candidate installation now stores the exact canonical signed candidate beside
the payload directories, through existing private-file, sync and atomic no-replace
rename helpers. Existing conflicting records refuse before payload changes;
identical records replay. A final persistence failure uses existing installation
compensation. Interrupted temporary metadata cannot confer product authority.
Signed metadata remains with diagnostic receipts after uninstall.

`verify_installed_native_candidate_product` discovers the fixed managed root,
requires an active payload receipt and revalidates the signed candidate, source
commit, platform/version, portable release and requested Host grant against current
trust/time. The supplied process path must resolve to the verified installed binary;
an identical copy elsewhere is refused. Archive and release-note bytes remain
installation qualification inputs, not runtime dependencies. Receipt and signed
release digests must agree. The returned capability is private-constructed and its
debug output redacts the executable path; it does not carry write approval.

The existing receipt schemas and integrity-only diagnose/remove path remain
compatible. Legacy installs without signed metadata refuse this new authority path
until a fresh verified candidate apply replays the installation. App-backed product
operation routing is the next increment; this source change does not yet make those
operations standalone or prove a running release binary/real Host journey.

Checks on macOS: the existing `native_release_candidate` integration test passed
(1.74 seconds), including installed metadata, legacy replay, tampering/hard-link
refusal, recovery journal refusal, uninstall invalidation, both Host scopes,
wrong source/signature/key/time/channel/generation and identical-copy path refusal.
Three platform candidate parser/identity tests passed; affected-target Clippy passed
with the existing local Rust 1.98 `chunks_exact_to_as_chunks` exception. Formatting
and whitespace checks passed. Final review found no actionable findings; native
Linux/Windows, crash injection and actual product-entry qualification remain open.
Integrate locally after the frozen-source guard, reusing these checks. CLI-403 and
accepted ledger status remain unchanged; no publication or remote synchronization.

## CLI-403 fourth increment — independent product operation routing

Base: `a92820aa`; branch: `codex/cli-product-operation-authority`.
The running-product boundary now routes executables in the fixed native payload
root through signed installed-candidate verification with embedded source/version,
authority and current time. Both signed Host capabilities must verify against the
same candidate digest and executable. Other paths retain desktop/managed-shim
verification and source builds retain their existing refusal.

`VerifiedPackagedProduct` now exposes verified artifact, source and resource facts
instead of retaining a desktop manifest/control document. Both delivery verifiers
construct those facts after their own checks. Existing installation, reconciliation
and migration consumers use the shared accessors; no fake Desktop identity, wire
schema changes or new operation engine. Native plan identity binds the signed
candidate preimage digest; desktop control digest semantics remain unchanged.

Validation on macOS:
- Candidate integration test passed (1.60 seconds), including native product
  capabilities and existing operation-owner preview/apply with wrong-digest refusal.
- Eight packaged-product regressions and eight managed-operation regressions passed,
  including changed-state refusal, source-build refusal, digest/approval/expiry,
  canonical contracts and user-data preservation. Affected-target Clippy passed
  with the existing local Rust 1.98 `chunks_exact_to_as_chunks` exception.
- The actual `native_candidate_acceptance` runner passed using ephemeral signing,
  empty PATH and isolated homes outside the checkout. For both target directories,
  the installed CLI generated an integration-removal plan, refused missing approval,
  applied the approved plan, and restored installation before the existing full
  candidate uninstall check. All 22 reported checks passed.
- Output: `/Users/pengjiaxin/Work/qiongli-cli403-product-authority-20260907`;
  `acceptance-evidence.json` SHA-256:
  `0c12f27f262c0882ba83822f4ef058270d3db08806f1d897f2386d59a434f6cd`.
  Source label is `a92820aaccfa443eb55955b2a1cf76d9cbc80e0c` plus this working
  diff, not an immutable release candidate. `publication_allowed=false`;
  real Hosts, production signing and Linux/Windows runtime remain unqualified.
- Formatting, whitespace and generated index checks passed. Review found no
  actionable findings. Local integration reuses these results.

Next: standalone CLI shim/PATH installation and update/rollback compatibility,
then CLI-404 trusted human approval and CLI-405 real research/recovery evidence.
The first-stage objective and CLI-403 remain active; local merge does not accept
the program or authorize publication.

## CLI-403 fifth increment — native command install and PATH

Base: `c38c1e40`; branch: `codex/cli-native-command-install`.
Native payload executables now resolve as the CLI install source instead of an
imagined sibling `qiongli-cli`. The existing copy/install, backup, remove and PATH
owners and v3 receipt format are reused. For a command copy at the fixed user bin
path, the receipt version and digest must match the fixed native artifact derived
from embedded version/channel. The running-product owner then freshly verifies
that source's active installation and signed candidate. The path/digest resolver
itself grants no authority. Legacy receipts and Desktop bindings retain their
previous routing; there is no extra receipt or unsigned trust flag.

Checks on macOS:
- All 21 CLI installation/PATH tests passed. The native resolver test additionally
  passed after adding legacy-receipt refusal, alongside changed command/source,
  arbitrary copy and missing-receipt checks. Source-process authority refusal passed.
- Actual candidate acceptance passed in
  `/Users/pengjiaxin/Work/qiongli-cli403-command-install-final-20260907`.
  Both isolated target homes installed the command through `app plan/apply`, ran
  CLI/MCP and managed integration removal from the command copy, configured Bash
  PATH, launched `qiongli --version` in a fresh login shell, and removed the owned
  command. Shell-profile and existing user-state canaries were preserved.
- Evidence SHA-256:
  `2e7edbaacf372fd0384fef00860f97007d01a46a67fae507ebcd64a2b639779a`.
  The source label is `c38c1e4087de0ea2e084c5c47abc99b121d69e39` plus this
  working diff. Ephemeral signing and `publication_allowed=false` remain explicit;
  this is not immutable release qualification. Windows PATH, native Linux/Windows
  execution and real Hosts remain unqualified.
- The first candidate run refused the fixture's extra Host approval flags on a
  filesystem-only CLI operation (`managed-operation-approval-unexpected`). The
  fixture now supplies exactly the operation's approvals; product validation was
  retained. Final Clippy, formatting, whitespace and index checks passed.

Review found no actionable findings. Integrate locally after the frozen-source
guard and reuse these results. Next: native package update/rollback and active
process/version consistency, followed by CLI-404/405 qualification. First-stage
integration remains incomplete; ledger acceptance is unchanged.

## CLI-403 sixth increment — side-by-side candidate staging

Base: `afb7d5d7`; branch: `codex/cli-candidate-version-staging`.
Complete candidate apply previously coupled payload installation to fixed Host
source installation, preventing a newer version from being staged beside an
existing integration. The new stage owner shares payload preparation and immutable
signed-record persistence with apply. It writes no Host source/registration or
installed command. Complete apply keeps its existing commit and compensation order;
stage persistence failure compensates only its fresh payload.

The existing candidate integration test now composes two independently signed
version identities, stages the newer payload, verifies its installed identity and
compares the older integration's full receipt closure unchanged. Replay returns
AlreadyApplied. Existing payload rollback removes the stage and invalidates its
product authority while the prior product and integration still verify. Expired
staging refuses; rollback refuses modified staged bytes and preserves that canary.

Validation on macOS: the expanded candidate integration test passed (1.85 seconds),
including prior installation conflict/compensation cases. Affected Clippy passed
with the existing local Rust 1.98 `chunks_exact_to_as_chunks` exception; formatting
and whitespace checks passed. Review found no actionable findings. These are
synthetic version identities using the small existing test binary, not actual
two-version executable or live-process qualification. No release fixture suite is
repeated for this shared-owner extraction.

Next: wire digest-bound staging into the CLI, then activate/reconcile through the
existing owners with recovery and current-process/version checks. Current candidate
verification expects the running build's source/version/content; support for newer
downloaded content must be explicit rather than bypassing that check. Active-version
rollback, cross-platform runtime and CLI-404/405 acceptance remain open. Local
integration does not complete the first-stage objective or change accepted rows.

## CLI-403 seventh increment — digest-bound CLI staging

Base: `9bc93bca`; branch: `codex/cli-candidate-stage-command`.
`install candidate stage-preview` and `stage` now expose the existing payload-only
stage owner. Stage re-verifies the signed candidate and requires its own
candidate/target-bound digest plus exactly filesystem-write approval. Full-install
and stage digests are mutually unusable; Host approval flags are rejected. Source
builds retain authority refusal. No Host source, registration or installed command
is changed. The existing candidate commands keep their JSON contracts.

The new stage JSON v1 has a Rust-generated Draft 2020-12 schema and three
Rust-produced golden fixtures, with an additive public-schema policy record.
The native README documents commands, generation and the current-build limitation.

Validation on macOS:
- Three focused Rust tests passed for approval parsing, digest separation and
  generated schema/fixture consistency; the expanded source-build CLI refusal
  test passed. Affected-target Clippy passed with the existing local Rust 1.98
  `chunks_exact_to_as_chunks` exception.
- Public-schema validation and its 12 tests passed; generated roadmap index and
  whitespace checks passed. Final diff review found no actionable findings.
- The actual `native_candidate_acceptance` runner passed with 26 evidence fields,
  including stage preview, missing-approval refusal, both directions of digest
  misuse, successful stage and replay for both target configuration fixtures.
  It checked absence of Host/command writes and retained the prior lifecycle checks.
  Evidence: `/Users/pengjiaxin/Work/qiongli-cli403-stage-command-20260907/acceptance-evidence.json`;
  SHA-256 `fe8fcf577e2bd3379c181fb174e42df602a6f847db81ce86505ad2a8a20642b9`.
  Build source label is `9bc93bca77b81bb0bc3406cf00b209b5354e8e07` plus this working
  implementation, not an immutable release candidate. Signing keys were ephemeral,
  PATH was empty and homes were isolated; no real Host ran or publication occurred.

Next: activate/reconcile a staged version through existing owners, with recovery,
current-process/version checks and rollback. Stage still verifies the running
build's source/version/content; newer downloaded content needs explicit trusted
verification. Windows/Linux runtime, CLI-404/405 and first-stage acceptance remain
open. The ledger's accepted rows are unchanged.

## CLI-403 eighth increment — shared rollback preflight

Base: `15d3c2cc`; branch: `codex/cli-reconciliation-rollback-guards`.
Tracing staged-version activation found that the reusable reconciliation rollback
moved active files before checking its old backups. The regression reproduced this:
a damaged registry backup returned an error only after another managed surface's
inode had changed. Before wiring standalone activation to this owner, rollback now
validates the journal and every operation's old/new identity before the first
rename. Automatic activation compensation uses the same implementation, avoiding
a second rollback path. Public JSON and persisted journal versions are unchanged.

The existing real-materialization test now checks corrupted backup and active
bytes, a dangling backup link in the final reverse-order operation, exact inode
and canary preservation on refusal, both interrupted rename states, and replay.
On macOS the reconciliation tests passed (2), direct replacement-owner tests
passed (10), and affected library Clippy passed with the existing Rust 1.98
`chunks_exact_to_as_chunks` exception. The earlier REL-913 selection also passed
(4); unchanged results were not repeated at merge. Formatting and whitespace
checks passed. Final review found no actionable findings in this bounded fix.

Next remains actual independent version activation and rollback across installed
command and Host surfaces using existing reconciliation/CLI owners. The current
reconciliation journal covers Skills, plugin sources and registration, not the
installed CLI binary/receipt; CLI replacement currently has its own transaction.
Their coordinated recovery and current-process checks need implementation and
real versioned execution evidence. This prerequisite fix is not qualification of
that missing flow, CLI-404/405, additional platforms or first-stage acceptance.
Ledger acceptance is unchanged; no remote operation or publication is included.

## CLI-403 ninth increment — CLI replacement receipt CAS

Base: `822e1489`; branch: `codex/cli-install-receipt-cas`.
The CLI replacement owner checked source and target bytes but did not bind the
previous installation receipt. A targeted regression reproduced a successful
replacement after that receipt's version changed. Preview now includes the exact
validated receipt byte digest (or absence) in its private native plan digest.
Apply re-observes it and the retained predecessor backup before any filesystem
mutation. The same receipt read supplies both decoded fields and byte identity.
All callers, including managed CLI plans, use this shared check. Persisted receipt
and public JSON shapes are unchanged; older approval digests require a new preview.

Checks on macOS: all 22 CLI install tests and all 8 managed-operation tests passed;
library Clippy passed with the existing Rust 1.98 exception. The final regression
also passed with receipt modification, deletion, appearance, backup tampering,
unchanged old command/receipt/canary assertions and successful replacement after
restoring the exact preview state. Formatting and whitespace checks passed. Review
found no actionable findings in this scope. No package qualification suite was
repeated for this owner-only fix.

The next implementation remains coordinated CLI binary/receipt and Host-surface
activation, crash recovery and active-version rollback. The existing reconciliation
journal does not yet include the CLI pair; its schema must evolve explicitly if
those surfaces are added, retaining old journal recovery. This preflight CAS check
is not a cross-process lock or complete multi-file transaction and does not claim
those guarantees. First-stage and program acceptance remain open and unchanged.

## CLI-403 tenth increment — CLI pair in the shared recovery journal

Base: `e61068e0`; branch: `codex/cli-reconciliation-pair`.
The reconciliation owner can now prepare an installed managed CLI binary and its
receipt as two linked operations, alongside its existing content/registration
operations. Preparation reuses the CLI owner's exact preflight and receipt
construction; direct CLI install retains its behavior. The caller supplies a
verified predecessor pack identity, and the journal binds both receipt hashes,
binary hashes and versions. Partial or mismatched pairs cannot activate. Journal
v2 requires the pair, while existing callers continue creating v1 and existing v1
operation/registration digest domains remain unchanged. Reusing a prepared journal
with a different target version, pack or CLI identity refuses.

Checks on macOS: CLI install tests passed (22); reconciliation tests passed (3),
including an actual file-pair activation, recovery after only the binary committed,
receipt-drift refusal, rollback/replay, and v1/future/incomplete-pair refusal.
The existing real-materialization test now includes the CLI pair with Skills and
registry state, verifies canonical non-empty v1 compatibility, and restores every
old surface plus unrelated canaries. Direct replacement-owner tests passed (10).
Clippy initially requested a collapsed condition; the version/pair check was
simplified to a slice match, then its focused test and Clippy passed with the
existing Rust 1.98 exception. Format and whitespace checks passed. Final review
found no actionable issue in this internal capability. Fixtures use small CLI
byte payloads, not two published or production-signed executable versions.

Next: expose a digest-bound standalone activation/recovery flow that supplies this
optional CLI plan and retains one transaction outcome for the chosen Host surfaces.
Existing integration commands intentionally pass no CLI update: their approvals
do not authorize silently adding command replacement. A transaction lock, durable
outcome/health handling and current-process/version checks remain required before
calling this an end-user update/rollback flow. Real Host/human approval, CLI-405,
platform qualification and first-stage acceptance remain open. No accepted ledger
row, public CLI JSON or release/publication claim changed.

## CLI-403 eleventh increment — resumable reconciliation cleanup

Base: `206dea01`; branch: `codex/cli-reconciliation-cleanup-recovery`.
Tracing the durable activation outcome exposed a missing recovery case: committed
cleanup required all old backups, so an interruption after deleting one prevented
replay. Rolled-back cleanup likewise required every staged file and recursively
removed its whole staging container. A regression reproduced deletion of an extra
file in that container without an error.

Both outcomes now use one cleanup implementation: validate the whole journal,
retained destinations and every remaining owned cleanup target before deletion;
accept already removed cleanup targets; reject unexpected container entries;
remove containers only when empty. Discard uses this same resumable path and keeps
its journal on failure. Missing journal replay still succeeds, but a dangling
journal link is not treated as absence. Existing journal versions are unchanged.

On macOS, reconciliation tests (3), direct replacement-owner tests (10), formatting
and library Clippy passed with the existing Rust 1.98 exception. The final focused
case also exercised an actual private persisted journal through load/discard,
confirmed that an extra-file refusal retains the journal and other staged files,
then resumed after one staged file had been deleted. Committed cleanup resumed
after one backup deletion; both modes replayed successfully with correct active
CLI/receipt bytes. The initial failing check demonstrated the extra-file deletion;
final review and whitespace checks found no actionable issue in this scope.

Next remains the standalone activation/recovery coordinator and its approved CLI
entry point: reserve the existing update state, hold the replacement lock, persist
the journal/outcome, activate the CLI/content set, then recover or finish cleanup
from that durable outcome. Neither this cleanup fix nor the prior file-pair owner
constitutes that complete command flow. Process/version checks, real signed-version
execution, Host approval/research journey and first-stage acceptance remain open;
no ledger acceptance or remote/publication state changed.

## CLI-403 twelfth increment — persistent activation and recovery coordinator

Base: `f8f8560a`; branch: `codex/cli-native-activation-transaction`.
The native reconciliation library now activates an approved v2 journal under the
existing replacement lock and update-state reservation. The macOS helper uses the
same relocated lock owner. Activation rechecks the exact journal digest under the
lock, writes an immutable start record, reserves the transaction, activates the
linked surfaces and runs its caller-supplied health check. Failed health restores
the old surfaces; success records a committed decision before cleanup. Recovery
uses the persisted decision and never reruns activation or health. It retains the
journal and outcome after clearing the reservation, allowing idempotent replay.
Other active transactions, wrong digests, malformed records and attempted fresh
activation of an existing transaction refuse.

Validation on macOS: reconciliation tests passed (3), direct macOS replacement
owner tests passed (10), and Clippy passed with the existing Rust 1.98 exception.
The expanded file-pair case exercised failed health, panic/unwind during health
with a durable active reservation, successful activation, and committed cleanup
failure caused by an extra file. Recovery restored undecided state, finished
committed cleanup and replayed without incrementing state revision. It also checked
lock contention, a foreign active transaction and malformed persisted start data
without overwriting those states. Final additions reran only that focused case and
Clippy; format and whitespace checks passed. Review found no actionable issue in
this bounded coordinator. Panic/unwind is an interruption fixture, not evidence of
SIGKILL or a real cross-process signed-version health journey.

Next: connect the command boundary to signed candidate/predecessor verification,
preview/approval and this coordinator, with real child-process health, current
process/version checks and exclusion of competing managed writes. Existing user
commands remain unchanged. The coordinator deliberately preserves release-generation
and last-known-good package fields; committing those requires the signed release
identity in the next command-level integration. Native Windows update-state support,
real Host approval/research recovery, named candidate qualification and first-stage
acceptance remain open. No accepted ledger row or publication state changed.

## CLI-403 thirteenth increment — real installed-process health

Base: `75235fa0`; branch: `codex/cli-native-process-health`.
`check_native_cli_health` checks the installed command hash against an already
verified candidate, starts that exact managed command with empty PATH and explicit
home/config roots, and validates its existing CLI-install plan through the managed
plan type. Version, resource pack and candidate control digest must match; the
command hash is checked again afterward. Process timeout/output bounds reuse the
existing Host-command executor. Existing Host environment construction is unchanged;
child failures expose static reason codes. Public CLI JSON remains unchanged.

Checks on macOS: the new identity-plan test and existing bounded-command failure
test passed; library and candidate-example Clippy passed with the existing local
Rust 1.98 exception. Extraction initially left the output limit constant in its old
scope, and the example initially shadowed its clock function; both compilation
issues were fixed before validation. Formatting, whitespace and generated roadmap
index checks passed. Final review found no actionable issue in this scope.

The actual `native_candidate_acceptance` runner passed with 27 evidence fields.
Both isolated target fixtures executed healthy installed CLI children; tampering
the command caused refusal before execution and preserved the tampered canary.
Restoring exact bytes allowed the remaining lifecycle checks to complete. Evidence:
`/Users/pengjiaxin/Work/qiongli-cli403-process-health-20260907/acceptance-evidence.json`;
SHA-256 `b4ef9acf3c0b2d90befbd5553208bfc2a04be0f52b64c16509960113506b6b2d`.
The build source label is `75235fa006cb641e52e225577fb610e207487a79` plus this working
implementation, using ephemeral signing keys. It is not an immutable release
candidate. Real Host checks were not run; `publication_allowed=false`.

Next: connect candidate/predecessor preview and explicit approvals to the persistent
activation coordinator with this health callback; bind the signed release outcome
and last-known-good metadata, add process/version and competing-write guards, and
expose recovery. This health function validates the current candidate build and
native startup; it does not establish cross-version update selection, real Host
activation, research writes or cross-platform qualification. First-stage acceptance
and ledger acceptance remain open and unchanged.

## CLI-403 fourteenth increment — receipt-owned predecessor integrity

Base: `9f877607`; branch: `codex/cli-predecessor-payload-verification`.
A successor's embedded resource pack cannot validate a predecessor's different
pack. The payload executor now exposes a read-only receipt-owned check that reuses
canonical manifest parsing, strict artifact tree validation, binary hashing and
exact receipt bindings. Existing install, repair, lifecycle and signed-product
paths retain their pack-bound checks. No public CLI or persisted schema changed.
This check supplies integrity evidence only, never launch or write authority.

Focused macOS checks passed: 22 platform native tests, then the expanded predecessor
case with pending-journal and symlink negatives. The case also covers distinct
resource packs, changed binary bytes, foreign files, mismatched manifest receipts,
linked receipts and removed payloads while preserving refused state. Platform
library Clippy passed with the existing local Rust 1.98 exception; formatting,
whitespace and generated roadmap index checks passed. A test initially called the
binary-path helper with the OS instead of the artifact; fixed before passing runs.
The affected artifact (2), portable archive (1), and signed candidate (1)
integration tests also passed. Final review found no actionable issue in this
bounded integrity change.

Next: bind the predecessor payload receipt to the existing Host and CLI receipts
in the candidate activation preview, then connect explicit approvals, signed
outcome/last-known-good metadata, process and competing-write guards, and recovery.
The current check is not a cross-version executable health journey or activation
command. Real Hosts, Windows runtime and named candidate qualification remain
unverified. First-stage and ledger acceptance remain open; no publication occurred.


## CLI-403 fifteenth increment — activation identity preview

Base: `2af0d211`; branch: `codex/cli-activation-preflight`.
The release-engineering `install candidate activate-preview` command now requires
signed candidate/archive/notes, a selected Host and `--previous-install-id`. It
verifies the running staged product against the exact candidate, joins the existing
payload/source/registration receipt closure with the managed native CLI copy, and
requires a newer version in the same artifact stream/platform. Its snapshot digest
binds the candidate, target, CLI plan and predecessor receipts. CLI preconditions
are rechecked after predecessor discovery. No transaction, mutation or approval is
created. The platform closure helper reuses existing validation; pack-bound callers
retain their original behavior.

Checks on macOS: activation parser/contract tests passed (2); the signed-candidate
integration fixture passed (1), including old/new identity mismatch, same-version
refusal, missing/changed command, mismatched CLI receipt and exact receipt-byte
binding. The real source-build CLI refusal test passed (1), preserving static errors
and path redaction. The integration fixture uses small synthetic executable bytes
and version identities, not a cross-version running-binary qualification. Library
and generator Clippy passed with the existing Rust 1.98 exception. Public schema
policy validation and its 12 tests passed. Draft 2020-12 schema and fixture are
Rust-generated with an additive policy record. Final diff review found no actionable
issue; format, whitespace and generated-roadmap checks passed.

Next: prepare the complete activation transaction, including managed Skills and
settings state; obtain digest-bound write/config/Host approvals; connect signed
release generation/last-known-good metadata, competing-write exclusion and recovery
to the existing coordinator and process health check. The new preflight digest is
explicitly not activation authorization. Successful packaged command execution,
real Hosts and platform qualification remain outstanding. First-stage integration
and program acceptance remain incomplete; no accepted ledger or publication state
changed.


## CLI-403 sixteenth increment — approved activation preparation

Base: `3e781644`; branch: `codex/cli-activation-prepare`.
`install candidate activate-prepare` joins the preflight to the existing v2
reconciliation owner. Signed release inputs, the previous install ID, exact preflight
digest and filesystem-write approval are required. State-root creation uses
`GlobalSettingsStore`. Preparation checks active transactions/release generation,
rechecks the preflight under the replacement lock, stages registered Skills,
selected Host sources/receipts and the CLI pair, then rechecks workflow/update state.
The output lists actual surfaces and a separate activation approval digest binding
the journal, candidate, preflight and update/workflow revisions. It grants no
activation and does not reserve or alter active update state. Existing transaction
roots refuse without replacement; failed fresh preparation uses guarded cleanup.

Checks on macOS: 7 affected activation/parser/schema/reconciliation unit tests
passed, the signed-candidate integration test passed, and the source-build CLI
refusal test passed. The integration case staged four Codex/CLI operations, preserved
old command/Host identities and update state, rejected incorrect preview digests,
kept an existing journal on retry, and refused foreign active transactions and an
already-accepted release generation. Existing reconciliation unit coverage includes
registered Skills and CLI-pair rollback. This new integration fixture uses synthetic
binary bytes and does not execute a packaged cross-version CLI command or live Host.

First-run testing exposed missing configuration ancestors; using the existing
settings owner fixed the production path. The manually created CLI receipt fixture
also needed the real owner's private file mode. A later test passed an unnecessary
borrow to `UpdateStateStore::replace`; fixed before the final passing run. Temporary
diagnostics were removed. Library/generator Clippy passed with the existing Rust
1.98 exception; public schema validation and 12 policy tests passed. The new Draft
2020-12 schema and golden fixture are Rust-generated and recorded as additive.
Formatting, whitespace and generated-roadmap checks passed; final review found no
remaining actionable issue in the preparation scope.

Next: expose guarded discard for an unactivated preparation; bind signed release
outcome and last-known-good metadata, exclude competing managed writers, and connect
actual activation/recovery commands to the coordinator and process health check.
The prepared approval digest must be revalidated against the same journal and
state revisions before any live rename. Cross-platform state support, real Host
research/approval and named package qualification remain open. First-stage
integration and ledger acceptance remain incomplete; no publication occurred.


## CLI-403 seventeenth increment — cancel an unactivated preparation

Base: `473e4a5a`; branch: `codex/cli-activation-discard`.
`install candidate activate-discard` requires an exact transaction/journal digest and
filesystem-write approval. It uses the replacement lock and existing whole-set
cleanup owner, refusing active update state, activation records, backup files and
foreign transaction-root entries. Remaining staged files can be cleaned after an
interrupted deletion. Cleanup uses the already checked journal; the digest is
rechecked before removing its file and empty root. A missing journal refuses and
cannot be mistaken for successful cancellation. This command needs no fresh release
authority, executes no candidate binary and never switches installed destinations.

The shared journal reader now verifies private, non-linked state/update/staging/
transaction directories, reusing the existing directory checks. The new CLI output
has a Rust-generated Draft 2020-12 schema, golden fixture and additive policy record;
existing reconciliation and activation record formats remain unchanged.

Validation on macOS: 9 activation/parser/contract/reconciliation unit tests passed;
all 10 direct replacement-owner tests passed after the shared reader change. The
signed-candidate integration case passed with a real source-build CLI child
successfully cancelling a synthetic candidate preparation with empty PATH. It
verified wrong digest, foreign active transaction, activation record, extra file,
unexpected backup and linked transaction-root refusals; resuming after a staged
receipt deletion preserved the old command/Host identities and update state.
Missing-journal replay refused. Final snapshot-use hardening reran only this affected
integration case and Clippy. Library/generator Clippy, public schema validation,
12 policy tests, formatting, whitespace and roadmap-index checks passed. Review
found no remaining actionable issue in the cancellation scope. This is a real
cancellation subprocess, not cross-version activation or real Host qualification.

Next: bind signed release generation and last-known-good metadata into the durable
activation outcome, exclude competing managed writers, then expose activation and
recovery using the existing journal/health owners. Prepared transaction cancellation
is now available; first-stage integration, real Host research/approval and named
cross-platform package qualification remain incomplete. Accepted ledger rows and
publication state are unchanged.


## CLI-403 eighteenth increment — journal-bound accepted release state

Base: `8a999ac2`; branch: `codex/cli-activation-release-state`.
Native candidate preparation now emits journal v3 with the candidate digest, prior
update revision/release metadata and the next signed release identity. The existing
approved journal digest and immutable outcome record therefore bind release state
without another sidecar format. v1/v2 retain their canonical representation and
legacy callers explicitly omit the binding. v3 requires the CLI pair and validated
release metadata; mixed/missing fields and unknown versions refuse.

The coordinator checks the prepared state revision before reservation. Successful
health records the committed decision before cleanup and atomically advances the
accepted generation/known-good identity while clearing the reservation afterward.
Failed health and undecided recovery preserve the old release state. Committed
cleanup recovery finishes that same decision; completed replay preserves revision.
Unexpected release metadata refuses before cleanup. Transaction IDs now include
update/workflow revisions so retry after rollback cannot collide with the retained
prior journal. Public CLI output schemas and activation record v1 are unchanged.

Checks on macOS: the 9 focused activation/parser/schema/reconciliation unit tests
passed; the final expanded CLI-pair compatibility case also passed after adding the
unknown-v4 negative. The signed-candidate integration test passed for failed health,
panic/unwind during health, committed cleanup blocked by a foreign file, and normal
success. It verifies old release preservation, signed generation/archive promotion,
recovery replay without revision increments, journal metadata tamper refusal, stale
state-revision refusal before health, and preparation with a new ID after rollback.
These are synthetic binary/health fixtures, not real cross-version executable or
SIGKILL qualification. The existing real CLI discard subprocess still passes with
v3 preparation. The successor fixture now uses generation 30 after 29; its test key
window was expanded to include 30 after the first run correctly rejected it.

Library Clippy passed after applying its nested-condition suggestion, with the
existing Rust 1.98 exception. Formatting, whitespace, schema-policy validation and
generated roadmap checks passed. Final review found no actionable issue in this
metadata/commit scope. Next: exclude competing managed writers and connect public
activation/recovery commands with fresh signed-candidate and approval revalidation,
process checks and the existing bounded real CLI health callback. Real Host
research/approval, native Windows state support and named package qualification
remain open. First-stage and ledger acceptance remain incomplete; no publication.


## CLI-403 nineteenth increment — shared Home write exclusion

Base: `28a93d09`; branch: `codex/cli-install-write-exclusion`.
Native activation/recovery/discard and preparation now take a fixed Home installation
lock before the existing config replacement lock. A private copy of the existing
journal-bound start record persists until outcome cleanup/state completion. A second
config root sharing the Home therefore cannot use the participating managed writers
while activation runs or awaits recovery. Recovery rejects a different marker and
clears its own marker only after completion. Existing journal/record/public schemas
are unchanged; Windows managed writes remain available while native activation
continues to be unsupported there.

The common managed-operation apply owner, candidate stage/apply/remove, and Desktop
Skills/workflow/CLI/packaged Host confirmation paths use the shared guard. Read-only
plans remain available. This increment does not claim complete writer coverage:
legacy migration, engineering `install native`, the old Desktop replacement helper
and other direct write dispatchers need the next inventory/guard pass. Public native
activation remains unexposed until those paths and fresh authority/process checks
are complete. The cooperative lock does not prevent arbitrary external file writes.

Checks on macOS: 9 focused activation tests, 9 managed-operation tests, and 9 Skills
consumer tests passed (sets overlap). The expanded CLI-pair case verifies Home-lock
contention across config roots, refusal after health interruption and committed
cleanup interruption, mismatched-marker preservation, recovery release and replay.
The contention assertion is outside the expected-panic catcher so an assertion
failure cannot masquerade as the simulated interruption. Signed-candidate integration
and source-build authority-refusal subprocess checks passed. Library Clippy passed.
These checks use synthetic activation/health fixtures, not SIGKILL or real Host
qualification. No program acceptance or publication status is advanced.

Next: finish the remaining writer inventory, then wire approved public activation
and recovery through the existing signed-candidate, journal and real CLI health
owners. First-stage integration and named package/real Host qualification remain open.


## CLI-403 twentieth increment — engineering and migration writer guards

Base: `c02e440b`; branch: `codex/cli-remaining-write-guards`.
Engineering native apply/remove now receive the command environment and acquire the
existing Home/config write guard before payload mutation. Apply retains release,
digest and approval validation first. Preview/verify remain available without that
lock. The existing authority-backed lifecycle test proves lock contention leaves the
payload absent and a pending recovery marker prevents removal of an installed payload;
normal apply/replay/verify/remove/replay still pass after the guard is released.

Legacy migration apply, continue and recover now acquire the same guard in their
shared CLI/Desktop owner. Apply validates approval/product identity before taking it
and holds it across provider staging, Host installation and receipt writes. Continue
covers Host confirmation, cleanup and finalize; recovery loads its receipt first and
holds the guard across restoration and receipt advancement. Existing data ownership,
secret handling, product control and CAS checks remain in place.

Checks on macOS: the expanded native CLI lifecycle test passed; all five migration
unit tests and the real CLI migration inspection/source-authority refusal test passed.
Library Clippy and whitespace checks passed. Migration tests cover persisted-state,
provider rollback and source-build refusal, not a new real signed-product migration
under lock contention. No public wire contract or accepted ledger row changed.

Next: finish old Desktop replacement/update-state dispatch coverage and address the
engineering arbitrary-root alias boundary (its current guard follows the invoking
Home). Keep public activation unexposed until this inventory and the remaining fresh
candidate/approval/process validation are complete. First-stage integration and real
Host/named package qualification remain incomplete; nothing was pushed or published.


## CLI-403 twenty-first increment — actual payload Home and Desktop exclusion

Base: `a205417d`; branch: `codex/cli-payload-home-guard`.
Engineering apply/remove resolves its already-approved managed directory and selects
the actual Home when it is the fixed native candidate payload root. Changing the
invoking Home can no longer bypass that target's installation lock or pending native
activation marker. General engineering roots retain their existing invoking-Home
scope. The authority-backed lifecycle test runs both ordinary engineering and
other-Home fixed-payload cases, retaining contention/preservation negatives and
successful apply/replay/verify/remove/replay.

The retained macOS replacement executor now takes Home then config locks after
parent exit and rejects native activation markers before changing state or files.
Handoff-failure restoration also takes both locks; it cannot race a native activation
by resetting update state outside the guard. Existing health completion remains
available while the executor holds its locks. The new refusal test checks unchanged
update state, retained staged application and absent backup when another writer or
native recovery blocks replacement.

Checks on macOS: both native lifecycle variants passed; all 11 replacement-owner
tests passed, including health failure and interruption/known-good behavior. Library
Clippy, the 11 replacement tests, whitespace, roadmap-index and frozen-source
boundary checks passed after the final handoff-error guard addition. These are owner-level fixtures, not a real parent/helper process or
SIGKILL qualification. No release authority, public schema or ledger acceptance changed.

Next: guard update-state dispatch and resolve the cross-config interaction with an
interrupted legacy Desktop replacement (which still lacks a durable global marker).
Then connect public activation/recovery with fresh candidate/approval/process checks.
The first-stage objective remains open; no push or publication occurred.


## CLI-403 twenty-second increment — update dispatcher write coordination

Base: `14c801c6`; branch: `codex/cli-update-dispatch-guard`.
The existing managed-write guard now reuses a transaction-compatible Home/config
lock helper. CLI/Desktop channel/cancel, signed verify/stage and staged reconciliation
use that helper while retaining their stage-specific state/CAS checks. Read-only
status/check and token-bound legacy health remain available. Signed paths without
release authority still refuse through their original owners before mutation.

Download locks cover only reservation and initial private staging after manifest
verification. Transport and its private-state CAS remain cancellable. Holding locks
across transport initially broke cancellation and stranded the two-party manifest
barrier test; that exact test process was identified and terminated before rerunning
the corrected scope. Concurrent reservation can now refuse either by revision or
installation-lock contention. A real first-directory creation race also surfaced;
the shared platform owner now accepts AlreadyExists only after full private-directory
validation. No link, ownership or permission check was removed.

Install's staged child owns its own guard. The parent waits without holding the lock,
then acquires it and checks the exact state/revision before advancing and preparing
handoff. This avoids introducing a parent/child lock conflict. Test adapters now use
isolated fixture Homes instead of the developer's process Home.

Checks: all 14 update unit tests passed, including new channel/cancel marker refusal
without state writes, status availability under lock, concurrent cancellation and
single-winner downloads. Nine managed-operation tests and CLI-pair activation/recovery
passed after extracting the shared guard. Final library Clippy, signed-candidate
integration, whitespace, roadmap-index and frozen-source checks passed. These tests do not qualify a real
Desktop parent/helper handoff or real Host research. Public wire formats and accepted
ledger rows are unchanged.

Next: complete durable cross-config coordination for interrupted legacy Desktop
replacement, then expose native activation/recovery using fresh candidate/approval
and real installed-CLI health checks. First-stage integration remains incomplete;
no push, publication or acceptance advancement.


## CLI-403 twenty-third increment — preserve legacy rollback evidence

Base: `d9fddb25`; branch: `codex/cli-legacy-rollback-evidence`.
Review of the legacy recovery path found rollback removed the transaction directory
and cleared active state before Host cleanup. A later cleanup failure therefore lost
its recovery evidence. Application restoration now preserves both. All three failure
callers use one completion owner: finish Host cleanup, remove the failed application,
sync its directory, then clear active state through CAS. Journal/health evidence is
retained even after completed rollback. A missing failed application can resume cleanup;
a substituted file or link refuses without clearing state.

Checks on macOS: all 11 replacement-owner tests passed before the final explicit link
check; the expanded health-rollback case was rerun afterward. It verifies retained
old application bytes, active state and journal on a substituted-file cleanup failure,
dangling-link refusal, then successful completion after restoring the failed directory.
Final library Clippy, whitespace, roadmap-index and frozen-source boundary checks
passed after the explicit link check. These owner-level fixtures do not prove real process interruption
or public recovery. Public schemas, accepted ledger rows and publication state did not
change.

Next: use the now-retained evidence for durable legacy Home exclusion and recovery;
then expose native activation/recovery with the existing signed candidate, approval
and installed CLI health owners. First-stage integration remains incomplete.


## CLI-403 twenty-fourth increment — durable legacy Home exclusion

Base: `0a1bae8b`; branch: `codex/cli-legacy-activation-marker`.
The old macOS executor now writes its serialized replacement journal to the shared
Home activation marker before live file replacement. Native and legacy paths share
exact-byte marker binding/clearing under the Home lock. Success and completed rollback
clear the marker; interruption or cleanup failure retains it across process-lock
release. A different config root therefore cannot begin a participating installation
write while legacy recovery is pending. Existing journal formats and public schemas
are unchanged.

Pre-activation restoration now returns errors instead of silently ignoring them. It
requires the restored/staged layout, absent backup and successful state CAS before a
marker may be cleared. A fixture that omitted its staged application correctly failed;
it now verifies evidence preservation on that missing-layout refusal before creating
the staged application and checking successful restoration.

Checks on macOS: all 12 replacement tests and the native CLI-pair activation/recovery
case passed. New panic/unwind coverage confirms HealthWindow state, other-config
writer refusal after the lock drops, wrong-marker clearing refusal, and return to
write availability after lower-level rollback/cleanup plus exact marker removal.
Existing success, pre-activation and rollback assertions also require marker removal.
Library Clippy, signed-candidate integration, whitespace, roadmap-index and
frozen-source boundary checks passed. This is owner-level recovery exercised by the test,
not a supported automatic/public legacy recovery command or SIGKILL qualification.

Next: connect recovery to the retained legacy journal/marker and then enable native
activation/recovery with fresh candidate/approval/process validation and installed CLI
health. The full first-stage objective remains open; no acceptance or publication.


## CLI-403 twenty-fifth increment — callable legacy health recovery owner

Base: `bb81d38b`; branch: `codex/cli-legacy-recovery-owner`.
Added `recover_legacy_health_interruption`, a library recovery owner requiring the
exact Home marker digest. Under Home/config locks it validates the replacement
journal against the configured store, active transaction identity/phase/generation,
canonical Host reconciliation journal and digest, backup ownership, application
layout and installed new canonical binary hash. It reserves RecoveryRequired through
CAS before rollback, preventing late HealthWindow completion from committing during
recovery, then reuses the existing rollback/cleanup/marker owners. No health rerun or
new product adoption occurs. Other platforms refuse.

The interrupted-health test now calls this owner instead of manually composing its
steps. Wrong marker digest and binary drift refuse before rollback, preserving backup
and active state; restoring the exact binary permits rollback, journal retention,
marker clearing and another config root's writes. The test fixture now persists the
canonical reconciliation bytes; the initial plain JSON serialization correctly failed
the existing canonical-input check and was fixed without weakening the loader.

Checks: the replacement suite's other 11 tests passed; the affected recovery case
passed after correcting fixture serialization. Final library Clippy, whitespace,
roadmap-index and frozen-source boundary checks passed. This is a library entry point and synthetic filesystem test,
not a public recovery CLI or real process-kill qualification. The full recovery goal
still includes committed cleanup, a second interruption during rollback, and public
approval/process validation. First-stage integration and acceptance remain incomplete.

Next: complete those recovery states, expose the reviewed recovery contract, and then
connect native activation with fresh candidate/approval and installed CLI health.


## CLI-403 twenty-sixth increment — retry interrupted legacy rollback

Base: `03b1ab45`; branch: `codex/cli-resumable-legacy-recovery`.
Legacy rollback now persists strict private record v1 with the marker digest, prior
accepted-release metadata hash and old canonical binary digest. Both ordinary rollback
and recovery reserve RecoveryRequired through CAS before application moves; late
health cannot commit a rolled-back version. The same record lets recovery verify and
continue after parking the new application, restoring the old one, removing the failed
application, or clearing active state before marker removal. It never infers successful
restoration from an absent backup alone. Retained failed applications must also match
the new canonical identity before cleanup. Journal/record evidence remains afterward.

Checks on macOS: all 12 replacement tests passed. The expanded recovery case injects
interruption at four boundaries and retries through the actual coordinator. It checks
wrong marker digest, new/old binary drift, unknown record version, changed marker and
prior-release bindings, and restored write availability after success. The targeted
case was rerun after the final semantic record negatives. Library Clippy, whitespace,
roadmap-index and frozen-source boundary checks passed. Application fixtures now include canonical
binaries so identity verification uses the production owner rather than placeholder
paths. Public schemas and ledger acceptance are unchanged.

Remaining: committed-outcome cleanup, a public approved recovery command and real
process interruption qualification. Checkpoints between operations do not prove recovery
from deletion stopped inside the failed application tree; if the remaining tree cannot
prove identity the owner refuses. Then native activation/recovery still needs public
candidate/approval/process wiring. First-stage integration remains incomplete; no push
or publication occurred.


## CLI-403 twenty-seventh increment — recover committed cleanup

Base: `6fe5b562`; branch: `codex/cli-committed-recovery`.
Added `recover_legacy_committed_cleanup`, a library owner that only finishes a durably
accepted legacy update. It binds the exact marker/configured journal and canonical
Host journal version/pack/digest, verifies the complete known-good identity including
channel, and checks installed new/remaining old binary identities. It never reruns
health, rolls back, or advances state. Missing backups after completed deletion are
valid for retry; substituted or unverifiable identities refuse.

The prior-release/old-binary record is now captured before application activation so
committed cleanup has the same identity evidence as rollback. Verified pre-activation
restoration discards that snapshot with its obsolete handoff contract. Committed
cleanup keeps the transaction journal, snapshot and downloads rather than deleting
its own evidence before marker removal. Bounded garbage collection is deferred; this
retention ceiling is explicit in the owner.

Checks on macOS: all 13 replacement tests passed. The new case interrupts before and
after backup removal and before marker clearing, rejects wrong marker digest, changed
accepted archive identity, and drift in either application binary. Retry preserves
the exact state revision and installed new version. A malformed channel/version test
was correctly rejected by the state owner; a valid changed archive identity tests the
recovery layer instead. Final library Clippy, whitespace, roadmap-index and
frozen-source boundary checks passed.
No public JSON contract, ledger acceptance or publication status changed.

Next: expose approved recovery through CLI with process checks, then connect native
activation using fresh candidate/approval verification and installed CLI health. Real
process-kill qualification and cleanup interrupted inside an application tree remain
open; unverifiable partial trees still refuse. First-stage integration is incomplete.


## CLI-403 twenty-eighth increment — inspect live applications before recovery

Base: `d3e25b03`; branch: `codex/cli-recovery-process-guard`.
Both recovery library owners now inspect the current user's mapped application files
before mutation. They reuse the bounded Host child collector with fixed native lsof,
empty inherited environment, 30 seconds and 8 MiB per stream. Existing probes keep
512 KiB. Error exit, stderr, malformed/empty/truncated or invalid-encoding output
refuses recovery. Destination, backup, staged and failed application paths are checked
by components, so a similarly named neighboring directory is not mistaken for a match.
No existing user process is stopped.

Real local inspection showed lsof adds ftxt fields and produced about 2 MiB, exceeding
the Host-probe default; its protocol and bound were adjusted explicitly. A real Chinese
path test exposed C-locale hexadecimal UTF-8 escaping. Matching now encodes target bytes
as documented in the installed lsof manual, while control-character targets refuse.
The Clippy exception is local to fixed native lsof inspection; external model-runtime
launch policy is unchanged.

Checks on macOS: 15 replacement/process tests passed with process-read permission.
The final controlled live-process case passed for a path containing Chinese and spaces:
it detects a copied system sleep binary while running and allows the path after that
test child exits. Parser negatives and the existing bounded Host failure test passed
after the final matching change. Library Clippy passed. Only aggregate protocol metadata
was inspected outside tests, without exposing other process paths. This proves process
inspection, not a real update/SIGKILL recovery or cross-platform qualification.

Remaining: public recovery approval/output contracts, native activation command wiring
and real package/Host qualification. The check is a current-user snapshot and cannot
prevent a subsequent launch or attest to other users' processes. First-stage integration
is incomplete; no push, publication or acceptance advancement.


## CLI-403 twenty-ninth increment — expose approved legacy recovery

Base: `a30cdc0c`; branch: `codex/cli-public-recovery`.
Added `update recovery-preview` and digest-bound, explicitly approved `update recover`.
The preview reads existing private state without creating directories and reports the
legacy transaction, marker digest and rollback/committed-cleanup mode. Execution routes
to existing owners, preserving Home/config locks, CAS, process inspection and identity
checks. Source builds need no fresh release authority for this exact-owned recovery.
Native activation markers remain outside this legacy entry point. Recovery must run
from a CLI outside affected application paths; preview is not a readiness guarantee.

Review added transaction-ID validation at the shared journal validator and retained
private native-directory validation in the read-only marker path. The new additive
public JSON contract has a Rust Draft 2020-12 producer, two golden fixtures, strict
consumer checks and a public-schema policy entry. Both general and update help list
the commands.

Checks on macOS: 15 replacement/process tests passed after final validation changes;
existing interrupted rollback and committed cleanup cases now finish through the real
CLI dispatcher, checking preview mode, wrong digest, missing approval and final output.
The subprocess CLI check covers syntax, duplicate/missing/unknown options, both option
orders and no state creation on missing-marker preview/recovery. Rust schema/consumer
test, library/generator Clippy, schema-policy validator and its 12 tests, roadmap index,
whitespace and frozen-source boundary checks passed. No full desktop/package suite was
rerun for local integration.

Remaining: native activation/recovery command wiring, real update process-kill cases,
partial-tree recovery limits, and named-candidate/platform/Host qualification. Successful
synthetic recovery via dispatcher does not prove a real packaged application crash.
First-stage integration and program acceptance remain incomplete. No push/publication.


## CLI-403 thirtieth increment — guard native activation against live CLI files

Base: `9faf9022`; branch: `codex/cli-native-process-guard`.
Native activation and recovery on macOS now check mapped CLI destination, staged and
backup files under their existing Home/config locks, before records or file mutation.
They share the bounded process-inspection owner used by legacy recovery. Other platform
behavior is unchanged; this does not qualify their process visibility or public activation.
No process is stopped by product code, and the snapshot cannot prevent a later launch.

Checks: all four reconciliation tests passed, including a new controlled process case.
The case copies system sleep into its isolated CLI fixture, checks destination/staged
refusal before activation, interrupts health, checks destination/backup refusal during
recovery, then stops only its own children and completes rollback. It verifies unchanged
state, receipts, marker and retained backup at refusal boundaries. The signed candidate
integration test passed through v3 activation/recovery with the new guard. Library
Clippy, whitespace, roadmap-index and frozen-source checks passed. An initial test build
referenced an unavailable clock helper; using stdlib time fixed the test without changing
product behavior. No schema or program acceptance changed.

Next: connect fresh candidate/approval validation and installed CLI health to the public
native activation command, with native recovery exposed through its existing owner.
Real process-kill, platform and Host/named-candidate qualification remain open; the first
stage is not complete. No push or publication occurred.


## CLI-403 thirty-first increment — connect public native activation and recovery

Base: `28f6a329`; branch: `codex/cli-native-activation-commands`.
Added macOS `install candidate activate` with signed release inputs, predecessor,
transaction/journal/approval digests and all three explicit approvals. The dispatcher
verifies the signed candidate and running packaged product. Under the existing
Home/config locks, the coordinator's preflight checks the candidate/Home/full next
release identity and recomputes the preparation approval against current preflight,
workflow and update state. The existing preparation approval bytes remain unchanged.
Installed CLI health is mandatory; callers of the public command cannot replace it.

Added `install candidate activate-recover` with exact transaction/journal identity and
filesystem approval. It uses the existing recovery owner, process check and durable
outcome without rerunning health or requiring new release adoption authority. These
public commands explicitly refuse non-macOS pending equivalent process inspection;
existing lower-level platform behavior is unchanged. Both success outputs have a
Rust-generated additive schema and activate/recover golden fixtures.

Checks: 13 focused activation tests passed, including preparation golden stability,
new parser approvals and new schema consumer checks. The signed candidate integration
case passed through the approved wrapper, proving wrong approval and substituted next
archive identity refuse before records/state changes; its non-runnable payload fails
mandatory real CLI health and rolls back. Native recovery for all macOS outcome cases
runs through a real CLI subprocess and checks output identity/state. The standalone
CLI source-authority refusal test passed without creating Home/config state. Library
and generator Clippy, schema-policy validator plus 12 tests, roadmap index, whitespace
and frozen-source boundary checks passed. Unchanged results were reused at integration.

Next: prove the complete successful public activation path with a real signed runnable
candidate, then real process-kill recovery and named-candidate/platform/Host qualification.
Current positive commit tests still use the lower-level coordinator's health callback;
they are not successful packaged public-activation evidence. First-stage integration and
program acceptance remain incomplete; no push or publication.


## CLI-403 thirty-second increment — real public activation journey

Base: `57bfd7b8`; branch: `codex/cli-public-activation-journey`.
Extended the existing nonpublishing candidate runner with optional
`--predecessor-manifest`. It shares the existing package/signature construction for
both generations, builds and reads the actual predecessor version, then installs its
CLI before staging the successor. Public preview, preparation and activation run from
the staged successor. Actual installed version, binary change, health and MCP are checked;
public recovery replay must preserve the exact committed update state.

The real macOS run passed: `2.0.0-alpha.4` -> `2.0.0-alpha.5`, Codex target, temporary
signed generations 1 -> 2. The predecessor was built from a `git archive` of
`57bfd7b8dca007e2432b26797e8032f68695c80f` with only the workspace version and nine
matching lockfile version entries changed to alpha.4 in an isolated source copy.
This is a real executable/version switch using the same content pack, not a historical
published predecessor, resource-pack migration or immutable release qualification.
The successor product source is the base commit; harness changes are on this branch.

Evidence: `/Users/pengjiaxin/Work/qiongli-cli403-public-activation-20260907-r2/acceptance-evidence.json`;
SHA-256 `5dc93c197b84ba4ff3539f67f0b20ab47199ad231d6a163937d8943feb39be39`.
The receipt contains the existing 27 baseline evidence fields plus the passed native
activation journey, old/new binary identities, committed outcome, installed health/MCP
and unchanged recovery replay. `publication_allowed=false`; real external clients,
displayed window and production signing remain not run. Test signing keys stayed in memory.

Six runner tests, 15 release-version/boundary tests and final example Clippy passed.
Extraction initially left needless borrows, fixed before final Clippy. The first real
journey exposed a harness path error: install receipt IDs were incorrectly used as
artifact directories. A shared helper now reuses the existing artifact path owner for
both old/new and baseline launches. The initial failed output is retained separately;
the complete real journey was rerun successfully after the fix. Whitespace, roadmap
index and frozen-source boundary checks passed; unchanged focused results were reused.

Next: real process-kill recovery using these runnable packages, then remaining platform
and live Host/named-candidate qualification. First-stage integration and program
acceptance remain incomplete. No push or publication occurred.


## CLI-403 thirty-third increment — real SIGKILL recovery and predecessor health

Base: `3ed2e335`; branch: `codex/cli-native-sigkill-recovery`.
The existing two-version runner now reuses its signed packages for separate normal and
interrupted Homes. It starts public activation in a new test-owned process group,
observes the installed CLI inode change with a Home marker and no durable outcome,
and sends SIGKILL to that group (including its health child). It requires an actual
signal exit, the new binary at interruption and unchanged prior accepted release.
Already-reaped children are never signaled; missed windows fail rather than pass.
Public recovery must restore the old binary/version/release and pass actual health/MCP.

The first run completed rollback but exposed a real cross-version health bug:
`verify_native_cli_health_plan` used the running process version when validating the
restored predecessor's plan. The shared validator now accepts an explicit expected
version for trusted candidate health; constructors and managed writes keep the current
process version. Schema, digest, TTL, operation and approval checks remain intact.
Nine managed-operation tests passed, including exact predecessor health, digest drift,
wrong candidate/version and refusal to use the old plan for current-version writes.
Six runner tests and final library/example Clippy passed.

The complete real macOS runner passed after the fix. Evidence:
`/Users/pengjiaxin/Work/qiongli-cli403-native-sigkill-20260907-r2/acceptance-evidence.json`;
SHA-256 `63ae003977c4a5dc407841544117be244c9e8094ce73d64800a303e4ce796d73`.
It includes the normal alpha.4 -> alpha.5 commit/replay and real SIGKILL rollback,
restored binary/version, unchanged prior release, and restored health/MCP. The initial
failed run is retained separately and is not counted as passed. The predecessor is a
version-only build derived from a `git archive` of the base, with matching lockfile
version changes; the successor includes this branch's health fix. Both use ephemeral
test signing, the same content pack and a source label, not immutable released source.

Whitespace, roadmap-index and frozen-source checks passed. No public wire schema,
program acceptance, push or publication changed. This is one observed process-kill
boundary; arbitrary interruption positions, power loss and partial-tree deletion are
not qualified. Next: real Host baseline and remaining platform/named-candidate evidence,
while preserving these explicit limits. First-stage integration remains incomplete.


## SEC-401–403 first CLI Host boundary increment — untrusted approval claims

Base: `307b4086`; branch: `codex/host-untrusted-evidence-boundary`.
The existing Host packet now explicitly classifies PDF excerpts, web pages,
repository content, dataset documentation and imported notes (alongside project
and tool data) as untrusted evidence. It says source instructions cannot extend
tool/project scope, authenticate evidence or approve writes; candidate acceptance
is not human approval. No new parser, permission owner or public schema was added.

The copied-binary MCP journey now reads a project name instructing it to invoke
capture apply, verifies that source text stays out of the trusted handoff, then
attempts the write through orchestration read with approval set true. The existing
server rejects it. Existing cross-project and forged-evidence refusals still pass.
The connected capture fixture now contains an imported note claiming human approval;
its existing false-approval and wrong-digest cases still reject before the explicit
approved happy path. These are server-boundary fixtures, not a real model attack
corpus or proof of human-origin approval.

Eight orchestration input tests and all seven MCP stdio tests passed. Six MCP tests
passed inside the sandbox; the existing Zotero loopback test was initially denied
permission to bind, then passed when rerun alone with scoped permission. Execution
library Clippy and both deterministic Codex/Claude plugin bundle tests passed.
Whitespace, generated roadmap index and frozen-source boundary checks passed.
Runtime contract documentation records the remaining human
origin gap: a Host-supplied boolean plus digest is not human-interaction evidence.

SEC-401–403 remain unaccepted. Next: inspect the actual Host approval interaction
and isolated read-only client integration before qualifying research writes. No
private research data, remote publication or program acceptance was authorized or
changed; first-stage integration remains incomplete.


## CLI-404 first real-client baseline — isolated Codex and Claude installation

Product source: `8ffacfe1`; branch: `codex/real-host-client-baseline`.
Ran the existing ignored integration tests against actual installed clients using
explicit executable paths and fixture-owned Home/config roots. No model prompt,
private research input, publication or user-profile registration was involved.
Both tests completed successfully, including removal of temporary client entries.

- Codex `0.153.4`: `real_codex_clean_client_installs_enables_caches_and_launches_bundle`
  passed (44.10 s). Local plugin installation, enablement, inventory, cache receipt,
  customized Skill bytes, Lite registration/removal and plugin removal passed.
  Bundle receipt SHA-256:
  `fafae1302449a2e081f6a98d54c60f41e5f347020afdf6d6757651bd3cb898b8`.
  Codex MCP evidence is paired client inventory plus independently launched exact
  protocol, not a model-initiated tool call. The optional Plugin Creator validator
  was not configured (`plugin_creator_valid=false`); it is not counted as passed.
- Claude Code `2.1.263`: `real_claude_clean_client_discovers_and_installs_both_local_forms`
  passed (65.18 s). Strict plugin validation, direct Skill discovery, local marketplace
  installation, inventory, cache receipt, customized Skill bytes and cleanup passed.
  Direct and marketplace receipt SHA-256:
  `4bf27e05710b140faf8414e6e0b80cebef99d5da4f7b6b316592282bd10e2c3a`.
  The real Claude client reported its Lite MCP connection as connected.

Commands used `cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli`
with `--test codex_plugin_bundle` / `--test claude_plugin_bundle`, the exact names
above, `--locked --offline -- --ignored --nocapture`, and `QIONGLI_CODEX_BIN` /
`QIONGLI_CLAUDE_BIN` pointing to the actual mise installation executables rather than
Home-dependent shims. Both also passed cached MCP execution with empty PATH,
14 Lite / 32 Full tool contracts and the Full Host route, using the existing test
signing authority. This source-build fixture evidence does not qualify an immutable
standalone release candidate or production signing.

Review confirmed the tests launch protocol probes separately from model execution;
therefore no human approval, authenticated research reasoning, same-device Host
handoff or SEC program acceptance is claimed. Next: prepare the real research
preview and verify human-origin approval through an available Host interaction,
while completing SEC prerequisites. First-stage integration remains incomplete.


## CLI-404 human approval preview — pending a real decision

Base: `a199656f`; branch: `codex/host-human-approval-preview`.
Reused the existing `project capture preview/apply` commands; no new approval
system was necessary. A private fixture under
`packages/qiongli-native/target/cli404-human-review-a199656f` contains the created
empty project, portable candidate, preview, missing-approval result and empty Inbox
snapshot. `REVIEW.md` presents the exact scope in Chinese. The candidate records
only the prior real-client engineering observation, with no academic changes,
decisions or evidence items. This is an approval exercise, not a research journey.

Project: `prj_f7708e7c379deb5b44de85781096804c`, revision 1.
Plan digest: `9cc0ed23c200fe3e1809ee2072d6e26d1ecc5e7b4c5ce4e69f42d5bd4c4ff371`.
Capture: `cap_b204255fd6efd7c43d69440267032506e4004f72810851b7c0b091783cba562a`.
A real CLI apply without filesystem approval failed; the history file remained
absent and a fresh CLI Inbox read reported zero entries. No capture was applied.
The existing owner can apply the exact preview after a real human decision and
fresh digest/revision revalidation. Do not infer that decision from development
authorization or from these fixture setup writes. The user-facing review requests
only pending Inbox intake, never consolidation or academic mutation.

Human approval is pending, SEC-401–403 remain unaccepted, and first-stage integration
is incomplete. Continue independent security work while this interaction is pending;
publication, private research access and broad Host write enablement are not covered.


## SEC-401–403 packaged entry guidance and accurate active status

Base: `324ae5c0`; branch: `codex/host-adapter-evidence-guidance`.
The native Codex and Claude plugin adapter text previously lacked the explicit
untrusted-source rule before orchestration issued its first handoff. Both existing
adapter owners now classify PDF/web/repository/dataset/note/project/tool content
as evidence and reject embedded system-message or human-approval claims as control.
No permission mechanism or public schema changed; the existing server guards remain
the enforcement boundary. Packaged Skill assertions verify the guidance survives
materialization, alongside existing deterministic receipt/tamper and MCP checks.
Both Codex/Claude package tests and platform library Clippy passed. Whitespace,
generated index and frozen-source boundary checks passed.

The ledger now marks CLI-404 and SEC-401–403 active, linking the existing bounded
plan and recording each remaining gap. Their proposed states no longer described
the implemented source and real-client evidence. Accepted count remains 46; no
commit/run acceptance evidence or dependencies were fabricated. Seven roadmap
validator tests passed, and the generated index is regenerated from the ledger.

The previous exact Inbox preview still awaits a human decision; this increment
neither applies it nor treats automatic continuation as approval. Next: complete
the pending human interaction and broader source-boundary adversarial validation.
First-stage integration remains incomplete; no remote push or publication.


## SEC-402–403 evidence isolation — second-process replay refusal

Base: `16c41cf5`; branch: `codex/host-evidence-process-isolation`.
Extended the existing copied-binary Host round trip through its existing process
helper. A second real MCP process loads the same project/checkpoint and submits
the original process's otherwise valid candidate and evidence references. It must
reject with `host-candidate-evidence-unauthenticated`. The original process then
submits successfully with the unchanged generation/document reference, proving
that rejected replay neither advances CAS nor consumes the original evidence.

The focused copied-binary test passed (1.42 s), retaining its cross-project,
forged-result, write-tool refusal, valid submission and cancellation checks. Review
traced the server-local ledger, exact project/revision/run/handoff/evidence match,
and checkpoint CAS owner; no implementation or wire-contract change was required.
The runtime specification now records this process boundary. Authenticated reads
are observations, not continuous freshness guarantees; no live-file revalidation
or restart recovery beyond this two-process replay case is claimed.

Whitespace, generated-index and frozen-source boundary checks passed. The pending
human Inbox preview is unchanged and unapplied. Broader adversarial evidence,
human approval and first-stage integration remain incomplete; no acceptance or
publication state changed.


## CLI-404 live Host routing probe — current installation is not the candidate

Base: `d1e64ff0`; branch: `codex/live-host-qualification-gap`.
The current Codex conversation exposes native Qiongli tool names. An actual
`qiongli_orchestrator_route` call with `platform=codex` and a bounded two-source
comparison/review request (explicitly routing only) returned `mode=preview`,
`runtime_profile=marketplace_lite`, `preview_only=true`, `project_writes_allowed=false`
and `upgrade.required_for_execution=true`. No project read or write was requested.
This proves actual Host-to-tool connectivity, but contradicts a Full-ready claim.

Read-only local inspection found the cached personal `qiongli-next/2.0.0-alpha.3`
manifest invokes `./bin/qiongli mcp serve --profile full --transport stdio`.
The binary reports `qiongli 2.0.0-alpha.3`, exits zero, and has SHA-256
`6561b33f347a44bedbc6709523e1ba53d24e24289553b25924a1acb29c6ee7a0`, matching its v2
bundle receipt with `mcp_profile=full`. These cached bytes are not the current
alpha.5 candidate. The live response is consistent with the historical Full/Lite
routing defect already fixed and tested in current source; no duplicate fix or
manual cache edit was made. Tool declaration visibility alone is not readiness.

The first-stage scope remains CLI-401–405 plus CLI-410 preparation. Current source
and temporary packages cover independent builds/resources and substantial macOS
install/update/recovery. Open requirements include clean-machine qualification
for the declared supported platform scope (empty PATH is insufficient), a named
candidate in the actual Host, authorized two-source research, human-approved
canonical writing and actual restart/same-device continuation. The existing
pending Inbox preview is only an engineering approval exercise, not P01's formal
comparison note or P05's real Host journey. Program acceptance is still incomplete.

Next actions need to retain this distinction: qualify the chosen package and Host
in an isolated authorized environment, then execute the research journey with
approved test sources and human decisions. Existing user installation/account
settings were inspected only for public version/launch metadata and not modified;
no credentials, private research, remote push or publication were used. Whitespace,
generated-index and frozen-source checks passed for this evidence-only update.


## CLI-404 candidate-backed real clients and virtual-environment invocation

Base/product source: `b5e2f6f008d6cf03f94f79a1ef2703c59a1baefb`;
branch: `codex/candidate-real-host-qualification`.
Preparing the existing runner with the installed Plugin Creator validator exposed
an invocation bug: `valid_external_file` canonicalized virtual-environment Python
symlinks, bypassing the environment containing PyYAML. The new command-path wrapper
reuses existing absolute/nonempty/regular-file resolution validation but preserves
the supplied invocation path for Codex, Claude and Python. Data inputs still use
canonical paths. A Unix regression checks preserved command paths, canonical data
paths, and rejection of dangling links, directories and relative paths. Seven
runner tests and example Clippy passed.

The actual candidate runner completed successfully with both real clients and the
repository's existing Python virtual environment; no dependency was added. Evidence:
`/Users/pengjiaxin/Work/qiongli-cli404-candidate-real-hosts-b5e2f6f0/acceptance-evidence.json`;
SHA-256 `c32889c07532fa15b9e0e77479cdf58309d524af15938d75e8be9f0c16cc100d`.
macOS aarch64 alpha.5 archive SHA-256:
`08f3d65d81619f92d9b7fee6bfdb7ed114973180192893a14f8701251927f105`;
candidate SHA-256:
`3250e7f946d8b8d68dc1ce3f8544a642b4a313ab329c5de0e8a08aff7e8a454a`.
The source commit labels product code; only the acceptance runner changed here.

All 27 baseline fields and the candidate-backed real-client gate passed. Codex
0.153.4 passed Plugin Creator validation, install/list/cache/empty-PATH MCP/removal;
Claude Code 2.1.263 passed strict validation, local marketplace install/list/cache/
empty-PATH MCP/removal. Both used fresh Home/config roots. Test keys stayed in memory,
`publication_allowed=false`; no existing user installation was changed.

Review found an important scope limit: this runner's cached MCP probe asserts the
14-tool Lite contract and config status, not Full routing or model research. Earlier
source fixture Full checks remain separate evidence. Next: extend this same owner
to verify the candidate's actual Full route before using it for Host research.
Native two-version activation was not rerun because no predecessor was provided;
prior SIGKILL evidence remains separately scoped. Clean-machine qualification,
production signing, human-approved academic writing and first-stage acceptance are
still open. Whitespace, generated-index and frozen-source checks passed.


## CLI-404 cached candidate Full MCP qualification

Base/product source: `8c6450f0a744812958154fa94a6bb9612d309de6`;
branch: `codex/candidate-full-mcp-qualification`.
The existing cached MCP runner now probes both Marketplace Lite and Full from
both actual client caches with empty PATH. It preserves the 14-tool Lite check,
requires all 32 Full tools, then calls the Full orchestration route. Full must
return `orchestrator_mcp` and `requires_full_runtime=true`, without Lite preview,
runtime or upgrade fields. The same process/output validator still requires clean
stdio, exact response count and redacted config status. Client receipts now record
Full tool count and route verification only after these calls succeed.

Eight runner tests passed, including rejection of missing, incomplete or mixed
Full/Lite responses; example Clippy passed. The real macOS aarch64 candidate run
also passed with Codex 0.153.4 and Claude Code 2.1.263. Evidence:
`/Users/pengjiaxin/Work/qiongli-cli404-candidate-full-mcp-8c6450f0/acceptance-evidence.json`;
SHA-256 `210b6bbee64ff3977d7bad08abbcd7d2aecc461fede187cde358c5601f7f483e`.
Archive SHA-256:
`a63fe31aacb82b59430e6589308beb8179cfcdda8a9129b1e926ab798d1ffca3`;
candidate SHA-256:
`74c256cd0fc03fcd4b9ef172b08bc1df3a45ed4c67c9184a75495f462a1ba130`.
Both candidate-backed client entries report 14 Lite / 32 Full tools and
`full_route_profile_verified=true`; strict plugin validators, install/cache/remove
and the 27 existing baseline fields also passed. Product code is unchanged; this
branch changes the acceptance runner. Whitespace, generated index and frozen-source
checks passed.

The runner launches cached MCP protocol probes directly after actual client
installation; this is not a model-initiated research session. Test signing remains
ephemeral and `publication_allowed=false`. The current conversation still uses its
old alpha.3 installation. Next: authorized research inputs and an actual candidate
Host session, the pending human decision, canonical note writing and same-device
continuation. No scientific correctness, SEC acceptance or first-stage completion
is inferred from the successful route. Prior update/recovery evidence retains its
separate scope; it was not rerun here.


## CLI-403 Linux ARM64 build and clean-container consumer evidence

Source: `458c3caa`; branch: `codex/linux-arm64-consumer-evidence`.
Read-only environment inventory found three suspended Parallels guests and a
running Podman machine. No guest was resumed and no existing container was changed.
Two existing ARM64 images enabled independent validation while human approval
remains pending. The build used Rust 1.97 (the workspace minimum), no network,
read-only source and the active mise Cargo registry cache. Command:
`cargo build --manifest-path /src/packages/qiongli-native/Cargo.toml -p qiongli
--no-default-features --release --locked --offline --target-dir /build`.
It passed in 1m16s. Rust image ID:
`72db5382ffef4a5dbf89a125ee07bf0073404cee718902ed4ef0d6da99b0261a`.
Initial attempts failed before compilation due to using an image ID as a repository
digest, then mounting an unused default Cargo cache; corrected without downloads.

A separate Debian 12 ARM64 image, ID
`61c44d586ac3463e3f9160e3cf6190e96b0769c58707517c313bb451095495a4`,
received only the executable and isolated test state. It had no source mount and
asserted absence of Python, Python3, Node, npm, Cargo and rustc. `ldd` showed libc,
libgcc and the Linux loader, with no GUI libraries. Twelve recorded checks passed:
environment, version/help, embedded content, UI refusal, project preview,
missing-approval refusal with no created project, approved fixture creation,
project read in a fresh container, Full MCP 32-tool/route checks, and export
preview/apply. Each command exited in its own disposable network-disabled container.
This is a source binary and an empty fixture project, not research completion.

Evidence and runnable local probe script are under
`packages/qiongli-native/target/cli-linux-native-458c3caa/`.
`consumer-evidence/receipt.json` SHA-256:
`fdcf3b570b15f2ceae05a617dcda99ec26d63fe05cc7b200421117227b16c21e`.
Linux executable SHA-256:
`31f69845508e4ec3431710595f590b14d13753498919cc84fa7547235bc28f4f`.
The first MCP probe omitted Podman's stdin forwarding and returned no messages;
its failed output is retained separately. Added `-i` and reran only MCP/export,
reusing completed checks. Final response parsing confirmed 32 Full tools and the
Full route. Project/export outputs were reviewed; semantic revision remains 1,
and export contains the portable project manifest.

This is stronger than clearing PATH, but still a clean container rather than a
clean VM. Signed Linux packaging/install/update/recovery, desktop sessions,
additional architectures and real Host research remain unqualified. The pending
human preview remains unchanged and unapplied. No new production code, dependency,
program acceptance or publication was introduced. Whitespace, generated-index and
frozen-source checks passed. Continue platform/candidate qualification and the
human research interaction without claiming first-stage completion.


## CLI-403 Linux candidate install and exact consumer follow-through

Source: `da10e055dd72d9d697f69a37fd8644060498f3af`;
branch: `codex/linux-candidate-install-evidence`.
Ran the unchanged native candidate acceptance owner inside the existing Rust 1.97
ARM64 container, with network disabled, source/cache read-only, private output and
separate Linux build storage. The first invocation contained a mistyped full source
hash; its test-owned container was explicitly stopped (exit 137) before acceptance.
That output remains under `run` and is invalid evidence. The corrected invocation
used the verified full HEAD and a fresh `correct-source` output; it passed.

Linux candidate receipt:
`/Users/pengjiaxin/Work/qiongli-cli403-linux-candidate-da10e055/correct-source/acceptance-evidence.json`;
SHA-256 `05df98dd1eff2eb4a22555ca003de8f0f4c445118e16e402d371d990c2af6a7d`.
Archive SHA-256:
`e20f00a7b2a719e7eaaec74c56873ae5edc5129477df358097b507e71098f7dc`;
candidate SHA-256:
`9c8f904df80cc818ea011426726a350b9fb83aa5abba519c22339bf89175d2ae`.
All 27 baseline fields passed, including candidate staging/approval replay, initial
install, installed child health/tamper refusal, failure compensation, uninstall and
preservation of unmanaged state. Host lifecycle fixtures are local registration
checks; real clients explicitly remain not run. Ephemeral signing is not production
signing, and `publication_allowed=false`.

Then reran the existing 12-check consumer script against that candidate's exact
executable in a fresh Debian 12 ARM64 state directory without Python/Node/Cargo,
source or network. Version/content, missing approval, fixture project creation,
fresh-process read, Full MCP 32-tool/route and portable export checks passed.
`packages/qiongli-native/target/cli-linux-candidate-consumer-da10e055/consumer-evidence/receipt.json`
SHA-256: `adae0525ff635b1d2fe328a7d1c5afeeea4df637edaadd988ce2cb6753a64dbc`.
Consumer binary SHA-256:
`e0fbe8d2eecfff4a250619a3bb59452dcc3704a3d109122f23f35f08bed3a6b4`;
verified that this digest appears in the signed candidate document. The probe script
and command outputs remain beside the receipt for review.

No production code changed. Public native activation/recovery remain macOS-only;
Linux process inspection and update qualification were not bypassed or claimed.
This is container evidence, not a clean VM or physical Linux machine. Windows,
real research/human approval and first-stage acceptance remain open. The earlier
pending human preview was not applied. Whitespace, generated-index and frozen-source
checks passed; no existing VM or user installation was modified or published.


## CLI-403 Linux reconciliation atomic no-replace prerequisite

Base: `e6a37d41`; branch: `codex/linux-atomic-update-rename`.
Tracing activation, failed-activation compensation and rollback found that Linux
used `ensure_absent` followed by `fs::rename`, allowing a target created between
those operations to be overwritten. Linux now reuses the existing rustix
`renameat_with(NOREPLACE)` owner already used on macOS. Linux EEXIST preserves the
existing collision reason; other failures remain activation failures. Unsupported
kernel/filesystem operations fail closed rather than falling back to overwrite.
macOS behavior and other platform branches are unchanged.

A shared native test races eight source files into one destination and requires
exactly one winner, intact bytes for every loser, and refusal to overwrite a
symlink destination. It passed on macOS. All four Linux `update_reconcile::tests`
passed in the Rust 1.97 ARM64 container, including CLI-pair activation/recovery,
incomplete journals and non-product canary preservation (3.21 s after build).
macOS library Clippy passed with the existing Rust 1.98 lint exception.

The first Linux test container failed during tmpfs mount preparation with ENOSPC.
Read-only checks showed 17 GiB available in the Podman VM and ample host space;
no unrelated image, VM or data was removed. Replacing that mount with a dedicated
repository target directory allowed the tests to run successfully. Only the test
container was disposable; existing containers and suspended VMs were untouched.

Runtime contract, whitespace, generated-index and frozen-source checks passed.
This fixes a lower-level data-loss race, not Linux process inspection or permission
to enable public activation/recovery. Those commands remain macOS-only. The earlier
candidate receipts predate this product change and retain that scope; no new
package/Host acceptance is implied. Human approval and first-stage completion
remain open, with the pending Inbox preview still unapplied.


## CLI-403 Linux current-user process inspection prerequisite

Base: `503426f7`; branch: `codex/linux-process-update-guard`.
Internal reconciliation now routes Linux CLI paths through the existing shared
installation-process guard. The Linux implementation reads `/proc` directly with
stdlib/rustix and adds no runtime utility dependency. It bounds mount data, status,
entry count and scan time; checks real/effective/saved/filesystem UIDs; and anchors
status/executable reads to an open process directory to avoid PID-reuse retargeting.
Other users and identified kernel threads are excluded. Ambiguous executable access,
invalid status/mount data and ptrace-only process visibility fail closed.

Both the literal executable link and its possible ` (deleted)` suffix form are
checked by path components. This follows the Linux
[proc executable documentation](https://man7.org/linux/man-pages/man5/proc_pid_exe.5.html)
and [kernel proc filesystem contract](https://www.kernel.org/doc/html/v6.15/filesystems/proc.html):
an unlinked executable may still run, while an unavailable executable link does not
prove process exit. These are current-user observations in the visible PID namespace,
not prevention of later launches, namespace-independent host coverage or protection
against arbitrary same-user execution. No process is terminated by product code.

Two Linux tests passed: a real test-owned copied sleep executable was detected both
before and after unlink, then allowed after kill/reap; a proc fixture rejects missing
exe, duplicate UID records and hidepid=4, distinguishes neighboring paths and skips
other-user records. Four Linux update/reconciliation tests also passed after hookup,
including activation/recovery, atomic moves and non-product canaries. macOS library
Clippy passed. The existing Linux Rust 1.97.1 image lacked Clippy; installing only
that official toolchain component inside a disposable container allowed Linux
release Clippy to pass with `-D warnings`. No project dependency changed.

Runtime documentation, whitespace, generated-index and frozen-source checks passed.
Public Linux activation/recovery remain unsupported pending complete platform
qualification. Prior named package receipts predate this code and are not upgraded
by these tests. Next: exercise the Linux process guard at the public activation
boundary and qualify actual commit/recovery before enabling that path. Human
approval and first-stage integration remain incomplete; no user process, current
Host installation, research data or publication was changed.


## CLI-403 Linux public native activation and interrupted recovery

Base: `3d01349961fc886b1cba02cf928cc843ba21d8c7`; branch:
`codex/linux-native-activation`. The three native candidate activation/recovery
platform gates now include Linux. Existing approval, signed product verification,
Home/config locks, exact journal/revision checks, process inspection, atomic moves
and installed health remain the owners. Legacy App replacement stays macOS-only;
other platform public activation remains unsupported. No public JSON changed.
The existing acceptance runner's Unix process-group/inode interruption probe now
runs on Linux too; no second qualification framework was added.

A real Rust 1.97.1 Linux ARM64 container built successor alpha.5 from the base plus
this platform-gate change and predecessor alpha.4 from a base archive with only
workspace/lockfile version edits. It used ephemeral in-memory signing keys and
an isolated `/qualification` Home outside the source checkout, with empty runtime
PATH. The predecessor is a version fixture, not historical published-source proof.
The disposable container used an init process to reap its test-owned descendants.

Receipt: `/private/tmp/qiongli-linux-activation-3d013499/run/acceptance-evidence.json`;
SHA-256 `9c63ffbab0273222605e9230471f829e8f62f03a16863fe9bb0cda9b0da28f3c`.
Archive SHA-256 `3eff741e7e44c9200be05a8f86075ca2d069e33d4127db3474d356327d3bf4d0`;
candidate SHA-256 `9a870a2b82df4a5347458cd3bdd7360a1e5f465b46636354f177b24a29243148`.
All 27 baseline receipt fields passed or describe the isolation context. Public
activation committed alpha.4 to alpha.5; installed health/MCP passed and committed
recovery replay left state unchanged. A separate fixture observed the new CLI inode
before a durable outcome, killed only its own activation process group with SIGKILL,
then used public recovery to restore the exact prior binary/version/known-good state,
clear the marker and pass restored health/MCP. The source commit field names the
base; the working platform-gate delta is explicitly part of this evidence.

macOS checks passed: eight acceptance-runner tests, the existing all-approvals and
transaction parser negative test, and library/example Clippy with the existing
Rust 1.98 lint exception. The existing live-CLI refusal test was enabled on Linux
and passed there: destination/staged executables block activation, destination/backup
executables block recovery, and rejected attempts preserve state, receipts and the
recovery marker. Recovery succeeds after the test-owned children are killed/reaped.
Final whitespace, generated-index and frozen-source checks passed.
This container result does not qualify clean hardware/VMs, arbitrary power loss,
production signing, resource migration or a model-driven Host session. Ledger
acceptance remains unchanged. The real human Inbox preview remains unapplied;
CLI-404/405 research approval, live Host work and first-stage integration remain
open. Next increment: continue those existing Host/research owners and remaining
platform qualification without treating test receipts as human approval.


## CLI-405 two-source research candidate — awaiting Inbox approval

Base: `10e1b36b`; branch: `codex/two-source-research-preview`. The next research
increment now has actual bounded scholarly content, rather than the older
engineering-only Inbox observation. Native Codex web retrieval read the official
[REALM proceedings abstract](https://proceedings.mlr.press/v119/guu20a.html) and
[RAG proceedings abstract](https://proceedings.nips.cc/paper/2020/hash/6b493230205f780e1bc26945df7481e5-Abstract.html).
The candidate compares training/output emphasis without a quantitative ranking.
It explicitly records abstract-only coverage, unread full papers and no application
recommendation. This is B2 preparation, not complete paper reading or P01 acceptance.

Private fixture: `packages/qiongli-native/target/cli405-two-source-review-10e1b36b`.
`REVIEW.md` states the exact write scope. `review-draft/RESEARCH/realm-rag/` contains
paper notes, comparison summary/matrix, bibliography, retrieval manifest, claim
ledger and boundary note outside the registered project. `capture.json` carries
one literature change, two HTTPS evidence references, no decisions and two next
actions. The isolated empty project was created using existing preview/apply
owners; no prior research or pending engineering observation was changed.

The existing named macOS alpha.5 candidate (the 8c6450f0 Full-cache qualification)
validated this portable capture through the actual CLI with empty PATH and a fresh
Home/config. Binary SHA-256:
`60df1cc42a49aec90a71f7035b31d7bc4e5a01d2147c3f0bcd1b42b26b8c6259`.
The receipt names this older binary; it is not current-head package qualification.
Project `prj_5a00bc9a5788c19a0220a505bf4d7f24`, revision 1;
capture `cap_d0d842b83874b2c0f51a7bf02a1277db53c0c1ede6d108143f3fa5c18b6c45c4`;
plan digest `373815edc7074f98a9eaa65f41b057a8920df94cc30ea7771c5e90df6e1cf55b`;
capture-file SHA-256 `292c6574a43564f400c820ae3c108d0ffdf8b17ecce956225c8848bb35f8f784`.
Preview reports append-pending-history. A real apply without filesystem approval
was rejected, Inbox was empty and the target history file remained absent.

A fresh actual bundled `qiongli_orchestrator_route` call still returned Lite
preview-only with project writes disabled. This candidate was reasoned about by
the current Codex conversation, not passed through Full orchestration; no fabricated
handoff/evidence authentication or model-child run was used. Human approval is still
false in `review-evidence.json`; the new candidate and older engineering candidate
both remain unapplied. The next human decision can authorize this exact Inbox
append only. Canonical consolidation needs its own concrete owner-produced preview,
followed by approved writing and real restart/same-device continuation. Existing
skills explicitly require artifact apply approval; this is the real interaction
under qualification, not a development phase sign-off. Ledger acceptance remains
unchanged. Whitespace, generated-index and frozen-source checks passed; no runtime
code changed and no unchanged suites were rerun.


## CLI-410 preparation — first-stage completion audit at 712265cf

This is an evidence-to-requirement audit, not another status ledger. ADR 0218 and
the roadmap still define CLI-401–405 plus CLI-410 preparation as this stage.
The supplied September 5 acceptance notes enumerate B01–B06, H01–H03, P01–P05
and R01–R04 for the declared baseline scope. L01–L12 belong to later collaboration;
publication/announcement and cross-device work are not authorized by this goal.
The previous turn made progress by producing an actual two-source review candidate.
No human reply to its exact Inbox approval request has arrived.

Current source check: `cargo tree --manifest-path packages/qiongli-native/Cargo.toml
-p qiongli --no-default-features --edges normal,build --locked --offline --prefix none`
exited zero. Its 194 distinct package names contain no tauri/gtk/gdk/webkit/wry/rfd
prefix. This verifies this host's selected normal/build graph, not all target-specific
dependencies, test graphs or a clean-machine run. Prior compilation/test evidence
retains its named source/platform scope.

| Requirement | Evidence available | What still prevents a full claim |
|---|---|---|
| B01 build isolation | Current selected normal/build dependency graph; earlier native CLI builds and tests | Target-specific graphs/builds must match each declared supported package |
| B02 no product window | Named candidate empty args, version, window refusal, MCP and managed-operation probes | One end-to-end approved research/continuation journey on the declared Host combination |
| B03 independent resources | Candidates run outside checkout with installed payload/resource authority | The entire stated path/cwd/old-GUI-removal matrix is not established by empty PATH |
| B04 clean consumer | Real Debian ARM64 container lacks Python/Node/Cargo and exercises the candidate | A container does not satisfy the notes' clean VM/physical-machine requirement; macOS/Windows clean-consumer evidence remains absent |
| B05 trust | Named package tamper, partial approval, candidate staging and installed health refusal probes | Aggregate all required trust cases at the chosen release candidate/target; prior sources are not silently promoted |
| B06 lifecycle | Real isolated Codex/Claude installs/removal and macOS/Linux commit/rollback receipts | Whole supported platform/Host lifecycle and repair combinations are not yet qualified |
| H01 live Host readiness | Real client cache installs and direct Full MCP probes | Current conversation still routes through old alpha.3 Lite behavior; a current Full model-initiated journey is missing |
| H02 clean stdio | Protocol probes and MCP process tests | Approved live business journey remains unrun |
| H03 authentication boundary | Native adapter assigns authentication to the Host; no alternate credentials or transport used | Actual absent/expired account, quota and denied-tool cases are not proved by parser fixtures |
| P01 two-source formal note | REALM/RAG abstract-level comparison, two real source links, valid pending capture preview | Human intake approval, full source review as needed, concrete consolidation preview and approved canonical note are missing |
| P02 domain truth | Candidate explicitly marks abstract-only coverage and avoids unsupported performance ranking; local evidence tests exist | Complete missing/truncated/unconfirmed/sparse-domain matrix in the real journey remains unqualified |
| P03 human approval | Real missing-approval refusal; exact immutable pending review packet | No positive human-origin approval evidence; automatic goal continuation is not approval |
| P04 replay boundary | Existing cross-MCP-process evidence replay refusal and changed-input/approval checks | Evidence authentication is not itself an approval-token lifecycle; killing an approval holder and restarting must be qualified against the actual approval owner |
| P05 restart/second Host | Protocol checkpoint/replay and installed startup/restart primitives | No second actual Host has continued this approved scholarly project after the first exited |
| R01 regression | Existing scoped project/Capture/Graph/MCP/install/migration tests | Their historical results do not prove every retained feature at one named baseline package |
| R02 update/recovery | Real macOS/Linux two-version commit, SIGKILL rollback, health/MCP and live-CLI refusal | SIGKILL at one observed boundary does not establish arbitrary power loss, migration windows or Windows recovery |
| R03 diagnostic privacy | Candidate receipt path-redaction/unrelated-state checks and redacted MCP tests | Live Host research/error evidence still needs scoped privacy review; local private fixtures must not be published as redacted receipts |
| R04 qualification preparation | Existing runner and actual named test candidate/source/hash receipts | Declare the supported source/package/platform/Host combination, fill gaps above; production signing and publication remain separate authority |

Read-only receipt revalidation found all four files present and their SHA-256 values
unchanged: Linux activation `9c63ffba…a28f3c`, macOS SIGKILL `63ae0039…6d73`,
macOS real-client Full cache `210b6bbe…f483e`, and clean Debian candidate consumer
`adae0525…a64dbc` (full paths/hashes appear in their earlier entries). Their scopes
are intentionally not combined into a fictitious current-head universal package.
The two-source capture still matches SHA-256
`292c6574a43564f400c820ae3c108d0ffdf8b17ecce956225c8848bb35f8f784`,
and its target history file is absent. No candidate or approval data was edited.

Next work must close these actual gaps, not add another generic framework or repeat
already-passing suites. Research apply depends on the pending human decision and
Full Host qualification; offline platform support and release-scope preparation can
continue independently. Windows native activation remains explicitly unsupported
in current source; do not claim its earlier App tests as CLI package qualification.
No acceptance count changed. This audit passed whitespace, generated-index and
frozen-source checks and required no runtime suite or publication action.


## CLI-403 direct Linux VM consumer and Unicode path evidence

Base: `6389d186`; branch: `codex/linux-vm-consumer-evidence`. The preceding audit
identified VM and path qualification gaps. Read-only inspection of the already
running `podman-machine-default` found Fedora CoreOS 43.20251110.3.1 on Linux
ARM64 and `/usr/bin/python3`. It cannot establish the no-Python clean-machine B04
requirement, and no installed system dependency was removed or hidden to claim it.

The existing Linux two-version candidate binary was copied over SSH into a unique
private `/var/tmp/qiongli-cli-vm-6389d186-*` directory inside that VM, under a Chinese
name containing a space. Host and guest SHA-256 matched
`8c990c0ceff7275ffb9193008a196549fb488791f76377f086567d8f86599a13`,
the successor binary in the existing Linux activation receipt. Commands executed
directly in the VM, not in a new container, from that unrelated fixture directory
with cleared environment, empty PATH and isolated Home/config. Python orchestrated
SSH on the development Mac; Qiongli did not invoke a language runtime in the VM.

All eleven recorded checks passed: exact version, help, embedded content, UI refusal,
project create preview, missing-approval refusal, approved empty engineering-project
creation, project show in a new process, Full MCP, export preview and approved export.
After missing approval, no project directory existed. MCP emitted only three JSON
responses, no stderr, exactly 32 tools and a Full route without Lite upgrade fields.
The portable export manifest was verified in the VM after the real CLI operation.
Only the fixture-owned VM directory was written; existing containers, suspended VMs,
Host configuration, system dependencies and research data were not changed.

Local evidence: `packages/qiongli-native/target/cli-linux-vm-consumer-6389d186/`
contains the runnable `consumer-script.py`, exact private VM path, environment
observation, individual command outputs and `receipt.json`.
Receipt SHA-256: `2edda697c0e9a1f458560e9f2ee61cd1a7e3dfbf0cb602a12f553149d9c1eac6`.
The private path and raw outputs are local engineering evidence, not publishable
redacted diagnostics. The receipt explicitly limits this to the candidate binary,
an existing Python-containing VM and an empty engineering project. It adds native
VM/Unicode-path evidence to B02/B03 and startup/export checks; it does not qualify
B04, package installation/update in that VM, P01 research, human approval or P05.
The pending research previews remain unapplied; acceptance status is unchanged.
Whitespace, generated-index and frozen-source checks passed; runtime source did not
change and existing source suites were not repeated. Next: a genuinely suitable
clean-consumer environment and the pending actual Host/human research chain.


## CLI-405 targeted main-paper review supplement

Base `3695b0b4`; branch `codex/research-source-review-addendum`. Official REALM
and RAG PDFs were read for methods, experimental setup and relevant results.
The new private `target/cli405-two-source-review-10e1b36b/fulltext-review-addendum.md`
(under `packages/qiongli-native/`) binds findings to sections/pages and distinguishes
training stages, extractive/generative output and reported comparison from causal
attribution. It records unread supplementary/code/data material, no reproduction
and a failed RAG table screenshot request; it makes no numeric table claim.
SHA-256: `0848acf81ea72cb7376e85a9c7ba04defeb4ec471650375ea505acc6a3d8f1a3`.
This advances source review for P01 without claiming the entire B2 workflow or a
Full Host handoff. The existing capture SHA-256 was rechecked unchanged and its
history file remains absent. No approval was received, no capture was replaced,
and no research state was applied. A richer future capture needs a fresh preview.
Only this evidence note is committed; whitespace, generated-index and frozen-source
checks passed. Existing runtime suites and unchanged package checks were not rerun.


## CLI-403 macOS to Windows x64 cross-build

The maintainer requested attempting the cross-build on macOS after the tooling
permission question. Installed Rust `x86_64-pc-windows-msvc` standard libraries and
cargo-xwin 0.23.1, reusing LLVM 23.1.0 and the existing cargo-xwin Windows SDK/CRT
cache. Rust is 1.98.1. Source: `59a7080e6f7c8a02cbc0c4c17cbaee6318f29f3e`;
branch: `codex/windows-cross-build-evidence`. No product source, Cargo.lock or default
build configuration changed. Initial offline build lacked curve25519-dalek-derive
0.1.1; a locked online retry fetched it and successfully built the release CLI.

Both runs used `cargo xwin build --manifest-path packages/qiongli-native/Cargo.toml
-p qiongli --bin qiongli --no-default-features --release --target
x86_64-pc-windows-msvc --locked`, with existing LLVM/Rust bin directories on PATH
and `XWIN_CACHE_DIR=/Users/pengjiaxin/Library/Caches/cargo-xwin`. Normal CRT output
is under `packages/qiongli-native/target/cli-windows-cross-59a7080e/`; its binary
SHA-256 is `40d83562da4f1d45ed19ef31db643e843280b5b1e7d775cf52830183023c2bce`.
It imports VCRUNTIME140.dll and dynamic UCRT entrypoints.

A second successful offline build added `RUSTFLAGS="-C target-feature=+crt-static"`
and used target directory `packages/qiongli-native/target/cli-windows-cross-static-59a7080e`.
Its `x86_64-pc-windows-msvc/release/qiongli.exe` is 15,579,136 bytes, SHA-256
`4c0a2f692a2543a305a672124ea4b270ff4186d9d9b9a7101119a1c433224de1`.
`llvm-readobj` confirms AMD64 PE/COFF, console subsystem and no VCRUNTIME/MSVCP/
api-ms-win-crt imports. Eight Windows OS DLL imports remain. The SDK static libraries
reference missing Microsoft PDBs (LNK4099); linkage succeeded, while CRT debugging
symbols are incomplete. No warning was suppressed or represented as a runtime pass.
The target directory retains `pe-inspection.txt` and `cross-build-receipt.json`.

The selected Windows normal/build dependency graph has 186 distinct package names
and no tauri/gtk/gdk/webkit/wry/rfd prefix. Its tree is saved with the first build.
Compilation and PE inspection are complete; no guest/hardware execution, installer,
signed candidate, research approval or Windows update qualification is implied.
Next: exercise the Windows artifact in a declared Windows environment and qualify
its actual package/consumer behavior. Existing human research approval remains
pending. Local whitespace, generated-index and frozen-source checks passed; no
unchanged source tests were rerun. Publication remains unauthorized.

## CLI-403 Parallels Windows installation test

Base `dee4d81c`; branch `codex/parallels-windows-install-test`. The maintainer
requested installation testing in Parallels Desktop. Resumed the existing Windows
11 Enterprise ARM64 guest (10.0.26200.8037) and tested the preceding static-CRT
x64 executable under Windows x64 emulation. Its SHA-256 matched
`4c0a2f692a2543a305a672124ea4b270ff4186d9d9b9a7101119a1c433224de1`.
Commands ran as the interactive user in an isolated test-owned directory, with
private HOME/config overrides and an empty child PATH; real research and Host
configuration were not changed.

Thirteen command observations matched their expected outcomes: version/help,
embedded content, deliberate UI refusal, install status, installation-plan
refusal, Full MCP initialize/list/route, project creation preview, refusal without
filesystem approval, approved empty-project creation, reading in a new process,
and export preview/apply. Full MCP exposed 32 tools and routed through the Full
runtime. The exported portable manifest existed. No missing-runtime-DLL startup
failure occurred. This is process restart evidence, not a Windows reboot test.

Managed installation did **not** succeed: `qiongli app plan cli-install` returned
`source-build-read-only`. This source executable has no embedded release authority,
source identity or candidate identity. Its trust gate was preserved; copying and
running the executable is not a completed supported installation.

Private raw outputs, PowerShell probes and `installation-test-receipt.json` are
under `packages/qiongli-native/target/parallels-install-dee4d81c/`. This existing
guest is not a clean consumer VM; x64 emulation is not native x64 hardware evidence.
No signed candidate, actual model Host integration, Windows update qualification,
research approval or program acceptance is claimed. Next: prepare a trusted Windows
test candidate through the existing release owner and repeat managed installation.
Only this evidence note changes tracked files; unchanged runtime suites are reused.


## Windows partial closeout and next development increment

The maintainer explicitly paused the Windows lane after `d6d50131`: **partially
complete**, not accepted or supported-installation qualified. Compilation and
Parallels Windows ARM x64-emulation runtime checks are complete; trusted Windows
installation/update, clean-consumer and native x64 qualification remain open.
CLI-403 stays active overall; its ledger blocker records the paused platform lane.
No new ledger state or acceptance record is introduced.

The next bounded outcome returns to CLI-404/405: use the current candidate through
a real Full Host and resume the isolated project without a Qiongli window. First
inspect the existing candidate/client-cache and project/checkpoint owners, then
exercise a read-only Host session and continuation; fix only concrete gaps found.
Reuse the existing review packet for any later research apply, which still needs
its exact human decision. This priority change is not approval of that packet.
CLI-410 qualification preparation follows; CLI-406 collaboration expansion retains
its baseline dependency. Windows retesting is no longer the immediate next task.


## CLI-404 model-initiated read and Host approval boundary

Base `4b201c62`; branch `codex/live-host-approval-boundary`. The existing named
macOS candidate from `8c6450f0` was configured as a process-scoped Full MCP server
for authenticated Codex, with only project-read and route tools enabled. No global
Host configuration changed. Automatic approval review rejected use of the existing
research fixture because external disclosure lacked specific authority; it did
not execute. An approved safer run used a newly created empty synthetic article
project, isolated configuration, an ephemeral Host session and a read-only sandbox.

The actual model discovered and invoked `qiongli_project_read`. The Host rejected
it with `MCP tool call requires approval, but approval policy is never`; no project
data returned. Host process exit zero is therefore not successful product-read
evidence. The event log contains exactly one completed, failed MCP call and no
fallback. A local assertion verified that outcome. This advances model-initiated
Full tool discovery evidence only, not project continuation or research acceptance.

The native automatic-approval option conflicts with the explicit read-only sandbox
flag. A proposed retry using its workspace-write default was rejected by automatic
approval review as unauthorized permission expansion and did not execute. No trust
guard or product code was weakened. Next: obtain explicit authority for the Host
approval mode on this empty fixture before retrying; research disclosure and the
pending research write remain separately unapproved.

Private invocation, JSONL events and receipt are under
`packages/qiongli-native/target/cli404-live-read-4b201c62/`. These bind an older
candidate, not current-head package qualification. Windows remains paused and no
acceptance state changes. Only this evidence note is committed; runtime suites
are unchanged.


## CLI-404 authorized model reads across independent Host sessions

Base `ca4f5396`; branch `codex/verified-live-host-read`. The maintainer explicitly
authorized the previously described automatic approval mode and its workspace-write
sandbox for the new empty engineering fixture. Two authenticated ephemeral Codex
sessions used the existing `8c6450f0` macOS candidate through process-scoped Full
MCP. Each model called only `qiongli_test.qiongli_project_read` once for
`prj_67c275569f1d50bfbffe8089a8ff8cd2`. Both calls completed successfully with
structured project results, no tool error and no fallback. The second session
started after the first process exited and had a different session identity.

Both returned exactly the same project state: semantic/library revision 1, article,
idea/active, empty research overview. Neither model invented continuity artifacts
or research handoff status absent from the response. SHA-256 snapshots of every
project/config file matched before and after both runs, including the file set.
The actual invocation records retain the automatic approval option; no persistent
user configuration or product authorization check was changed. Host startup stderr
included unrelated plugin OAuth/state-database warnings; these did not prevent the
observed Qiongli calls and are not claimed as clean Host startup evidence.

Private evidence remains under
`packages/qiongli-native/target/cli404-live-read-4b201c62/`: the authorized/restart
invocations, JSONL event logs, before snapshot and `authorized-receipt.json`. The
runnable `verify-authorized-read.py` passed checks for exact tool identity, one
successful call per session, distinct sessions, identical structured results and
unchanged files. This establishes real model-initiated reads and same-Host restart
access for an empty fixture on an older named candidate. It does not establish
checkpoint recovery, second-Host research handoff, current-head qualification or
research acceptance. Prior research disclosure/write approvals remain pending.

Next bounded increment: exercise the existing checkpoint owner on synthetic inputs
before any real research continuation. Keep Windows qualification paused and reuse
existing candidate/build results until their inputs change. Only this evidence
note changes tracked files; no runtime source fix was required.


## CLI-404 synthetic checkpoint interruption and reissue

Base `67d397d5`; branch `codex/synthetic-checkpoint-recovery`. The maintainer
requested the next synthetic-data verification. Reused the named `8c6450f0` macOS
candidate in a fresh empty project and isolated config. Direct Full MCP processes
used an explicitly synthetic `other-local` Host descriptor; its ready fields are
test inputs, not evidence that another real Host was installed or trusted.

Fifteen actual MCP observations passed. After the start response, SIGKILL ended
the owned server with exit -9. A new process discovered the same generation-2
checkpoint, and `next` returned the identical A1 handoff, document and handoff
digests without advancing the run. Seven refusals covered wrong digest, wrong
generation, changed Host binding, pause while a task is active, resume while
running, duplicate start and the old reference after cancellation. Rejected
operations left the observed checkpoint unchanged.

Cancellation advanced to generation 3; another SIGKILL after its response retained
that exact cancelled state in a new process. `next` on the current terminal
reference returned no handoff and could not continue. Project/library revisions
remained 1 and no capture was created. The active run reports `canPause=false`,
`canRecover=false` and `recoveryRequired=false`: this Host path uses unchanged
handoff reissue after process exit. Positive pause/resume or destructive recovery
is not inferred from the negative calls. Neither kill targeted an in-flight write,
and this does not qualify arbitrary power loss, candidate submission, real Host
identity, or scholarly continuation.

Private `probe.py`, individual responses/stderr, `verify.py` and `receipt.json` are
under `packages/qiongli-native/target/cli404-checkpoint-67d397d5/`. The verifier
passed all 15 observations, seven exact refusal codes and both signal exits.
No source fix was needed; tracked changes contain only this evidence note. Next:
validate synthetic candidate/evidence submission across the existing handoff owner,
including stale and cross-process evidence refusal, before research continuation.
Windows remains paused; pending research authorization and acceptance are unchanged.


## CLI-404 synthetic candidate and evidence submission

Base `37433b34`; branch `codex/synthetic-evidence-submission`. The previous goal
turn produced checkpoint interruption evidence and therefore counts as progress.
This increment exercised the existing candidate/evidence owner through the named
`8c6450f0` macOS binary, using a fresh empty project and synthetic Host descriptor.
The first script used candidate schema 1 and was correctly rejected with
`host-candidate-binding-mismatch`; inspection confirmed the canonical candidate
schema is 2. Its process was reaped and test run cancelled. The corrected run uses
a separate directory and retains the failed evidence without weakening validation.

The schema-2 run passed authenticated read/submission in one MCP process. It
rejected cross-project reads, forged evidence, evidence replay from a second MCP
process, and stale read/submission after successful advancement. SHA-256 snapshots
prove forged/cross-process and stale requests changed neither project nor config
files. Rejected replay did not consume the original process's valid evidence: the
original candidate was accepted once, completed one task and issued the next
handoff. Candidate plaintext was absent from project files, semantic/library
revision remained 1, no capture existed, and the engineering run was cancelled.
This is protocol evidence, not model-produced research or human writing approval.

Private runnable evidence: `packages/qiongli-native/target/cli404-evidence-v2-37433b34/`
contains `verify_submission.py`, responses and `receipt.json`; it reuses the previous
private probe's process/CLI helpers. The failed schema-1 attempt remains at
`target/cli404-evidence-37433b34/` under the same native package. Product code needed
no change, and unchanged source suites were not repeated.

The next first-stage requirement is the actual approved scholarly journey, not
another repetition of synthetic baseline checks. The pending two-source capture
SHA-256 remains `292c6574a43564f400c820ae3c108d0ffdf8b17ecce956225c8848bb35f8f784`
and its target history file remains absent. Its exact Inbox approval and external
research Host disclosure remain unapproved; synthetic-fixture approval does not
cover either. Canonical consolidation, actual research continuation and named
package qualification remain unproved. Windows stays partially complete/paused.
The first-stage goal is not achieved and no acceptance record changes.


## CLI-403/404 current-source macOS candidate qualification

The pending research capture and absent history file were revalidated unchanged;
automatic continuation did not authorize disclosure or Inbox writing. Independent
CLI-410 preparation could still advance: the existing `native_candidate_acceptance`
owner rebuilt source `dfebb17c30b6748ee72ac0485f24d11ed9405205` offline and generated
a fresh macOS aarch64 alpha.5 test candidate. Branch: `codex/current-macos-candidate-evidence`.

Two failed setup attempts were preserved. `/private/tmp` was rejected by the
explicit-target ancestor permission policy after successful compilation. A private
Work directory passed that boundary, but mise shims failed Codex version discovery
under the isolated Home. The successful run uses installed executable paths from
`mise which`, the existing `.venv/bin/python` and Plugin Creator validator. No
permission check was weakened, global tool installed, or existing client modified.

Final evidence: `/Users/pengjiaxin/Work/qiongli-cli-baseline-dfebb17c-real-clients/acceptance-evidence.json`,
SHA-256 `90211013b558c3b3f85842ca6bcac54c9bb8815be2d12b96615243959b6f3b10`.
All 27 baseline fields passed their declared checks/environment observations. Codex
0.153.4 and Claude Code 2.1.263 each passed isolated install/cache/remove, 14 Lite
and 32 Full tools, and Full routing. Artifact hashes were independently rechecked:
archive `8ba271c3f9021497d2fd9049eaa0ef4f33b24e4161cff64211dff0676545198c`;
candidate `57b1a9549f9ea134ed68d7b28cbd4ff297e2fc31d118bd32ac4a2c023c2e9cc6`.
The release binary SHA-256 is
`1a3ba25cf36026cb29c6e2fc1e8e40ca33b9d3600bd000287498b29ee552946d`.
Signing keys stayed ephemeral/in-memory and `publication_allowed=false`. Native
activation was not run without a predecessor; prior activation evidence stays scoped
to its earlier source. Isolated Home installation is not clean-machine qualification.

A real authenticated Codex model then called this candidate's Full project-read
tool against the already-authorized empty synthetic fixture. Exactly one call
completed, returning the expected project/revision. Before/after project/config
hashes matched. Private invocation, events and receipt are under
`packages/qiongli-native/target/cli404-current-candidate-dfebb17c/`. This updates
model-read evidence to the new candidate, not scholarly continuation or approval.
The ledger's CLI-404 blocker now reflects that precise progress. No acceptance
count changed. Research disclosure/Inbox approval remains pending; canonical
consolidation and real research handoff remain unproved. Windows remains paused.
