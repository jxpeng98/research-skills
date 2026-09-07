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

## Quality Check

- Run the closest crate or integration test first.
- For contract changes, verify tool registry, dispatch, schemas, and docs agree.
- Run affected tests once locally; reuse unchanged results for integration.
  Remote CI is optional for local integration; named candidates own full
  cross-platform qualification.
- Confirm public CLI examples exist in the parser and `--help` output.
