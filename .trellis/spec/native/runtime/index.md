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

Public writes use preview, digest-bound approval, revalidation, and fail-closed
errors. `qiongli_project_capture_apply` is a real Full MCP project write and
must never be described as read-only. ToolHost remains read-only in-process.

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
reconciliation v2 wire semantics remain unchanged. Actual activation and native recovery command wiring remain separate pending work.

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
overwriting historical records. The caller still owns fresh candidate/approval and
process verification; public activation/recovery commands and competing-write
exclusion remain pending.

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
CLI health. Non-Unix managed writes retain their existing behavior. This is partial
entry-point coverage: legacy interrupted-update recovery still needs a final
cross-config guard pass before
public native activation is enabled. This lock coordinates participating processes;
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
existing independent completion path. This is cooperative process exclusion, not a
durable global marker for an interrupted legacy Desktop replacement; that remaining
legacy recovery interaction and update-state writers must be resolved before public
native activation is enabled.

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
contract as evidence rather than recursively deleting the transaction root. Durable
Home marker/recovery wiring for legacy replacement remains separate pending work.

## Quality Check

- Run the closest crate or integration test first.
- For contract changes, verify tool registry, dispatch, schemas, and docs agree.
- Run affected tests once locally; reuse unchanged results for integration.
  Remote CI is optional for local integration; named candidates own full
  cross-platform qualification.
- Confirm public CLI examples exist in the parser and `--help` output.
