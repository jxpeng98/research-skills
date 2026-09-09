# ADR 0222: User-approved local Plugin sources

- Status: Accepted
- Date: 2026-09-09
- Task: CLI-403 / CLI-405
- Authority: maintainer requested the next CLI-to-Plugin onboarding increment.
- Extends ADR 0219's ordinary CLI distribution with a distinct local source export.
  ADR 0213's signed managed installation and official Host authority stay intact.

A CLI installed through npm, PyPI, Cargo or a local build may export its own
binary and embedded content as an explicitly approved local Plugin source.
This operation does not verify publisher provenance or create a launch grant.
The user selects an absolute destination ending in `qiongli-next`, under an
existing secure parent. Host configuration/cache roots and `.qiongli` managed
paths are excluded. Existing files and signed bundles are never adopted.

Reuse the Codex/Claude bundle projection and transaction owners. The additive
`user-local-host-full-mcp` receipt kind has an empty signed-grant digest; signed
bundle verification, replacement and removal reject that kind. Local source
operations reject signed bundles. The local receipt binds every generated file,
the bundled binary, embedded pack and workflow variant. Dedicated marketplace
metadata names `qiongli-cli-local`; it does not overwrite the existing `personal`
or `qiongli-local` marketplaces.

`app plan plugin-source-install|plugin-source-update|plugin-source-remove` and
`app apply` reuse the existing ten-minute plan, exact digest, filesystem approval,
Home/config exclusion guard, target lock, staging, no-replace moves and
receipt verification. The plan binds the destination, target, current executable
hash, workflow variant and prior receipt. Update/removal additionally compare the
approved receipt inside the bundle transaction owner. Drift, links, unknown
files, stale plans and authority-kind changes refuse without deleting user data.

`app plugin-source-status` reports source integrity/currentness and explicitly
leaves Host state unverified. Export does not edit client configuration, invoke a
client or reload a session. Install/update reports `source-ready-host-action-required`;
removal reports `source-removed-host-state-unchanged`. Remove a registered Plugin
through its Host before deleting its source. In-use detection and automatic
Host registration/refresh/removal are the next increment, not implied here.

The initial complete increment is source installation, inspection, update and
removal for both targets, with executable exports and negative checks. Real Host
registration and tools-visible/read/handoff/approved-write evidence remain
separate. Canonical academic content, signed release authority and public Alpha.7
remain unchanged. Reverting the source commands does not remove existing exports;
older signed-only owners safely refuse the new receipt kind.

## Public schema compatibility

The new Plugin-source operation uses managed plan schema version 2. Earlier
operations keep emitting version 1; the current consumer dual-reads v1 legacy
operations and v2 source operations with the same digest/expiry checks. A source
operation relabeled as v1, or a legacy operation relabeled as v2, refuses. Older
CLIs fail closed on v2 and are not allowed to apply it. This is a
`migratable-breaking` managed-plan extension: keep old operation/approval meanings,
retain v1 support throughout 2.x, and regenerate a source preview with the new CLI
rather than rewriting an existing plan or its digest. No old support is removed.

Rust/Schemars generate the managed plan/result and source-status schemas and
positive fixtures through `plugin_source_contract`; checked-in bytes are outputs.
The shared result shape is unchanged (schema adoption only); source status is a
new additive contract. Path safety, filesystem identity, TTL and operation/version
relations remain consumer checks in addition to schema validation.
