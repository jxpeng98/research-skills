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
