# Native Runtime

The Qiongli 2 executable, CLI, Desktop service, Full MCP, project state, and
embedded resources live under `packages/qiongli-native/`.

## Local Pattern

- `apps/qiongli/src/command.rs` owns public CLI parsing and help.
- `apps/qiongli/src/desktop.rs` owns the shared App service; Tauri commands in
  `desktop/tauri_adapter.rs` adapt it instead of duplicating product logic.
- `crates/qiongli-runtime/src/contract.rs` and `apps/qiongli/src/mcp.rs` own the
  native MCP registry and dispatch boundary.
- [Full MCP profile routing](./full-mcp-profile-routing.md) defines how Full may
  reuse Lite validation without returning a Lite profile result.
- `crates/qiongli-project/src/service.rs` owns project mutations and revision
  checks; App, CLI, and Full MCP route through that service.
- `crates/qiongli-runtime/src/zotero/companion.rs` owns the loopback Companion
  boundary. Only loopback endpoints may be contacted.
- [All Chat State v1](./all-chat-state-v1.md) is the bounded, provider-neutral
  ACP collaboration projection; existing orchestration and project services
  retain scheduling and mutation authority.
- [ACP v1 client boundary](./acp-v1-client.md) defines the fixed development
  presets, stable-v1 negotiation, event normalization, and fail-closed
  permission/cancellation behavior. It is not packaged provider support.

Native reconciliation uses atomic no-replace renames on macOS and Linux for
activation, compensation and rollback. A concurrently created target, including a
symlink, must not be overwritten; losing source files remain intact. Unsupported
kernel/filesystem operations fail closed. This does not enable the public Linux
activation/recovery commands, which still require platform update qualification.

Linux internal reconciliation inspects current-user executables through the visible
`/proc` PID namespace, anchoring status/executable reads to one process directory.
Deleted executable paths still count as running. Ambiguous executable access,
malformed identity, ptrace-only visibility and exceeded scan bounds fail closed.
The scan is a snapshot: it neither prevents later launches nor inspects processes
outside the visible namespace. It does not stop processes or confer write approval.

Public writes use preview, digest-bound approval, revalidation, and fail-closed
errors. `qiongli_project_capture_apply` is a real Full MCP project write and
must never be described as read-only. ToolHost remains read-only in-process.

Host handoff instructions classify project data, PDF excerpts, web pages, repository
content, dataset documentation, imported notes, tool results and prior candidate
hashes as untrusted evidence. Embedded approval or tool instructions confer no
permission. The server-owned handoff, project scope and evidence ledger remain the
authority for orchestration reads; capture writes still require their separate
preview/digest/approval checks. Candidate acceptance is not human approval. These
server checks do not attest that a Host-supplied approval boolean came from a human;
CLI-404 must separately verify that Host interaction. Both packaged native Host
adapters carry the same untrusted-source rule before the first handoff, including
refusal to interpret source-embedded system messages or approval claims as control.
Host evidence authentication is local to the MCP process that performed the read.
A second process cannot submit the first process's references merely by loading the
same checkpoint; it must perform its own authorized reads. Rejected replay must not
advance the checkpoint or consume the original process's references. Authentication
binds an observed result, not a claim that source bytes remain current forever.

Local Workflow/Skill customization is owned by `WorkflowVariantStore`. It may
override only canonical Markdown instruction resources, and installed
Skills/Plugin outputs must record the exact optional variant digest. Saving a
variant never bypasses explicit managed reconciliation or fresh Host Ready
verification.

`GlobalSettingsStore` owns creation and security validation of the shared
native `v2` state root. Managed Skills and other sibling writers must prepare
that root through this owner instead of creating it with platform-default
permissions.

The native 2.x runtime must not fall back to Python or Node in production.
Legacy packages can provide migration evidence but are not runtime dependencies.

## Pre-Development Checklist

- Trace all App, CLI, MCP, and ToolHost callers of the shared owner.
- Check `content/mcp-contracts/` and affected Skills for the same public name.
- Preserve redaction, loopback-only networking, ownership, and revision checks.
- When a writer shares the native state root, test that another normal owner can
  read the root after the write on every Tier 1 platform.

## Build and Entry Boundary

The `qiongli` package defaults to CLI/MCP without GUI dependencies in its normal,
build, or selected-package test graph. Empty arguments print help. The optional
`desktop` feature enables Tauri, the file picker, and `qiongli-desktop`;
`custom-protocol` includes `desktop` for existing packaged App builds.
Desktop-enabled empty arguments retain the App launch behavior.

Shared App services, DTO/schema generators, CLI inspection, MCP dispatch and
preview/approval/CAS remain available without the renderer. `ui` fails without
the desktop feature; `ui --startup-check` reports shared service readiness only.
Embedded resource, release authority and Companion checks always run in the
build script. CLI-only compilation is not standalone package qualification.

`SignedNativeReleaseEnvelopeV1::verify_extracted_artifact` verifies an approved
native artifact directory against the signed release and launch grant without
requiring the original archive. It reuses release-key/generation/time policy and
the artifact owner's full file-tree validation, binding the manifest, binary and
resource digests to the expected artifact and requested launch scope. Its result
is a scoped launch grant, not running-process identity, candidate source provenance
or approval to write.

Candidate installation atomically persists the exact signed candidate beside the
native payload directories using the existing private-file and no-replace rename
owners. Conflicting metadata is refused before payload changes. Identical metadata
replays; legacy installs acquire it only through a freshly verified candidate apply.
The record is retained with lifecycle receipts after uninstall. It is not itself
authority: `verify_installed_native_candidate_product` requires an active payload
receipt, a freshly verified candidate/source/Host grant, and the matching executable
at the fixed managed path. Recovery journals, changed bytes, linked records, expired
signatures and removed payloads refuse authority. Temporary or orphan records do not
restore authority; successful replay can complete an interrupted metadata commit.
The app boundary supplies current_exe(), embedded identity, trust roots and time.
`verify_running_packaged_product` selects the native verifier for executables under
the fixed managed payload root; other executables retain the desktop/managed-shim
verification path. A path match only selects verification and never grants authority.
`VerifiedPackagedProduct` stores verified artifact/source/resource facts and scoped
Host capabilities, not a fabricated desktop manifest. Shared install, migration
and reconciliation owners consume those facts. Native plans bind the signed
candidate digest; desktop plans retain their existing control-document digest.
Each prepare/apply still re-verifies product authority and existing approvals/CAS.
Source builds without embedded authority remain read-only.

The existing CLI install/remove/PATH owners also support the native payload as
their source. The installed command copy uses the unchanged v3 receipt: its fixed
command path, version and binary digest must match the fixed versioned native
payload. This resolver is only a hint; every product operation still revalidates
the source's signed candidate, active receipt and embedded identity. Arbitrary
copies, changed command/payload bytes, absent receipts and legacy receipts cannot
establish native product authority. Desktop authority-bearing receipts retain their
existing route. Native source discovery no longer invents a sibling `qiongli-cli`.
Shell profile updates keep their existing preview/digest/approval rules. This does
not qualify Windows PATH handling, update/rollback, human approval across Hosts or
a release.

`stage_native_release_candidate_local` reuses the same payload preparation and
signature-record persistence as complete candidate installation, without changing
Host sources, registrations or the installed command. Its trusted caller must
obtain exact-candidate filesystem-write approval. Versioned payloads can coexist;
the existing payload executor can roll back a staged install while preserving the
prior integration. Modified staged bytes are refused, not deleted. Full candidate
apply retains its original source/registration compensation and metadata commit
ordering. Staging alone does not switch the active command/integration or qualify
live-process update and rollback; public update command wiring remains pending.

`ManagedNativePayloadExecutor::verify_receipt_owned` verifies an active predecessor
payload without requiring the successor's embedded resource pack. It reuses strict
artifact tree, canonical manifest, binary and receipt checks, and refuses pending
recovery. Its result proves receipt-owned integrity only; it never supplies signed
candidate, launch or write authority. Install, repair, removal and running-product
verification retain their existing pack-bound and signature checks. Activation must
also bind this predecessor evidence to the selected Host and managed CLI receipts.

`install candidate activate-preview` verifies the signed release inputs and the
current process at the fixed staged payload path, then binds the selected older
payload, Host source/registration and installed CLI receipts. It requires
`--previous-install-id`, the same artifact stream/platform, and an increasing
version. The existing CLI plan owner binds command bytes, receipt bytes and retained
backups; its preconditions are rechecked after predecessor verification. The output
is a read-only identity snapshot with `preflight_digest_sha256`, never an approval
or transaction reservation. It does not include managed Skills/settings changes;
activation preparation must bind those separately and revalidate all observed state.
The public contract is Rust-generated `candidate-activation-preview-v1`; existing
candidate preview/stage/apply wire formats remain unchanged.

`install candidate activate-prepare` accepts the same release and predecessor inputs,
`--expected-preflight-digest`, and filesystem-write approval. It creates the state
root through `GlobalSettingsStore`, rechecks identities under the replacement lock,
and uses the existing reconciliation owner to stage registered Skills, the selected
Host, and the CLI binary/receipt pair. Active update transactions and non-increasing
release generations refuse. It leaves active destinations and update state unchanged.
The output lists staged surfaces and binds the candidate, preflight, journal, update
revision and workflow revision/variant into a separate activation approval digest.
Workflow/update state is rechecked after staging. An existing transaction refuses
without overwriting its journal. Fresh failed preparation uses existing guarded
cleanup. The Rust-generated public contract is `candidate-activation-prepared-v1`;
reconciliation v2 wire semantics remain unchanged. The macOS activation and recovery commands are described below.

`install candidate activate-discard` cancels an unactivated v2 preparation using
`--transaction-id`, `--expected-journal-digest` and filesystem-write approval. It
needs no new candidate authority because it cannot activate or adopt a product.
Under the replacement lock it rejects any active transaction, activation record,
backup or unknown transaction-root entry. Cleanup uses the exact checked journal
and the existing whole-set ownership checks, allowing already-removed staged files.
The journal digest is checked again before removing the journal and empty transaction
root. Missing journals refuse rather than claiming a successful replay. The shared
journal reader rejects linked or insecure state/update/staging/transaction directories.
The Rust-generated output contract is `candidate-activation-discarded-v1`. Started
activations still require recovery; discard never rolls back active destinations.

Native candidate preparation now emits reconciliation journal v3. Its required
`native_release` binding includes the verified candidate digest, prior update
revision/accepted generation/known-good identity, and the next signed release's
version/channel/generation/archive/resource digests. The existing journal hash and
activation outcome bind these fields together. v1/v2 omit this field and retain
canonical bytes and behavior; v3 requires the CLI pair and release binding, while
unknown versions and mixed version/field shapes refuse.

Activation checks the prepared update revision and prior release state before
reserving the transaction through CAS. Successful health first records a durable
committed outcome; cleanup then advances accepted generation and known-good state
while clearing the reservation in one state CAS. Failure or undecided recovery
preserves prior release metadata. Recovery can finish a committed cleanup without
rerunning health, and completed replay does not increase the revision. Unexpected
release-state changes refuse before cleanup. Preparation transaction IDs now include
update/workflow revisions so a rolled-back attempt can be prepared again without
overwriting historical records. The caller still owns fresh candidate/approval verification. The macOS public native activation/recovery commands enforce these boundaries;
shared write exclusion is described below.

Native activation, recovery, discard and candidate preparation now acquire a fixed
private `HOME/.qiongli/native/.installation.lock` before the config-root replacement
lock. Activation copies its existing journal-bound start record to
`active-installation.json` in that Home directory before live changes. Recovery
requires the matching marker; successful cleanup/state completion removes it.
Interrupted activation therefore excludes participating writers using a different
config root even after the process lock is released. No new journal or public wire
schema is introduced.

The shared Unix managed-write guard covers managed-operation apply, candidate
stage/apply/remove, and Desktop confirmed Skills, workflow-variant, CLI and packaged
Host mutations. It rejects active update state and any Home activation marker, then
rechecks state under the config lock. Read-only plans remain available for installed
CLI health. Non-Unix managed writes retain their existing behavior. Public native activation is macOS-only; legacy interrupted-update recovery uses
the approved CLI entry described below. This lock coordinates participating processes;
it is not an operating-system access boundary against unrelated writers.

Engineering `install native apply/remove` also takes the Home/config guard using
the command environment; preview/verify remain read-only. Apply validates release
authority, plan digest and approval before acquiring it. Legacy migration apply,
continue (including cleanup/finalize), and recover use the same guard in their shared
CLI/Desktop owner. Apply acquires it after product/approval validation and before
provider or Host writes; recovery loads its receipt before acquiring it and never
bypasses pending native activation. Engineering roots resolving to `HOME/.qiongli/native/payloads` use that actual Home
for coordination, even when the invoking Home differs; other engineering roots
retain the invoking Home scope. Existing managed-root approval rejects unsafe links.

The old macOS replacement executor takes the same Home-then-config locks after
parent exit, refusing a native activation marker before switching files. Handoff
failure restoration also takes both locks before altering state. Health retains its
existing independent completion path. Legacy recovery uses the approved CLI entry described below.

The shared update guard acquires Home then config locks without rejecting an
existing update transaction; each update stage still validates its own state/CAS.
CLI and Desktop channel/cancel, signed verify/stage and staged reconciliation use it.
Downloads acquire it only after manifest verification, across reservation and private
staging setup, and release it before archive transport so concurrent cancellation
remains available. The reservation/CAS owner protects subsequent private download
writes. Installation waits for its guarded staged child before acquiring its own
guard, then rechecks the exact state revision before advancing or creating handoff
files. Status/check remain read-only; token-bound legacy health completion remains
available while the replacement helper holds locks. Authority-free signed paths
retain their existing refusal before any mutation.

Initial candidate-directory creation tolerates a concurrent `AlreadyExists` only
by revalidating the resulting directory's type, ownership and private permissions.
It never adopts a link or relaxes the security check.

Legacy Desktop rollback now restores the old application without clearing the active
transaction or deleting its evidence. A shared completion step first finishes Host
reconciliation cleanup, removes the failed application, syncs the transaction directory,
and only then clears the failed transaction through CAS. Cleanup failure preserves the
active transaction and retained journal/health contract; files and links substituted at
the failed-application path refuse. Completed rollback keeps the journal and health
contract as evidence rather than recursively deleting the transaction root. The approved legacy recovery CLI reuses these owners.

The legacy macOS executor now writes its serialized replacement journal to the shared
private Home activation marker before replacing application files. Shared marker
binding and clearing compare exact bytes under the Home lock; a different native or
legacy transaction cannot clear it. Successful commit, complete rollback and verified
pre-activation restoration clear their own marker. Panics and incomplete cleanup keep
it, excluding participating writers across config roots after process-lock release.
The marker reuses the existing replacement-journal format; it adds no public schema.
Pre-activation restoration now reports errors and requires the destination/staged
layout, absent backup and successful state CAS before releasing protection. Recovery through these owners and the CLI dispatcher is tested; real
process-kill qualification remains pending.

`recover_legacy_health_interruption` is a callable library owner for an interrupted
legacy HealthWindow (or its RecoveryRequired reservation). It requires the exact
Home marker digest, validates the marker's replacement journal against the configured
store, checks the canonical reconciliation journal/digest, backup ownership and the
installed new canonical binary hash. It reserves RecoveryRequired through state CAS
before rollback so late legacy health cannot commit, then reuses Host/application
rollback, cleanup and exact marker clearing. It does not rerun health. The public CLI enforces caller-owned filesystem approval; real process-kill
qualification remains pending. Unsupported layouts/states refuse rather than guessing a completed recovery.

Legacy rollback persists private `legacy-rollback-v1.json` before moving the application.
Its strict version-1 shape binds the Home marker digest, a hash of the prior accepted
release metadata, and the old canonical binary digest. Normal rollback also reserves
RecoveryRequired through CAS before moving files, excluding late health commitment.
Recovery validates the record and old binary before continuing a parked-new-application,
restored-old-application, or state-cleared/marker-retained layout. A retained failed
application must still match the new canonical binary before deletion. Successful
rollback keeps the record and journal as evidence. Unknown versions, changed marker/
release bindings, substituted paths and binary drift refuse. Tested checkpoints are
between filesystem operations; deletion interrupted inside a failed application tree
can still require manual recovery if its identity cannot be verified. The public CLI is described below; full process qualification remains pending.

`recover_legacy_committed_cleanup` is the library entry for an already accepted legacy
update. It validates the exact marker, configured journal, canonical Host journal and
matching version/pack, complete last-known-good identity (including channel), installed
new binary and any remaining old backup against retained identity evidence. It only
finishes cleanup and clears its own marker; it never runs health or changes accepted
state. Old identity is now recorded before activation so this evidence exists on both
commit and rollback paths. Verified pre-activation restoration removes that snapshot
with its old handoff contract to allow a fresh attempt.

Committed cleanup retains transaction evidence/downloads and supports an already
removed backup. Cleanup stopped inside a backup tree still refuses if identity can no
longer be established. Bounded garbage collection remains separate work; retaining evidence does not claim complete package qualification.

Both legacy recovery owners inspect the current user's mapped application files under
the installation locks before changing state/files. A fixed `/usr/sbin/lsof` invocation
uses the existing bounded child collector with a 30-second deadline and separate 8 MiB
stdout/stderr limits; existing Host probes retain 512 KiB. Nonzero exit, stderr,
malformed/empty output, invalid encoding or an exceeded bound refuses recovery. Matching
covers destination, backup, staged and failed-application directories by path components.
C-locale hexadecimal encoding of non-ASCII path bytes is accounted for; target control
characters refuse because reliable matching is unavailable. No processes are killed.
This is a current-user snapshot, not prevention of launches after inspection or a claim
of visibility into other users' processes. Full process qualification remains pending.

`qiongli update recovery-preview` reads the private Home marker and configured update
state without creating directories or acquiring write locks. It identifies a legacy
transaction, exact marker SHA-256 and `rollback` or `committed-cleanup` mode. This is a
recovery description, not proof that process, application or Host evidence will pass.
`qiongli update recover --expected-marker-digest <sha256> --approve-filesystem-write`
requires explicit approval and the exact marker, then delegates to the existing recovery
owner, which rechecks locks, state, process and filesystem identities before mutation.
Missing/duplicate options and malformed digests refuse. Source builds can recover owned
legacy evidence without obtaining new release authority. Run recovery from a separate
CLI outside the affected application paths; a running binary inside them is refused.
Native payload activation markers remain unsupported by these legacy commands.

Both outputs use the Rust-owned additive `update-recovery-v1` public JSON contract,
with generated Draft 2020-12 schema and preview/recovered golden fixtures. Recovery
retains the prior cleanup and partial-tree limitations; it does not grant package or
program acceptance.

On macOS, native activation and recovery also reuse the bounded installation-process
inspector while holding Home/config locks. They check every journal CLI binary's
destination, staged and backup path. A mapped executable refuses before activation
records or recovery mutations; an inspection error also refuses. Receipt files and
Host content are still governed by their existing ownership checks. This guard does
not stop processes, prevent subsequent launches or claim Host reload completion.
Other platforms retain their existing coordinator behavior and have not gained native
process inspection; their public activation must not claim this macOS evidence.

On macOS, `install candidate activate` accepts the same signed candidate/archive/notes,
Host target and predecessor as preparation, plus its `--transaction-id`,
`--expected-journal-digest`, `--expected-approval-digest` and all three approvals:
`--approve-filesystem-write`, `--approve-client-config-change`, `--approve-host-trust`.
Run it from the verified staged candidate binary. It freshly verifies candidate and
running product authority, then rechecks candidate/Home/release identity, preflight,
workflow and update revisions under the installation locks. It uses the unchanged
preparation approval hash and starts no activation records on mismatch. The coordinator
requires installed CLI health using the candidate identity; the public command cannot
supply a substitute health callback. Failed health rolls back through the existing owner.

`install candidate activate-recover --transaction-id <id> --expected-journal-digest
<sha256> --approve-filesystem-write` replays the exact existing native journal/outcome.
It requires no fresh release adoption authority and never reruns health. Use a separate
CLI outside affected executable paths. Public activate/recover currently refuse other
platforms rather than claim macOS process inspection there. Earlier preparation,
discard and lower-level platform behavior remains unchanged.

Both successful commands use additive Rust-generated `candidate-activation-completed-v1`
JSON with exact transaction/journal identity and committed/rolled-back outcome. A returned
outcome describes local transaction completion, not a live Host reload or named-candidate
acceptance. Successful real packaged activation and process-kill qualification remain
separate evidence requirements.

The nonpublishing `native_candidate_acceptance` example accepts optional
`--predecessor-manifest <absolute-Cargo.toml>`. It builds that source with the same
in-memory test authority, reads its actual CLI version and requires it to precede the
current candidate. Both packages use existing artifact/archive/signature owners;
the temporary authority supports predecessor generation 1 and successor generation 2.
The journey installs the real predecessor CLI, stages the successor, then runs public
activation preview/prepare/activate, installed version/health/MCP and recovery replay.
Omitting the manifest records this journey as not run. A caller-derived predecessor
build proves runtime switching only; it does not attest to historical published source,
production signing, resource-pack migration or live Host reload. Private keys are never
persisted, and the receipt retains `publication_allowed=false`.

Installed CLI health validates a plan against the explicitly verified candidate version,
including a restored predecessor. The same plan validator still requires the running
process version for every managed write. Schema, TTL, digest, operation and approval
checks are unchanged; observing an older healthy CLI does not authorize an old write plan.

The macOS two-version runner also exercises an independent interrupted Home. It creates
a new process group for the public activation command, observes the installed CLI inode
change while the Home marker exists and no durable outcome exists, then sends SIGKILL
to that test-owned group. It requires an actual signal exit and verifies the new binary
was present with the old accepted release still pending. Public recovery must restore
the exact old binary/version/known-good identity, remove the Home marker and pass real
installed health plus MCP. Already-reaped children are never signaled; a missed window
fails the run. This is process interruption evidence, not arbitrary kill-point coverage,
power-loss durability or deletion interrupted inside every application tree.

## Quality Check

- Run the closest crate or integration test first.
- For contract changes, verify tool registry, dispatch, schemas, and docs agree.
- Run affected tests once locally; reuse unchanged results for integration.
  Remote CI is optional for local integration; named candidates own full
  cross-platform qualification.
- Confirm public CLI examples exist in the parser and `--help` output.
