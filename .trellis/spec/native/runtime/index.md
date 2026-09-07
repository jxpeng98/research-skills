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

## Quality Check

- Run the closest crate or integration test first.
- For contract changes, verify tool registry, dispatch, schemas, and docs agree.
- Run affected tests once locally; reuse unchanged results for integration.
  Remote CI is optional for local integration; named candidates own full
  cross-platform qualification.
- Confirm public CLI examples exist in the parser and `--help` output.
