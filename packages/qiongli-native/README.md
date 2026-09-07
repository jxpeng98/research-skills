# Qiongli Native Workspace

This workspace is the canonical Rust-native product source for Qiongli 2.x.
It contains the product application, `apps/qiongli`, plus the accepted content,
config, runtime, execution, project, platform-trust, and isolated
Windows-security service boundaries. The content crate defines the versioned
resource-pack manifest,
frozen profile projections,
and bounded canonical source collector. It compiles those collected bytes into
an unsigned deterministic `.qlpack` core, verifies and loads that core entirely
in memory, and can atomically materialize a verified profile through a trusted
target capability.

For the supported Svelte/Tauri development loop and local desktop packaging
commands, see
[Local Desktop Development and Packaging](../../docs/development/local-desktop-build.md).

## Dependency direction

The product dependency graph is one-way:

```text
apps/qiongli -> service crates -> contracts/platform primitives
```

Libraries must never depend on `apps/qiongli`, and the application remains a
thin mode dispatcher. Future service crates are added only with their first
real contract and tests. Production crates must not require Python, Node.js,
Cargo, or another language runtime to start.

`qiongli-content` freezes three projections: `skill-only`,
`marketplace-lite` (alias `lite`), and `full`. Its synthetic golden fixture
tests the format contract without binding the crate to the uncommitted working
copy of canonical content. FND-202B collects only the 14 allowlisted roots under
`content/`, normalizes and sorts portable paths, and rejects links, path
collisions, unsupported file types, and bounded count/size violations.
FND-202C writes a 20-byte versioned header, an RFC 8785 canonical manifest, and
the sorted uncompressed payload. SHA-256 entry digests feed a domain-separated
content root; SHA-256 over the entire unsigned core produces `pack_sha256`.
Input enumeration order, filesystem metadata, build paths, and wall-clock time
do not enter the result. FND-202D requires an expected whole-core SHA-256,
enforces bounded pack/manifest/entry/payload/path limits, rejects noncanonical
manifests, incompatible format/profile declarations, paths outside the
canonical roots, path/resource-kind mismatches, portable path collisions,
trailing payload, content-root drift, and entry-digest drift, then exposes
borrowed immutable bytes through an explicit profile projection. It accepts no
output path and performs no filesystem writes.

FND-202E keeps the write boundary separate from loading. A caller can request a
unique target inside an atomically created private temporary container (`0700`
on Unix) or can explicitly approve an absolute normalized target at a trusted
CLI, UI, or installer boundary. Explicit Unix targets reject group- or
world-writable ancestors; the private temporary factory permits only sticky
system temporary ancestors before its owner-only container. Model-generated
and MCP tool arguments must not cross that approval boundary. Materialization
refuses linked/reparse-point ancestors, unmanaged existing contents, receipt or
file drift, hard-linked managed files on Unix, and concurrent target ownership.
The create-new lock is
pinned to its file identity so an earlier owner does not unlink a replacement.
The selected profile and canonical `.qiongli-materialization.json` receipt are
written into a unique sibling staging directory that remains `0700` while
files are atomically created as `0600` on Unix. Files are closed before final
modes are applied; directories become `0755` only after writing completes. The
tree is then verified with bounded canonical-source/profile paths and logical
`0644`/`0755` modes before the prior managed tree is renamed to a backup and the
staging tree is promoted.
A promotion failure restores the backup before returning; handled pre-commit
failures remove their staging and lock artifacts. A failure after promotion is
reported explicitly as committed-with-cleanup-failure, including any remaining
backup cleanup path, instead of being indistinguishable from a pre-commit
failure.

FND-202F turns that content pipeline into a self-contained product resource.
The committed `qiongli-core.lock.json` freezes the accepted 1.19 metadata, 427
entries, content-root SHA-256, and whole-pack SHA-256. The `qiongli` Cargo build
script collects canonical sources, deterministically rebuilds the pack, and
fails closed unless both identities match the canonical lock. It writes only
the verified `.qlpack` and expected digest under Cargo `OUT_DIR`; normal builds
never rewrite tracked sources. `content/**` is fixed to LF through repository
attributes so checkout line-ending policy cannot create platform-specific pack
identities.

The application embeds both outputs at compile time. `EmbeddedContent` verifies
the static bytes against the expected digest and exposes profile inspection,
profile-scoped resource reads, and materialization through the existing target
capability. The canonical executable performs this integrity check at startup.
An integration test copies the executable outside the checkout and starts it
with an empty `PATH`, demonstrating that runtime content access does not read
the source tree or launch Python, Node.js, Cargo, or another external runtime.
Building Qiongli from source still requires the pinned Rust toolchain and the
canonical source tree; the distributed executable does not.

When a future content baseline is explicitly accepted, regenerate and review
the lock with native tooling before rebuilding the application:

```bash
cd packages/qiongli-native
cargo run -p qiongli-content --example update_qiongli_core_lock --locked
```

The expected digest establishes integrity only when it comes from a trusted
embedding or authenticated descriptor. Publisher signatures, trusted-key and
revocation policy, running-product compatibility enforcement, public UI/MCP
wiring, host installation, and adversarial same-user handle-relative
filesystem hardening remain separate successor work.

The version 1 header is `QLPACK\0\0`, followed by a little-endian `u32` format
version and a little-endian `u64` manifest length. Payload offsets start after
the manifest. The content-root preimage is
`qiongli:resource-pack:content-root:v1`, one NUL byte, the little-endian `u64`
length of the canonical entry-array JSON, and those canonical bytes. The
manifest schema keeps numeric fields within the JCS safe-integer range.

The existing `packages/qiongli-lite-mcp` crate remains a migration oracle and
compatibility package. Native functionality will be extracted into shared
workspace crates rather than copied into a second implementation.

## R2A shared Lite boundary

`qiongli-runtime` owns the first extracted Lite boundaries: a strict parser for
the 12 public Contract v2 Lite definitions (11 canonical typed identities) and
bounded newline/Content-Length stdio framing. The config-wizard compatibility
name resolves to the same typed handler identity as the canonical name. Runtime
errors expose only stable reason codes, and framing limits messages to 8 MiB
and headers to 64 KiB.

The canonical app enables the embedded-content adapter and proves that the
registry loads through the already verified `marketplace-lite` projection. The
old Lite package uses the same parser, identity resolver, and framing code
through thin compatibility modules. This checkpoint does not add an MCP mode
to the native executable and does not migrate provider, evidence, Zotero, or
orchestration-preview behavior.

## R2B shared provider kernel

`qiongli-runtime` now owns the five-provider access/status model, bounded HTTP
policy, response normalization, concurrent search, diagnostics, deduplication,
cooperative cancellation, and deterministic search planning. Canonical search
requests cap queries at 4,096 bytes, per-provider results at 200, and combined
results at 1,000. Provider HTTP disables redirects and implicit proxy discovery,
uses 3-second connection and 15-second request timeouts, and reads at most
4 MiB per response.

The optional native-config adapter resolves secret references only through
`SecretStore`; it does not read legacy provider files or environment aliases.
The old Lite package retains those compatibility inputs at its edge, converts
resolved values into the shared in-memory access model, and re-exports the
shared provider/search implementations. Access values are zeroizing and
non-serializable; public status and failures remain redacted.

R2B still does not add `qiongli mcp serve`, provider-management CLI/UI, a
production secure-store backend, evidence/Zotero extraction, or an installable
native release. Canonical MCP availability remains closed until binary-level
initialize, tools/list, and tools/call tests pass.

## R2C shared evidence and Zotero services

`qiongli-runtime` now owns bounded literature evidence snapshots and the
accepted Zotero surface. Evidence input is capped at 2 MiB, 1,000 results, 32
container levels, and 100,000 JSON values. Credential-bearing keys are removed
recursively before the deterministic snapshot is returned; the compatibility
`cwd` is validated but never copied into output.

Zotero export validates at most 1,000 normalized literature records and returns
only selected CSL-JSON, RIS, BibTeX, and import-report contents in memory. The
combined result is capped at 2 MiB. RIS folds control and line-separator input,
while BibTeX escapes syntax-bearing characters so reference text cannot inject
new records or fields.

The shared probe accepts only bounded HTTP(S) loopback roots, normalizes them
to fixed `/connector/ping` and `/qiongli/ping` calls, disables redirects and
implicit proxy discovery, applies a five-second production timeout, and reads
at most 32 KiB from the companion response. It returns status and bounded
version metadata only. The old Lite package retains the historical
`QIONGLI_ZOTERO_*` environment mapping at its edge and otherwise re-exports
the shared implementation.

R2C does not write import files, mutate or search a Zotero library, install the
companion, add native Zotero settings/UI, or expose MCP mode in the canonical
binary. Returned import-file contents prove generation only, not a completed
Zotero import.

## R2D shared orchestration previews and typed dispatch

Every canonical Lite tool identity now projects exhaustively into a typed
config, literature, Zotero, or orchestration handler target. The two
orchestration targets dispatch only bounded route and task-plan previews. They
return the accepted Marketplace Lite safety flags with agent execution, shell
execution, and project writes disabled.

Route requests are capped at 4,096 bytes and accept only the six Contract v2
platform values. Task IDs and paper types are capped at 256 bytes, topics at
4,096 bytes, and current trimming behavior is retained. Unknown, missing,
mistyped, blank, unsupported, and oversized input returns static errors without
echoing private values. The old Lite preview module is now a shared-runtime
re-export and its server dispatches through the same typed projection.

The returned `qiongli mcp serve --transport stdio` value remains compatibility
guidance for the frozen 1.x Full runtime. R2D does not add that command or any
MCP mode to the native executable, launch agents or processes, write project
state, or provide native provider-secret configuration.

## R2E canonical native Lite MCP vertical slice

The canonical product binary now composes the embedded Marketplace Lite
registry, bounded line and `Content-Length` framing, JSON-RPC/MCP envelopes,
typed dispatch, and the shared R2 domain services through one closed command:

```text
qiongli mcp serve --profile lite --transport stdio
```

`marketplace-lite` is the exact profile alias. Both options are required;
unknown, duplicate, Full-profile, and non-stdio values fail before serving.
Once selected, stdout contains MCP responses only. The server supports
initialize, initialized notifications, ping, tools/list, and tools/call for the
14 Lite public names.

Search-plan and literature-search inputs are now parsed in `qiongli-runtime`,
so the canonical server and old Lite compatibility adapter use the same alias,
provider, mode, and limit boundaries. Native global settings are converted to
the shared in-memory provider model. Secret references use only the explicit
`SecretStore` boundary; R2E has no production secure-store backend and never
falls back to environment credentials or the 1.x plaintext provider file.

Provider-status results use `<managed-native-config>` instead of a local path.
Valid provider-save and wizard calls return a fixed unavailable tool error
after strict validation, without writing config, opening a listener, launching
a browser, or echoing the supplied value. Zotero status, explicit local search,
and receipt-bound dry-run/apply upserts use the loopback-only Companion contract;
import-file fallback remains available. Route and task tools remain preview-only.

The copied-binary acceptance runs with an empty `PATH` and proves initialize,
all 14 listed names, bounded safe calls, secret/path redaction, and clean EOF
without Python or Node. This establishes the native development vertical
slice only. Signed launch grants, plugin/Desktop activation, target packaging,
provider-secret mutation, Full MCP, agents, UI, installer, and release
readiness remain R3 or later work.

## R3A install-plan and Lite launch-grant contracts

`qiongli-platform` now owns the first installation trust boundary. It validates
the exact product/version/channel/profile/OS/architecture/installer identity,
then verifies a bounded Ed25519-signed Lite launch grant against public keys
already trusted by the product composition layer. The signature covers
domain-separated canonical JSON plus binary and embedded-pack SHA-256 values,
allowed `cli`/`lite-mcp` modes, local Codex/Claude Code scopes, validity times,
and an anti-replay generation. The verified capability token retains the mode
and scope checked by the verifier, so it cannot be reused to plan a different
integration.

The same crate defines a bounded, strict `InstallPlanV1` preview with closed
target and symbolic-root vocabularies, typed materialize/plugin-source/Lite-MCP
operations, observed state, postconditions, inverse operations, approvals, and
host-owned outstanding actions. Plans reject traversal, nonportable paths,
unknown roots, duplicate or unsorted identities, mismatched ownership, stale
targets, non-Lite profiles, altered semantic digests, and operations without a
matching inverse. Plan ID and display timestamps do not affect the canonical
semantic digest; every semantic field does.

The source-built executable reports this boundary through:

```text
qiongli install status
```

It truthfully returns `launch_grant`, `preview`, and `apply` as `unavailable`
and labels `codex-local` and `claude-code-local` as `contract-only`. Tests
generate ephemeral signing keys; the repository contains no signing private
key or caller-selectable trust root. This checkpoint does not discover or
write host paths, register a plugin/MCP entry, mutate a private cache, create
receipts, or claim installation, activation, packaging, or alpha.1 readiness.
Those remain R3 successor gates.

## R3B managed resource transaction vertical

`qiongli-platform` now contains the first executor behind the R3A trust
boundary. It accepts only an already verified signed plan, an exact local
approval token, one explicitly approved owner-only `QiongliManagedData` root,
and bytes from a verified embedded resource pack. The initial executable
subset is deliberately one Marketplace Lite resource directory with a missing
precondition; arbitrary paths, caller-provided bytes, client configuration,
multi-operation plans, managed overwrite, and host actions are rejected before
persistent mutation.

Fresh apply writes one root-scoped immutable journal before delegating the
atomic resource tree commit to `qiongli-content`, verifies the complete tree,
and commits a canonical owner-only platform receipt last. Read-only verify
binds the active platform receipt to the canonical materialization receipt.
Repair restores only an absent target with the same semantic plan and install
identity; a present but drifted target remains a conflict. Remove and rollback
identity-pin and reverify the exact managed target, quarantine it, commit a
distinct lifecycle receipt, and then clean the quarantine. Proven pre-commit
failures restore absence or the active target; uncertain materializer
ownership, state-commit results, or rollback retain data and the journal and
fail closed.

`qiongli install status` reports receipt contract version `1` and the
transaction engine as `grant-and-approval-gated`. The source binary still has
no production launch grant, approved root, or executable plan, so
`launch_grant`, `preview`, and `apply` remain `unavailable`. R3B does not
discover or register Codex/Claude, write MCP/client configuration, activate a
plugin, perform an in-place upgrade, install into Marketplace/Desktop/cloud,
or produce a package or release.

## R3C Codex personal marketplace adapter

`qiongli-platform` now contains the first `CodexLocal` host adapter. It derives
only the documented current-user personal marketplace and a fixed private
Qiongli source location, validates that source through its complete native
plugin-bundle receipt, and exposes a deterministic one-operation
`RegisterPluginSource` preview. The plan requires the exact
`filesystem-write`, `client-config-change`, and `host-trust` approval vector and
retains `install-or-enable-plugin` as an outstanding client action.

The registration executor preserves unrelated marketplace fields and entries,
uses private canonical receipts and a root-scoped journal, and supports apply,
verify, absent-entry repair, remove, and rollback. Conflicting unreceipted
`qiongli` entries, malformed or oversized documents, source or receipt drift,
linked/insecure paths, partial approval, and ambiguous rollback fail closed.
The embedded `marketplace-lite` projection contains the canonical Codex
metadata template; it is not itself treated as an installable plugin package.

The source executable adds the side-effect-free command:

```text
qiongli install codex status
```

It returns symbolic paths and typed source, marketplace, and registration
states without creating directories or exposing the user home. General install
status labels the Codex adapter engine ready, while production `launch_grant`,
`preview`, and `apply` remain `unavailable`. Registration does not write or
claim the Codex plugin cache, enablement state, Desktop installation, MCP
activation, public Marketplace publication, or cloud availability.

## R3D native Codex plugin package

`qiongli-platform` now composes a complete target-specific Codex plugin from
the verified embedded Marketplace Lite projection, a verified `PluginBundle`
launch grant, and the grant-matched native Qiongli executable. The generated
root contains canonical `skills/qiongli-workflow` content,
`.codex-plugin/plugin.json`, `.mcp.json`, `bin/qiongli[.exe]`, and a canonical
receipt covering every other file, mode, size, digest, artifact identity,
signed-grant payload, and resource-pack identity.

The root MCP declaration launches that plugin-local executable as
`qiongli mcp serve --profile marketplace-lite --transport stdio`. It never
names Python, Node, Rust, Cargo, a package manager, or a user-global Qiongli
command. Composition uses a private sibling stage, a target lock, no-replace
promotion, and committed verification. Existing targets, unmanaged data,
links, reparse points, hard links, permission drift, extra or missing files,
receipt drift, and signed binary or content mismatch fail closed.

The normal Rust test copies the product binary, builds the full 422-entry
embedded projection into the generated plugin layout, and launches the bundled
MCP with an empty `PATH`. A separate explicit acceptance test uses an isolated
home and `CODEX_HOME`, validates the generated root with Plugin Creator, asks
the real Codex CLI to install it from the isolated personal marketplace,
verifies Codex's cache and enablement record, and launches the cached MCP with
an empty `PATH`. It does not modify the developer's normal Codex state.

The source executable still has no production signing grant or public package
command, so general `launch_grant`, `preview`, and `apply` status remains
`unavailable`. R3D proves local package composition and client activation; it
does not publish a public Marketplace entry, make a local executable available
to cloud/web runtimes, ship an alpha, add package upgrade/rollback, or complete
Claude, UI, Full MCP, agent, or orchestrator execution work.

## R3E Claude Code local integration

`qiongli-platform` now composes the same verified native Marketplace Lite
content into a target-specific Claude Code plugin. The generated package
contains `.claude-plugin/plugin.json`, `.mcp.json`,
`skills/qiongli-workflow`, `bin/qiongli[.exe]`, and a canonical receipt binding
the signed grant, artifact, embedded pack, binary, modes, sizes, digests, and
complete package tree.

The preferred direct target is `<claude-config>/skills/qiongli`. Discovery
accepts only an exact verified package, reports symbolic state through
`qiongli install claude status`, and treats unreceipted or drifted content as a
conflict. Verified removal first moves the exact package into a
transaction-owned quarantine and never adopts or overwrites unmanaged data.

The alternative local marketplace remains below Qiongli-managed data. Its
receipt-backed adapter provides preview, apply, verify, absent-entry repair,
remove, and rollback while preserving unrelated state. Claude Code—not
Qiongli—adds that marketplace, installs or enables the plugin, and owns its
registry, settings, and versioned cache. The plugin MCP command uses
`${CLAUDE_PLUGIN_ROOT}/bin/qiongli[.exe]` and needs no Python, Node, Rust,
package manager, checkout, or global Qiongli command at runtime.

An explicit acceptance test isolates both `HOME` and `CLAUDE_CONFIG_DIR`.
Claude Code `2.1.206` strictly validates and discovers the skills-directory
package, adds and installs the local marketplace package, copies the verified
tree into its isolated cache, launches all 12 Lite MCP tools with an empty
`PATH`, and removes the isolated plugin and marketplace. The normal gate keeps
this external-client test ignored and passes all 199 normal native Rust tests,
all 69 focused Lite compatibility tests, strict host Clippy, and Windows MSVC
workspace check/Clippy.

Production grants, mutating install commands, Claude Desktop, cloud/web
execution, public marketplace publication, managed in-place upgrade, release
artifacts, UI, Full MCP, agent execution, and orchestrator execution remain
unavailable.

## R3F native desktop manager

The canonical executable now has one Rust-native desktop mode:

```text
qiongli ui
```

`qiongli-ui` owns only bounded presentation models, typed intents and events,
stock egui views, and the eframe application. The product composition root
builds a read-only snapshot from the verified embedded pack, redacted global
configuration, the 12-tool Lite MCP contract, and Codex and Claude Code local
discovery. The snapshot contains typed states, counts, symbolic locations, and
fixed remediation codes; it does not contain concrete paths, email addresses,
API keys, secret references, environment values, or raw service documents.

The window provides Overview, Skills, MCP, Providers, Integrations, and
Diagnostics views. Refresh and provider/integration previews cross the typed
service boundary. Provider preview accepts only a transient public contact
email and clears it after submission. The production adapter validates that
input but keeps config writes and every install confirmation unavailable.
Closing the window does not persist eframe state.

The UI pins eframe, egui, and egui_kittest 0.35.0, using native wgpu and
AccessKit without a webview, JavaScript frontend, glow renderer, or persistence
feature. Headless AccessKit tests cover all six views, keyboard activation,
labelled input, non-echoing feedback, confirmation/cancel, progress and
recovery states, narrow and normal widths, and 100%, 150%, and 200% scale.
These tests do not replace later packaged-window and target screen-reader
acceptance.

R3F remains a source-build alpha boundary. It does not install, remove, update,
or activate plugins; write provider configuration or secrets; launch or
supervise MCP from the window; package a desktop application; or claim Claude
Desktop, cloud/web, public marketplace, Full MCP, agent, orchestrator, updater,
signing, or release readiness. Building from source requires the pinned Rust
toolchain; a future packaged executable must not require that toolchain at
runtime.

## R3G current-target artifact staging

`qiongli-platform` can now assemble the canonical current-target executable
into a verified native artifact staging tree:

```text
qiongli-<version-channel-profile-os-arch-package>/
  .qiongli-native-artifact.json
  bin/qiongli[.exe]
```

The typed identity fixes product, version, channel, Lite profile, host OS and
architecture, and `portable-archive` package kind. The canonical RFC 8785 JSON
manifest records `assembled-unpublished` status, the binary's logical mode,
size, and SHA-256 digest, the caller-supplied verified resource-pack identity,
and a domain-separated artifact content root.

Composition validates a bounded regular source executable and an explicitly
approved canonical target, uses an owner-private sibling stage and target lock,
writes with create-new semantics and fixed logical modes, syncs the result,
promotes without replacement, and verifies the committed tree. Verification
rejects links/reparse points, hard links, ownership or mode drift, missing or
extra entries, non-canonical metadata, identity drift, resource-pack drift,
and binary/content-root tampering through fixed path-redacted reason codes.

Current-target application tests run only the committed artifact-local binary
from outside the checkout with an empty `PATH`. They verify `--version`, content
inspection, MCP initialization, the exact 12-tool Lite contract, and a bounded
read-only tool call. The Windows fixture uses the same owner-only directory
primitive required by the production validator.

R3G produces an unpacked staging tree, not the final portable archive. It does
not add compression, signing, notarization, checksums, SBOM, provenance,
installation, updater, publication, packaged-window startup, cross-target
packaging, or clean-machine release acceptance.

## R3H deterministic portable archive

`qiongli-platform` can wrap one verified R3G current-target staging tree in a
canonical portable archive named `<artifact-id>.zip`. The accepted container is
a strict store-only ZIP with exactly these entries in fixed order:

```text
<artifact-id>/
<artifact-id>/.qiongli-native-artifact.json
<artifact-id>/bin/
<artifact-id>/bin/qiongli[.exe]
```

The writer fixes UTF-8 names, the 1980-01-01 DOS timestamp, logical modes,
local and central records, CRC-32 values, offsets, and an empty comment. The
bounded parser rejects compression, encryption, ZIP64, data descriptors,
extra fields, comments, multiple disks, unknown or reordered entries, trailing
bytes, and any mismatch with the canonical R3G manifest or caller-supplied
verified resource pack.

Composition uses an explicitly approved canonical target, an owner-private
create-new sibling file, target locking, persistence sync, no-replace
promotion, and committed verification. Extraction parses and verifies the
complete envelope before mutation, never joins an archive-controlled path, and
commits only the fixed manifest and executable payload through the existing
R3G private staging path.

Current-target application tests compose two byte-identical archives, extract
one into an isolated root, and run only its executable outside the checkout
with an empty runtime `PATH`. They verify `--version`, embedded content, MCP
initialization, the exact 12-tool Lite contract, and one bounded read-only tool
call. Structural, content, source-drift, link, lock, and destination-conflict
failures preserve existing caller data and return fixed path-redacted reason
codes.

R3H is an unsigned, unpublished transport-container gate. It does not provide
signing, notarization, checksum sidecars, SBOM, provenance, launch grants,
installation, updater behavior, publication, packaged-window startup,
cross-target packaging, or clean-machine release acceptance.

## R3I verified native-payload installation

`qiongli-platform` can now bind one verified R3H current-target archive to an
R3A signed launch grant, deterministic `InstallPlanV1`, explicit
`filesystem-write` approval, and caller-approved `QiongliManagedData` root.
The additive `InstallNativePayload` action fixes the canonical artifact leaf
and binds the archive, manifest, resource-pack, artifact-content-root, binary,
and signed-grant digests before persistent mutation.

The managed root uses an archive-derived install identity and contains only
portable, caller-independent records:

```text
<managed-root>/
  <artifact-id>/
    .qiongli-native-artifact.json
    bin/qiongli[.exe]
  .qiongli-native-payload-<archive-sha256>.json
```

Apply and absent-target repair run through the accepted R3H parser and R3G
no-replace commit path, re-verify the installed tree, and atomically persist a
strict bounded owner-private receipt. Identical apply, read-only verify,
present-healthy repair, remove, rollback, and terminal lifecycle replay have
explicit dispositions. Remove and rollback first move a verified tree into an
identity-checked private quarantine; failures before the durable state point
restore it, while ambiguous outcomes retain a journal and return
`install-recovery-required`.

Current-target acceptance creates an explicit test-signed launch grant,
installs the verified archive into an isolated managed root, and runs only the
installed executable outside the checkout and archive trees with an empty
runtime `PATH`. It verifies `--version`, embedded content, MCP initialization,
the exact 12-tool Lite contract, and one bounded read-only tool call. Fault,
tamper, linked-state, drift, and foreign-destination tests preserve existing
caller data and use fixed path-redacted errors.

R3I is a shared installation service, not a production installer or release.
The source build cannot manufacture a production launch grant. Production key
provisioning, production-signed release metadata, automatic managed-root
discovery, client activation, public Marketplace installation, updater
behavior, notarization, SBOM, provenance, packaged-window startup, cross-target
output, and clean-machine release acceptance remain later gates.

## R3J signed native release envelope

`qiongli-platform` now authorizes one exact R3H current-target archive through
a strict bounded canonical `NativeReleaseEnvelopeV1`. The envelope binds the
complete artifact identity, release channel and generation, validity interval,
archive filename, size and digest, R3G manifest, embedded resource pack,
artifact content root, executable digest, and complete R3A signed launch grant.

Release-envelope Ed25519 keys and launch-grant keys are separate Rust trust
roles. A trusted release public key has a bounded key ID plus an inclusive
minimum and optional exclusive maximum release generation. At most sixteen
unique release keys are accepted, and there is no unsigned, caller-supplied, or
cross-role fallback. The repository exposes only deterministic envelope and
domain-separated signing-preimage construction; it contains no production
private-key API or material.

Successful verification returns a private token retaining the caller-approved
archive target, newly verified archive, and independently verified launch
grant. Native-payload preview, apply, and repair require that token. The plan
action, semantic digest, and canonical receipt bind its signed-payload digest,
and the executor re-verifies and compares the retained archive immediately
before extraction. Receipt-backed verify, remove, rollback, and recovery remain
available without the release source.

Acceptance uses distinct deterministic test-only release and launch keys. It
rejects noncanonical and oversized input, unknown fields, removed or
out-of-window keys, wrong key roles, stale generations, invalid time or
channel, signature and metadata tampering, payload mismatch, invalid attached
grants, archive drift, and alternate valid release tokens before managed
mutation. The installed current-target executable still runs CLI and the exact
12-tool Lite MCP contract outside the checkout with an empty runtime `PATH`.

R3J is a release-verification boundary, not the published Lite Alpha.1. The
accepted implementation remains test-signed and does not provide production
release-key provisioning, an executable CLI/UI install intent, automatic root
discovery, client activation, public publication, updater behavior, OS signing
or notarization, checksum sidecars, SBOM, provenance, cross-target artifacts,
or clean-machine release acceptance.

## R3K embedded release authority and native install CLI

The canonical product can now embed one strict public release-authority policy
at build time. `NativeReleaseAuthorityV1` fixes the channel, minimum release and
launch-grant generations, and bounded sorted Ed25519 public-key sets for the two
independent R3J trust roles. Release keys retain explicit generation windows;
key IDs and public-key bytes cannot overlap across roles. The policy channel
must agree with the compiled product version.

Release builds provide the byte-canonical public policy through
`QIONGLI_NATIVE_RELEASE_AUTHORITY_FILE`. The build script validates and embeds
the policy or fails the build. The variable is a build input only: the installed
binary has no runtime environment, command-line, file, network, or development
key override. Ordinary source builds embed an empty sentinel and truthfully
report release authority, signed-release preview, and apply as unavailable.

An authority-injected current-target product exposes:

```text
qiongli install native preview \
  --release <release.json> \
  --archive <archive> \
  --managed-root <existing-private-absolute-directory> \
  --target <codex|claude>

qiongli install native apply <same source options> \
  --expected-plan-digest <preview-sha256> \
  --approve-filesystem-write

qiongli install native verify \
  --managed-root <existing-private-absolute-directory> \
  --install-id <native-payload-id>

qiongli install native remove <same receipt options> \
  --approve-filesystem-write
```

Preview and apply derive the compiled current Lite portable target and repeat
the complete R3J release/archive/grant verification plus R3I plan verification.
Preview emits a path-redacted `mutation: none` summary. Apply reconstructs that
plan and requires both its exact semantic digest and explicit filesystem-write
approval. Verify and remove use the canonical owned receipt, so they remain
available without the release source or embedded authority; remove still
requires explicit approval and preserves R3I drift, quarantine, and recovery
rules.

R3K requires an already existing owner-private managed root. It does not choose
or provision production key values, sign or download a release, create or
discover the root, expose repair, activate Codex or Claude, write desktop state,
start a packaged window, publish an artifact or Marketplace entry, provide an
updater, sign/notarize an OS package, produce checksum/SBOM/provenance outputs,
or claim clean-machine Lite Alpha.1 readiness.

## R3L client activation coordination and desktop intent

The accepted Codex and Claude Code registration adapters now share one closed
`ClientActivationCoordinator`. Discovery returns a path-redacted capability
handle for exactly one target. Preview accepts only an already verified Lite
PluginBundle launch grant, re-verifies its trusted key, generation, target
scope, mode, artifact and plan, and requires exactly `filesystem-write`,
`client-config-change`, and `host-trust` approval. A private process-local
binding prevents a preview from one discovered handle being applied through a
different handle, even when both name the same client family.

The coordinator preserves target-specific apply/replay, verify, repair,
remove/replay, rollback/replay, recovery, and unrelated-entry behavior. It
verifies immediately after apply or repair and attempts the accepted adapter
rollback if a fresh mutation unexpectedly fails verification. Public discovery,
commit, verification, lifecycle, Debug, and error output remain path-redacted;
client-owned plugin installation, cache, enablement, reload, and trust prompts
remain explicit host actions.

The desktop service may receive at most one prepared trusted activation session
per local target from a release/installer boundary. A confirmable preview shows
the exact lowercase plan digest and the three approval labels, while the
verified plan remains application-owned behind one OS-random 128-bit operation
token. Confirm applies only that pending plan and refreshes the validated
snapshot; cancel, wrong token, stale token, malformed preview, and missing
session paths fail closed. Ordinary source builds receive no session, report
`apply: false`, and retain a truthful blocked integration preview.

The canonical binary also exposes the non-mutating preflight:

```text
qiongli ui --startup-check
```

It validates the embedded content, desktop service, redacted snapshot, UI app
state, and linked window entrypoint, emits versioned path-free JSON, opens no
window, and starts no subprocess. Acceptance runs the command from a copied
current-target artifact outside the checkout with an empty `PATH`, so it cannot
silently depend on Python, Node.js, Cargo, or another language runtime.

R3L itself does not assemble the R3K portable payload and target-specific
PluginBundle into a release candidate. The R3M successor below composes that
local candidate journey; production signing, publication, client-owned
enablement, cloud surfaces, updater behavior, and the remaining acceptance
gates stay separate.

## R3M signed candidate product journey

Release builds additionally provide the exact lowercase 40- or 64-character
source object ID through `QIONGLI_NATIVE_SOURCE_COMMIT`. The build validates and
embeds that value beside the public release authority. Neither value is read
from the runtime environment, and neither contains private signing material.
Source builds without both inputs fail candidate preview/apply closed.

The authenticated candidate is exactly one same-directory three-file set with
derived filenames: the portable archive, canonical signed candidate JSON, and
signed release notes. Product commands accept those files and a client target,
but no managed root or plugin-source path:

```text
qiongli install candidate preview \
  --candidate <artifact-id>.candidate.json \
  --archive <artifact-id>.zip \
  --release-notes <artifact-id>.release-notes.md \
  --target <codex|claude>

qiongli install candidate apply <same file and target options> \
  --expected-approval-digest <preview-sha256> \
  --approve-filesystem-write \
  --approve-client-config-change \
  --approve-host-trust

qiongli install candidate verify \
  --target <codex|claude> \
  --install-id <native-payload-id>

qiongli install candidate remove <same target and install ID> \
  --approve-filesystem-write \
  --approve-client-config-change
```

Preview performs no write and emits only versioned, symbolic-path output. Its
approval digest binds both the signed candidate digest and selected target.
Apply re-verifies the complete candidate and derives the fixed owner-private
payload root `<user-home>/.qiongli/native/payloads` plus the fixed Codex or
Claude Code source path. It then applies and immediately verifies payload,
source, and registration receipts as one closed identity chain. A fresh later
failure compensates only fresh earlier steps in reverse order.

For release engineering, payload-only staging uses the same verified candidate
and fixed payload owner without changing Host sources, registration or the
installed command:

```text
qiongli install candidate stage-preview <same file and target options>
qiongli install candidate stage <same file and target options> \
  --expected-approval-digest <stage-preview-sha256> \
  --approve-filesystem-write
```

Stage requires exactly filesystem-write approval; Host approval flags are rejected.
Its digest binds the candidate and target under a separate staging domain, so
neither a full-install digest nor a stage digest authorizes the other operation.
Execution re-verifies the candidate before writing. Replay reports `already-staged`.
Stage JSON version 1 is generated from the Rust type in `candidate_cli.rs`; the
schema is `apps/qiongli/schemas/candidate-stage-v1.schema.json` and its three
Rust-produced fixtures are under `apps/qiongli/tests/fixtures/candidate-stage-v1.*`.
Regenerate with the `candidate_stage_contract` Cargo example, which emits a
`schema` object and a `fixtures` array. Existing command JSON remains unchanged.
Staging still requires the running build's exact source, version and content;
it does not activate a different version or provide an end-user updater.

Verify and remove require no candidate, authority, source-commit input, or
unexpired release. They reopen only the fixed current-user paths and require
the payload, PluginBundle, and registration receipts to agree on target,
artifact, binary, pack, grant, and source identities. Remove verifies first and
then removes registration, source, and payload in reverse order. A partial
remove fails with recovery-required evidence rather than deleting unverified
state.

The native manager can open the same verified candidate session without a
second installer implementation:

```text
qiongli ui \
  --candidate <artifact-id>.candidate.json \
  --archive <artifact-id>.zip \
  --release-notes <artifact-id>.release-notes.md \
  --target <codex|claude>
```

Its operation token owns the verified in-memory candidate, shows the same
target-bound approval digest and three approvals, and calls the same local
apply pipeline on confirmation. Qiongli still does not drive client UI or
modify client-owned cache, enablement, reload, or trust state; those remain
explicit host actions. This implementation does not itself create a tag,
publish Alpha.1, provision production private keys, or claim Claude Desktop,
Codex/ChatGPT Marketplace bypass, cloud execution, updater, notarization,
SBOM, provenance, or cross-target readiness.

### Non-publishing candidate acceptance

Maintainers and Native CI can assemble and exercise an ephemeral test-signed
candidate with the repository example:

```text
cargo run --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --example native_candidate_acceptance \
  --locked \
  -- \
  --output <absolute-new-directory-outside-checkout> \
  --source-commit <exact-40-or-64-character-lowercase-object-id>
```

The output parent must already exist and pass the same trusted-path rules as a
release target. The harness creates distinct ephemeral release and launch keys
in memory, persists only the canonical public authority, builds one
authority/source-bound Release binary, normalizes Cargo output into a fresh
owner-private single-link artifact source, and verifies that the candidate
directory contains exactly the archive, signed candidate JSON, and signed
release notes. The normalized source is removed immediately after composition.
The release notes are rendered from a reviewed template for the exact current
OS, architecture, artifact identity, and three candidate filenames before the
candidate is signed. Acceptance evidence records each candidate file's exact
name, byte length, and SHA-256 so review does not depend on an unbound generic
notes document or an inferred target.

Acceptance extracts and runs only that binary from outside the checkout with
an empty `PATH` and isolated homes. It covers version, embedded skills,
materialization, UI startup preflight, the public Lite MCP tool list, Codex and
Claude Code preview/apply/verify/remove, wrong and partial approvals, fresh-step
compensation, and byte preservation for user projects, unrelated global v2
state, and unmanaged Host state. Explicit Native CI acceptance runs the same
receipt on Linux, macOS, and Windows. The generated
`acceptance-evidence.json` is explicitly non-publishing. Real-client UI,
displayed-window, and production-signing gates remain `not-run` with reasons
until their external environments and maintainer authority are provided.

## R3N Alpha.1 desktop application rebaseline

Interactive review of the R3F/R3M window found five Alpha.1 blockers: the
window cannot edit supported global settings, select a Skills destination, run
a Lite MCP self-test, usefully discover Codex and Claude Code from an ordinary
source session, or launch as a double-clickable cross-platform application.
R3M remains accepted signed-candidate core evidence, but it no longer closes
the Alpha.1 product milestone by itself.

R3N adds those five behaviors through the existing typed services and packages
the same native product as a macOS application, Windows desktop application,
and Linux AppImage/desktop launcher. Full MCP, agents, ToolHost, orchestrator,
updater, cloud execution, and public Marketplace distribution remain later
milestones. The authoritative scope and execution order are recorded in:

- `docs/superpowers/specs/2026-07-15-qiongli-r3n-alpha1-desktop-app-closure-design.md`
- `docs/superpowers/plans/2026-07-15-qiongli-r3n-alpha1-desktop-app-closure.md`

R3N Batch 1 is implemented. Overview can preview and atomically save the
supported non-secret global settings with optimistic revision checks. Skills
can select a destination through the native operating-system folder dialog,
then preview, materialize, verify, and receipt-safely remove an embedded
profile. Filesystem and config mutations require typed, digest-bound
confirmation; selected paths and public-setting input are excluded from Debug
output. Credential-store backends remain outside this batch.

R3N Batch 2 implementation is also complete. MCP now runs a bounded,
cancellable, five-second offline self-test against the canonical embedded
registry and `LiteMcpServer` dispatcher. Results cover initialize, the exact
ordered tools registry, offline task-plan dispatch, redacted provider
readiness, and local client registration with fixed remediation codes. The
Integrations view separately reports client discovery, Qiongli management
state, and candidate-backed install authority, so ordinary source sessions can
discover Codex and Claude Code without gaining write authority. Release-grade
real-client receipts remain an external acceptance requirement when usable
client binaries and validators are available.

Batch 5 real-client checks deliberately require absolute paths to the actual
client executables rather than version-manager shims. The tests replace the
client home/config roots with isolated directories, so a shim which resolves
tools from the ordinary home can produce false failures. Codex also uses the
Plugin Creator validator; its Python interpreter must provide PyYAML. That
interpreter is an external development validator dependency and is never
copied into, launched by, or required by the Qiongli product bundle.

```text
QIONGLI_CODEX_BIN=<absolute-real-codex-binary> \
QIONGLI_PLUGIN_VALIDATOR=<absolute-validate_plugin.py> \
QIONGLI_PLUGIN_VALIDATOR_PYTHON=<absolute-python-with-pyyaml> \
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli --test codex_plugin_bundle --locked \
  real_codex_clean_client_installs_enables_caches_and_launches_bundle -- \
  --ignored --exact --nocapture

QIONGLI_CLAUDE_BIN=<absolute-real-claude-binary> \
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli --test claude_plugin_bundle --locked \
  real_claude_clean_client_discovers_and_installs_both_local_forms -- \
  --ignored --exact --nocapture
```

Both tests isolate client-owned state, verify the receipt-backed cached bundle,
run that bundle's Lite MCP with an empty `PATH`, and exercise client and Qiongli
cleanup. They remain external acceptance tests rather than normal CI because
the supported client installations and Plugin Creator validator are not Rust
workspace dependencies.

The Alpha.1 candidate acceptance executable can also bind both real-client
journeys to the same ephemeral test-signed candidate instead of separately
composed test bundles. All tool paths must resolve from absolute paths; the
Codex binary requires the Plugin Creator validator and an explicit Python
interpreter that can import PyYAML as external development-only inputs. The
output root must be a fresh private directory outside the checkout and outside
recognized shared temporary roots:

```text
cargo run --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli --example native_candidate_acceptance --locked -- \
  --output <fresh-private-output> \
  --source-commit <exact-source-object-id> \
  --codex-bin <absolute-real-codex-binary> \
  --plugin-validator <absolute-validate_plugin.py> \
  --plugin-validator-python <absolute-python-with-pyyaml> \
  --claude-bin <absolute-real-claude-binary>
```

The executable applies each candidate into a separate fresh home, lets the
real client install and cache the registered source, verifies the cached
receipt, launches the cached Lite MCP with an empty `PATH`, removes the client
installation, verifies absence, and then performs Qiongli candidate removal.
It records client versions and digest-only evidence without paths. Supplying no
external client arguments preserves the normal CI journey and records the
real-client gate as `not-run`; supplying only one client records `partial`.
Every form remains `publication_allowed: false` and uses memory-only ephemeral
test keys, so production signing and final-source regeneration remain separate
maintainer gates.

R3N Batch 3 implementation is complete. Running `qiongli` without arguments
now opens the same native UI composition as `qiongli ui`; explicit CLI and
machine-readable modes retain their existing parser and output contracts. A
new `qiongli-desktop` activation binary is deliberately thin: it locates only
the sibling canonical runtime and starts UI mode. Windows builds use the GUI
subsystem and `CREATE_NO_WINDOW`, so desktop activation does not leave a
console window while all product logic and embedded content remain in the
canonical executable. The window now carries the Qiongli product name,
version, MIT license, `io.github.jxpeng98.qiongli` application identifier,
fixed startup error metadata, and a product-specific icon. Platform `.app`,
Windows portable, and AppImage assembly is handled in Batch 4.

R3N Batch 4 package finalization is implemented for exact-head CI acceptance.
A single Rust composer produces deterministic, verified source artifacts for
all three target families: a macOS `Qiongli.app` in an update ZIP, a Windows
portable application ZIP, and a Linux AppDir ZIP. Linux CI converts the exact
verified AppDir into a Type 2 AppImage with a digest-pinned official
`appimagetool`, extracts the result, and verifies every file, mode, manifest,
and digest before writing a separate AppImage receipt. All artifacts remain
labelled `assembled-unpublished` and unsuitable for public distribution until
their target activation and release gates pass.

Each desktop manifest is independent of the R3M three-file candidate and binds
the desktop archive to the exact portable artifact identity and manifest,
product source commit, canonical executable, thin launcher, native update
helper, embedded resource pack, application metadata, license, and every
archive entry. Verification
rejects layout, mode, order, size, hash, or source drift before output is
accepted.

The CI workflow builds release-mode artifacts on macOS, Windows, and Linux in
a parallel matrix, runs the actual packaged launcher through its fixed startup
preflight with an empty runtime `PATH`, and retains the results for seven days
as explicitly non-publishing evidence. The package command may also be run by
maintainers after building all three binaries with the same exact source-commit
binding:

```text
QIONGLI_NATIVE_SOURCE_COMMIT=<exact-clean-head> cargo build \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli --release --bins --features custom-protocol --locked

QIONGLI_NATIVE_SOURCE_COMMIT=<exact-clean-head> cargo run \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli --example native_desktop_package --release --locked -- \
  --canonical <absolute-path-to-release-qiongli-or-qiongli.exe> \
  --launcher <absolute-path-to-release-qiongli-desktop-or-qiongli-desktop.exe> \
  --update-helper <absolute-path-to-release-qiongli-update-helper-or-qiongli-update-helper.exe> \
  --output <absolute-new-output-directory> \
  --source-commit <exact-clean-head>
```

The command requires a fresh absolute output directory outside the source
checkout under a trusted, non-shared parent and writes exactly the target archive,
`qiongli-desktop-package.manifest.json`, and
`qiongli-desktop-package.receipt.json`. On Linux, CI additionally emits the
AppImage and `qiongli-linux-appimage.receipt.json`. The source composer does
not sign, notarize, publish, install, remove, or create a DMG. The automated
startup preflight does not claim a human Finder/Explorer/file-manager launch or
accessibility pass. Those Batch 5 gates must use the exact committed candidate;
a dirty-tree smoke package is never release evidence.

R3Q packaged-product control adds a second, stricter macOS assembly boundary.
The canonical runtime must be signed before its two target-specific launch
grants are prepared. After an external launch-key holder signs the exact Codex
and Claude Code preimages, `native_product_control finalize` verifies both
signatures and emits `.qiongli-product-control.json` plus the expected updated
desktop manifest. Re-run `native_desktop_package` with
`--product-control <absolute-control-path>` using the same already-signed
canonical runtime. The emitted manifest must be byte-identical to the finalized
expected manifest.

The remaining App signing step must use:

```text
tooling/scripts/macos_native_sign_notarize.sh \
  --artifact-dir <absolute-product-controlled-package-directory> \
  --expected-source-commit <exact-clean-head> \
  --expected-package-sha256 <composer-receipt-package-sha256> \
  --output-dir <absolute-new-signed-output-directory> \
  <--test-only-ad-hoc|--community-alpha|--production> \
  --preserve-signed-canonical
```

This option is mandatory when product control is present. The script verifies
the existing canonical signature, product-control digest, and
control-to-canonical hash; signs only the launcher, update helper, App, and DMG;
and fails if the canonical bytes change. Production therefore requires the
Developer ID signature on the canonical runtime before the external
launch-grant signing request is created. No private launch or release key is
accepted by the packaging or App-signing commands.

Native CI exercises the complete non-publishing form with an ephemeral in-memory
authority and ad-hoc macOS signatures:

```text
cargo run \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --example native_packaged_product_acceptance \
  --locked -- \
  --output <absolute-new-directory-under-a-private-temp-root> \
  --source-commit <exact-clean-head> \
  --signing-script <absolute-repository-path>/tooling/scripts/macos_native_sign_notarize.sh
```

The acceptance command never writes the ephemeral private keys. It proves
embedded public authority and source-commit presence, empty-`PATH` startup,
redacted two-client inventory, receipt-owned Skills materialization/verification/
refresh, the exact Lite MCP registry plus a representative offline call, and
macOS Keychain save/replace/restart-resolve/remove with random zeroizing test
values that never enter configuration. It also proves product-control
verification, Codex registration repair, packaged-product restart, Codex and
Claude Code install/verify/already-current/remove lifecycles, and preservation
of legacy `qiongli` canaries. Its receipt remains
`accepted-ad-hoc-nonpublishing`; it is not Developer ID, notarization, human UI,
real-provider connectivity, or publication evidence.

The macOS package job also runs the repository acceptance entry point and
uploads its path-redacted receipt with the package:

```text
tooling/scripts/macos_native_acceptance.sh \
  --artifact-dir <absolute-downloaded-artifact-directory> \
  --expected-source-commit <exact-package-source> \
  --expected-package-sha256 <trusted-package-digest> \
  --output <absolute-new-receipt.json>
```

The verifier binds the external expected digest, composer receipt, external
and bundle manifests, fixed Alpha.1 identity, bundle metadata, launcher,
canonical runtime, and update-helper digests, and an isolated empty-`PATH`
startup. On a target
macOS host, `--launchservices-preflight` additionally sends the fixed internal
`--startup-check` through `/usr/bin/open`. A zero exit records only that
LaunchServices accepted the request; it does not prove that the process was
observed or that a window appeared. The command does not bypass Gatekeeper.
Every generated receipt keeps clean-machine, manual scale, VoiceOver,
contrast, production signing, and publication gates open.

Maintainer-controlled macOS signing and notarization use
`tooling/scripts/macos_native_sign_notarize.sh`. Its default path verifies only
the exact externally bound unsigned source; Native CI exercises a separately
labelled ad-hoc test mode. The explicit production mode reads only a Developer
ID identity and `notarytool` credential-profile reference already held by the
macOS Keychain, verifies the expected Team ID, notarizes, staples, assesses the
bundle, retains a signed update ZIP for self-update, and emits a separately
signed/notarized drag-to-Applications DMG for first install. The DMG is mounted
and verified to contain exactly `Qiongli.app` plus an `/Applications` link. The
signing-boundary receipt binds both artifacts while the updater's strict
receipt intentionally remains ZIP-only. It does not accept private-key
or password inputs and never publishes. The embedded desktop manifest remains
the pre-signing source descriptor; the new sidecar receipt, platform signature,
and final archive digest bind the post-signing candidate. Full maintainer usage
and nonclaims are documented in `docs/advanced/native-desktop-alpha.md`.

The approved R3P Community Alpha policy adds a separate zero-cost distribution
lane for macOS arm64, Windows x86_64, and Linux x86_64. R3P-A is implemented in
the shared `distribution` module: it closes the class, platform trust and
warning mappings, exact three-target release set, raw-CI/Stable prohibitions,
and exact protected-environment authorization into strict canonical records
and a verified in-memory capability. It does not make these raw CI outputs
publishable. R3P-B adds a separate read-only exact-head promotion workflow and
the `native_community_alpha_promotion` Rust example. They require current remote
`2.x` HEAD, rebuild all three targets in one run, retain package/acceptance
receipt digests, reject cross-target promotion, and emit only a non-publishing
candidate. R3P-C/R3P-D add the checked-in public authority and
`native_community_alpha_release`: it verifies target receipts, creates the
sorted checksums, CycloneDX 1.6 SBOM, SLSA provenance, bilingual notes, signed
integrity record, and exact-set publication receipt. GitHub emits only a
required-reviewer, read-only authorization artifact; final Ed25519 signing is
offline. The first real workflow run remains pending. The macOS asset remains
ad-hoc/not-notarized, the Windows asset remains without Authenticode, and the
Linux asset remains an AppImage. The production-signed lane above remains
available for later Beta/Stable hardening.
See
`docs/superpowers/specs/2026-07-17-qiongli-community-alpha-distribution-note.md`.

Native CI passes the resulting ad-hoc-signed test artifact to
`tooling/scripts/macos_alpha1_update_journey.sh`. The journey uses an isolated
HOME and empty `PATH` to run the packaged helper through one successful atomic
replacement and one failed-health rollback. It verifies application inode
identity, update-state generation, last-known-good preservation, transaction
cleanup, and the restored or activated bundle signature. The emitted receipt
is explicitly non-publishing: Developer ID, notarization, Gatekeeper, network
selection, clean-machine, and publication gates remain open.

Before replacement, `update install` asks the verified staged `qiongli-cli` to
prepare receipt-owned content from its exact embedded pack. Explicit Skills
materializations are inventoried through the owner-private canonical
`managed-content.json` registry; Codex and Claude Code sources and
registrations are accepted only through their supported Qiongli 2 receipts.
The signed update manifest supplies target-specific PluginBundle launch grants
for both clients. A canonical reconciliation journal binds every old/new
version, pack, destination, receipt, content, and plan digest. The helper
activates these operations with the application and compensates them in reverse
order before app rollback. Before the first rollback rename, the shared executor
verifies every old backup and active/staged identity in the rollback set. Damaged
content or dangling backup links fail without moving another surface. Recovery
supports interruptions between either pair of renames and repeated rollback.
Committed and rolled-back cleanup revalidates every retained surface and remaining
owned cleanup target before deleting anything. Already removed targets support
interrupted cleanup and replay. Staging containers may contain only the expected
staged entry; extra files or links refuse cleanup, and the recovery journal remains
until cleanup succeeds. Containers are removed only when empty.

Config, secret references, research data, unmanaged host bytes, and 1.x content
are outside this transaction.

CLI-first reconciliation can additionally stage a managed CLI binary and its exact
installation receipt in that same transaction. The existing CLI owner performs
source/target/receipt CAS checks and generates the replacement receipt, retaining
any unmanaged predecessor backup. The two operations bind matching old/new binary
and receipt identities; an incomplete or mismatched pair is rejected before writes.
Version 2 reconciliation journals require this pair. Existing version 1 journals
and their operation digests retain their format and recovery behavior; unknown
versions fail closed. Activation and rollback use the same executor as content and
registration. This internal capability is not yet enabled by existing integration
commands: a separately approved activation entry point, transaction locking and
process/version checks remain necessary before standalone update qualification.


R3O Batch 5 exposes that same updater through the Overview Update card. The
typed desktop service owns Stable/Beta selection, signed metadata checks,
download/verification/staging progress, cancellation, revision-bound install
confirmation, native-helper handoff, restart recovery, and fixed path-free
remediation. Network and package work runs outside the render loop and is
polled as bounded update state. Stable excludes prereleases, Beta accepts
eligible Qiongli 2 preview releases, and neither stream reads or modifies
Qiongli 1.x state. Source and ordinary CI packages without embedded production
release authority report update as unavailable instead of accepting runtime
trust overrides.

Headless AccessKit coverage now includes current, available, offline, corrupt,
expired, read-only installation, cancellation, failed-health, recovery, and
restart states in addition to typed install confirmation. The startup preflight
also reports `update_surface: ready`. These checks complement the packaged
ad-hoc helper success/rollback journey. The final exact-head production-signed
metadata and package journey remains a production-lane gate; R3P requires a
separate Community Alpha metadata/package journey and ledger.

The maintainer-only `native_alpha1_release_evidence` example closes the
repository evidence boundary without changing that gate. Its preflight mode
can bind an unsigned exact-head package, while production mode re-verifies the
signed/notarized desktop set, public release authority, Beta metadata, both
client launch grants, Stable rejection, and the production-signed portable
candidate. It derives a sorted checksum manifest, CycloneDX 1.6 SBOM, and SLSA
Provenance v1 statement directly from the locked source and assets. Final
ledger creation additionally requires seven source/release-set-bound macOS
acceptance receipts and their exact attachments. Preflight evidence can never
finalize, and every successful output still records
`publication_allowed: false`. Exact commands and receipt schemas are in
`tooling/release/v2.0.0-alpha.1.md`.

Target-specific install, CLI, removal, trust, and architecture guidance is in
`docs/advanced/native-desktop-alpha.md`.

## Qiongli 1.x replacement migration

Qiongli 2 treats recognized 1.x Plugin, standalone Skills, marketplace, MCP,
and legacy provider-configuration surfaces only as migration inputs. They
never satisfy the current 2.x installation checks. Supported provider settings
are converted to the v2 document; approved plaintext literature-provider keys
move to the OS secret store and never enter the plan or receipt. Unknown
fields and existing v2 provider changes require review. The bounded CLI
workflow is:

```text
qiongli migrate-1x inspect
qiongli migrate-1x preview
qiongli migrate-1x apply --migration-id <id> \
  --expected-plan-digest <sha256> --approve-filesystem-write \
  [--approve-client-config-change] [--approve-secret-store-write]
qiongli migrate-1x continue --migration-id <id> --confirm-host-activation
qiongli migrate-1x continue --migration-id <id> --approve-cleanup
qiongli migrate-1x continue --migration-id <id> --finalize
```

`inspect` is path-free and available from a source build. All migration
mutations require the verified packaged product that created the plan.
`status` reloads the canonical plan, receipt, and current inventory; `recover`
restores only transaction-owned cleanup backups and refuses concurrent
changes. The Desktop Client Integrations page drives the same states through
separate confirmation steps.

Legacy research projects use the existing explicit copy migration:
`qiongli project migrate preview/apply --source ... --root ...`. Qiongli does
not scan a user's home directory for project candidates. This keeps project
selection bounded, leaves the legacy source untouched, and registers only the
verified 2.x destination.

If the process stops after the new project files are committed but before
Research Library registration is completed, recover the exact committed copy
without copying it again:

```text
qiongli project migrate recover preview \
  --source <legacy-absolute-path> --root <committed-2x-path>
qiongli project migrate recover apply \
  --source <legacy-absolute-path> --root <committed-2x-path> \
  --expected-plan-digest <sha256> --approve-filesystem-write
```

Recovery uses the original migration receipt and refuses source, destination,
manifest, inventory, identity, or Library-revision drift. The apply command
must use the plan digest returned by recovery preview.

Migration reconciliation and rollback use the same receipt authority:

```text
qiongli project doctor
qiongli project migrate rollback preview \
  --source <legacy-absolute-path> --root <migration-owned-2x-path>
qiongli project migrate rollback apply \
  --source <legacy-absolute-path> --root <migration-owned-2x-path> \
  --expected-plan-digest <sha256> --approve-filesystem-write
```

The preview reports research-state, decision, evidence, capture, semantic-link,
and continuity artifacts individually. Apply unregisters and removes only the
exact unchanged destination bound by the migration receipt; it always retains
the 1.x source. Source drift, destination academic drift, a conflicting marker,
an unrelated Library entry, or a stale revision blocks deletion. A changed 2.x
project must be exported or explicitly resolved rather than treated as a
rollback target.

The packaged Svelte App exposes the same operation in **Research Library →
Migrate 1.x project**. Real directory paths remain inside the Rust bridge; the
App API receives only an opaque selection token and a bounded label. After
normal migration or recovery, Qiongli rebuilds the Academic Graph index twice
and reports whether the projection and index identities were deterministic.

This is the final 1.x project boundary: 1.x is a user-selected, read-only
migration source and optional retained fallback copy. Qiongli 2 does not launch
a 1.x runtime, scan the home directory for candidates, write the 1.x source, or
require Python/Node to inspect, migrate, recover, reconcile, or roll back a
supported project.

## R4D execution boundary

`qiongli-execution` freezes the first Full-runtime trust boundary. A versioned
asynchronous `AgentBackend` advertises authentication, model, context,
streaming, structured-output, tool-call, multimodal, retry, and cancellation
capabilities. Preflight evaluates the complete normalized request before a
provider or ToolHost side effect, and the deterministic fake backend exercises
the same event stream and cancellation contract as future direct adapters.

Provider adapters receive no filesystem handle, process launcher, raw secret,
approval authority, or ToolHost capability. Model-emitted tool requests enter
`AgentExecutionPolicy`, where a closed Lite/Full profile, exact allowlist,
registered project identity and revision, relative artifact set, execution
limits, and a short-lived user/admin approval bound to the normalized request
digest produce `allow`, `deny`, or `approval-required`.

`ToolHostRegistry` accepts only typed tools. In-process registration is limited
to explicitly read-only service calls; every broader class is reserved for the
authenticated child boundary. Prepared ToolHost invocations carry the exact
policy decision, registered root, limits, and strict redaction policy. Audit
records contain identities, hashes, timing, counts, outcomes, and fixed reason
codes but never tool arguments, absolute paths, secrets, or unrestricted model
text.

ADR 0211 supersedes the original direct-provider closure path. The default
product is now host-driven: Codex, Claude Code, or another supported host owns
model authentication, conversation, and execution. Qiongli owns installation,
project state, Full MCP tools, orchestration checkpoints, and strict handoff
validation. It does not silently contact a provider or launch a model CLI when
the host path is unavailable.

The third R4D batch moves the nine existing Full project operations out of the
App entrypoint and into one shared `FullProjectService`, so Full MCP and
ToolHost no longer carry separate academic behavior. The in-process ToolHost
can dispatch the eight explicitly read-only project/library/graph/capture-
preview operations after revalidating the request-bound project identity,
semantic revision, and registered root. It enforces cancellation, input/output
limits, bounded JSON depth/count, fixed error classes, result redaction, and
hash-only audit metadata. `qiongli_project_capture_apply` is registered only as
a project-write `reserved-child` operation and has no in-process handler.

The host-driven contracts live in `qiongli-execution`. A runtime descriptor
declares the host family and supported capabilities. An orchestration handoff
binds the project ID and revision, run, task, role, attempt, checkpoint digest,
allowed tool evidence, and execution limits. A candidate envelope must bind
those values exactly before any Qiongli-owned state transition can accept it.
Unknown fields and undeclared evidence fail closed.

The default App capability snapshot reports direct backend configuration,
connection testing, agent runs, and the retired embedded orchestration surface
as unavailable. The former Model Backend page is migration-only and can remove
an old credential while disabling the legacy backend. The default CLI retains
only the redacted, non-network `config backend status` command. Full MCP no
longer advertises backend status/test/run tools or provider-dependent
orchestration start/continue tools; it retains local project and checkpoint
controls needed by the host.

The direct backend and bounded runner remain implementation evidence for a
separately gated experimental path. They are not a fallback and are not part
of ordinary acceptance. R4 acceptance now requires a supported host to call
Qiongli Full MCP, produce a strictly bound candidate, and preserve Qiongli's
local approval and checkpoint rules without storing host credentials or
conversation text.

The generic host execution service now exposes
`qiongli_orchestration_doctor`, `qiongli_orchestration_start`,
`qiongli_orchestration_next`, `qiongli_orchestration_read`, and
`qiongli_orchestration_submit`. A ready host descriptor is bound into the
persisted orchestration profile; changing the host version, adapter, capability
shape, or activation state cannot resume that run. Every transition requires
the exact project revision, checkpoint generation, and run-document SHA-256.
Reissuing an active handoff is idempotent and does not advance the checkpoint.

The `read` wrapper accepts only one read-only operation already offered by the
handoff. It returns the ordinary project result plus a hash-only evidence
reference in `structuredContent.qiongliOrchestration.evidence`, with an
identical copy in MCP `_meta` for clients that expose it; the ledger binds that
reference to the project, revision, run, and handoff. This dual projection is
required because Claude Code does not expose tool-result `_meta` to the model.
`submit` rejects invented, altered, cross-run, or already-consumed references,
validates the candidate envelope, persists only its digest, and returns the
next handoff. MCP `clientInfo` is display-only and never grants host trust.
Copied-binary stdio acceptance runs the full
doctor/start/next/read/submit sequence with an empty `PATH`, without a provider
request or model CLI subprocess.

The H3 Codex adapter packages that generic service as one receipt-owned Plugin.
Its generated `.mcp.json` starts the bundled native binary with the Full
profile through an exact bundle-relative path; it never resolves `qiongli` or
`codex` from `PATH`. Bundle receipt schema 2 records the Skills projection and
Full MCP projection separately. Codex installation consumes an explicit signed
`full-mcp` grant mode, so a legacy Lite-only grant cannot authorize the Full
runtime. Packaged-product control schema 2 records Codex as the current Full
MCP target while Claude Code remains Lite until H4. The projected Codex Skill
tells the active conversation to drive doctor/start/read/submit/next, preserve returned evidence
bindings, fall back to one truthful host agent unless native subagents are
actually exposed, and keep artifact preview separate from explicit apply
approval.

Client inventory schema 2 reports the receipt-verified Full MCP declaration
independently from Plugin registration. Registration alone is not activation:
the desktop keeps the runtime observation as unknown until Codex supplies
activation evidence, so an installed source cannot be mislabeled Connected.
The isolated real-Codex acceptance covers Plugin install, list, cache
verification, empty-`PATH` Full MCP launch, removal, and preservation of
unmanaged content without a provider request.

## R4E orchestration state foundation

The first R4E batch adds the provider-independent ORC-201 state core without
starting a model or widening ToolHost authority. `OrchestrationTaskGraphV1`
accepts at most 128 declared tasks, validates closed `prerequisites_all` and
`prerequisites_any` references, rejects duplicate dependencies and cycles, and
keeps declaration order as the deterministic scheduling priority.

`OrchestrationProfileV1` binds a closed solo, duo, or triad role shape to
explicit backend identities, one to three task attempts, and a declared
stop-on-failure policy. It does not discover or invoke Codex, Claude,
Antigravity, Python, Node, or another external process. Worker fan-out and
synthesis remain ORC-202 work.

Each `OrchestrationCheckpointV1` is bound to one run ID, project ID, exact
semantic revision, graph digest, and profile digest. Mutations require the
caller's expected monotonic generation, so stale UI, CLI, or MCP actions fail
before changing state. Checkpoints contain only task IDs, closed states,
attempt counts, output hashes, and a closed failure-code enum; prompts, model text,
absolute paths, credentials, and research artifact bodies are excluded.

The state machine supports deterministic ready-task selection, bounded retry,
dependency blocking, completion, explicit pause/resume, interrupted-task
recovery, and terminal cancellation. Canonical JSON restore rejects unknown
fields, non-canonical bytes, impossible task states, stale project revisions,
and graph/profile substitution. This deliberately improves on the frozen 1.x
boundary where durable task/team resume and public cancellation were absent.

The same batch includes a compact task-only projection of the frozen 76-task
`research-workflow-contract.yaml`. The projection contains only task IDs and
required dependency edges; it is not a second academic-policy authority.
`from_embedded_content` loads it only after the Full resource pack exposes the
exact source path and the resource entry plus bytes match the frozen source
SHA-256. Contract drift therefore disables orchestration instead of silently
running a stale graph. Executing role stages, durable checkpoint storage,
workers, synthesis, review, artifact mutation, quality gates, and product
surfaces remain later R4E batches.

## R4E single-task execution and recovery

The second R4E batch makes the ORC-201 state contract durably runnable without
adding worker fan-out or a new ToolHost authority. Each registered project may
hold at most 128 orchestration run documents under its private
`.qiongli/orchestration/` runtime directory. These files are non-portable
operational state: they do not change the project's semantic revision and do
not enter its academic export.

`ProjectStateService` resolves the registered project and exact semantic
revision before each read or replacement. Creation and replacement are bounded
to one MiB, use an owner-private lock plus atomic file promotion, compare the
expected prior document SHA-256, reject linked or insecure paths, and expose no
absolute path in their result. Listing is sorted, bounded, and locked against
an in-progress atomic promotion.

The canonical run document stores the safe `OrchestrationProfileV1` together
with its checkpoint, allowing restart discovery to reconstruct the exact solo,
duo, or triad plan against the verified embedded task graph. It stores backend
identities, roles, states, attempts, fixed failures, and hashes, but no
credential, prompt, model text, tool result, transcript, absolute path, or
artifact body. Unknown fields, non-canonical bytes, profile substitution,
graph substitution, run/file identity drift, and stale compare-and-swap writes
fail closed.

`OrchestrationTaskExecutor` now drives one ready task through the existing
`BoundedAgentRunner` in exact primary, reviewer, then verifier order. Every
role receives a domain-separated child run ID; all required backend runners and
their project/revision/root policy scopes are verified before a planned run
starts. The current persisted document is re-read before every backend call.
Prior role output is available only in memory to the next role, while a
successful role persists only its content hash.

Retryable backend failures return the whole task to `ready` within its declared
attempt bound. A process interruption leaves a visible running checkpoint that
must be explicitly recovered to `paused`; recovery clears partial role hashes
and restarts the complete role chain after resume. Explicit pause, resume, and
terminal cancellation use the same generation and document-CAS boundary.
Connecting the embedded task/role content to a product request builder and
exposing run discovery/actions through App and Full MCP are the next ORC-201
batch. Worker concurrency and synthesis remain ORC-202.

## R4E embedded task inputs and Full MCP control plane

The third R4E batch makes the single-task executor usable through the native
Full MCP boundary while keeping artifact writes, worker fan-out, and the
desktop Orchestrator view out of scope. The execution crate now parses the
`task_catalog` section of the verified embedded
`standards/research-workflow-contract.yaml` into a closed 76-task catalog. It
rejects missing, duplicate, malformed, oversized, or graph-divergent task
definitions instead of accepting an ungrounded Task ID.

`EmbeddedWorkflowRoleInputBuilder` turns one task and role into a bounded agent
request containing the registered project ID, exact semantic revision, task
stage, title, candidate output names, attempt, and fixed role instruction.
Primary, reviewer, and verifier use distinct instructions. Prior role output is
accepted only in the exact expected order and size, labelled as untrusted
evidence, passed in memory, and excluded from the checkpoint document. Every
role remains limited to the existing project-scoped read ToolHost, so the
result is explicitly a candidate rather than a claim that an academic artifact
was written or approved.

Full MCP adds five closed tools:

- `qiongli_orchestration_doctor` verifies project/revision binding, embedded
  contract availability, backend readiness, and interrupted-run state without
  a network request;
- `qiongli_orchestration_runs` returns only redacted run state, action
  availability, generation, and document digest;
- `qiongli_orchestration_test` starts one solo, duo, or triad run and executes
  its next deterministic task after explicit network confirmation;
- `qiongli_orchestration_continue` advances an unchanged run after the same
  confirmation; and
- `qiongli_orchestration_action` pauses, recovers, resumes, or terminally
  cancels only when the supplied generation and document SHA-256 still match.

Only one non-terminal run may be started for a project at a time. Backend
readiness failure happens before checkpoint creation, and a stale control
reference fails before mutation. The returned test output is visible to the
caller but only its SHA-256 enters private runtime state. Deterministic tests
exercise the real input builder with a fake backend, model-text exclusion,
pause/resume/cancel CAS, interrupted recovery without a backend, copied-binary
Full MCP discovery and cancellation, malformed arguments, and disabled-backend
preflight. No real provider or formal security scan is invoked. The following
ORC-201 batch exposes these same views and actions through the typed App API and
desktop Orchestrator view before ORC-202 adds workers and synthesis.

## R4E typed App and desktop orchestration control plane

The fourth R4E batch completes the ORC-201 product surface without creating a
second orchestration implementation. The versioned App API now exposes
revision-bound doctor, run-list, run-summary, role-output, and execution views.
Test and continue intents carry the selected project revision; continue and
control intents additionally carry the exact run ID, checkpoint generation,
and run-document SHA-256. Unknown fields, private paths, and stale checkpoint
references remain outside the IPC contract.

The desktop adapter owns one `FullOrchestrationService` over the same registered
project and embedded workflow services used by Full MCP. Loading the
Orchestrator is offline. Starting or continuing a run produces a native
operation preview first and does not call the provider until the user selects
**Confirm and run**. Pause, recover, resume, and terminal cancel use the same
generation/document compare-and-swap boundary. Each control response reloads
the doctor and complete run list in the same response, so cancellation and
recovery do not leave stale action availability in the UI.

The macOS desktop view provides:

- active-project and semantic-revision selection;
- embedded 76-task contract and backend readiness status;
- solo, duo, and triad bounded-test previews;
- persisted progress, next task, generation, recovery, and closed control
  actions; and
- bounded role output display with model identity and output digest.

English and Simplified Chinese labels, keyboard-visible focus, screen-reader
regions, reduced-motion behavior, a scrollable small-window sidebar, and an
explicit terminal-cancel confirmation cover the current accessibility and
error-prevention baseline. The source-build fixture exercises load, preview,
confirm, output, pause, and localization without a real provider. Rust and
TypeScript contract fixtures cover every event variant. No live provider or
formal security scan is part of this batch.

ORC-201 is now available through the native service, Full MCP, typed App API,
and desktop UI. ORC-202 worker fan-out and synthesis, ORC-203 artifact/review
and quality gates, and the opt-in live provider acceptance remain subsequent
R4E work.

## R4E bounded worker fan-out and synthesis

The fifth R4E batch implements ORC-202 without launching Codex, Claude,
Antigravity, or another external CLI. A versioned `WorkerOrchestrationPlanV1`
binds one supported Task ID, registered project, exact semantic revision, run
ID, backend assignment, merge policy, barrier policy, and one to four bounded
workers. `B1` uses the frozen delegated-worker profile with three functional
workers, a two-of-three minimum, and degraded synthesis when that threshold is
met. `H3` uses the frozen three-persona review swarm and blocks unless every
worker passes.

Worker checkpoints use an independent owner-private
`.qiongli/worker-orchestration/` CAS namespace. They contain only closed
states, attempts, fixed failure codes, generations, plan bindings, and output
SHA-256 values. Worker, synthesis, and review text remains in memory and is
returned only to the current caller. A restart that has lost those in-memory
values cannot continue from hashes as if content were available: explicit
recovery resets the affected fan-out for bounded replay.

`WorkerOrchestrationExecutor` drives the generic provider-neutral path through
the existing `BoundedAgentRunner` and project-scoped read-only ToolHost:

```text
bounded workers -> success barrier -> controller synthesis
                -> independent closed review -> completed or blocked
```

Every phase has a domain-separated agent run ID and re-reads the current CAS
document before another backend request. The embedded input builder labels
worker and synthesis text as untrusted, bounds content passed between phases,
preserves disagreement and gaps, prohibits artifact-write claims, and accepts
only the closed final verdicts `ACCEPT`, `REVISE`, or `BLOCK`. Canonical
artifact writes, merge reports, academic validator gates, and useful Stage I
mutation remain ORC-203.

Full MCP adds four closed worker tools:

- `qiongli_worker_orchestration_runs` lists redacted worker checkpoints
  offline;
- `qiongli_worker_orchestration_test` explicitly runs the supported `B1` or
  `H3` profile after network confirmation;
- `qiongli_worker_orchestration_continue` retries an unchanged retry-ready
  checkpoint; and
- `qiongli_worker_orchestration_action` explicitly recovers hash-only partial
  state or terminally cancels an unchanged run.

Deterministic fake-backend coverage includes complete fan-out/synthesis/review,
degraded and blocked barriers, retry, stale generation and document rejection,
model-text exclusion, restart replay, scope mismatch, and pre-request
cancellation. Copied-binary Full MCP acceptance covers discovery, closed
schemas, disabled-backend preflight, and response/path redaction. No provider
request, platform-native worker adapter, external process, or formal security
scan is used by these tests.

## R4E artifact review and quality-gate state

The sixth R4E batch starts ORC-203 at the provider-independent state boundary.
`ArtifactReviewPlanV1` binds a completed single-task or worker-synthesis source
run to a separate review run, registered project, exact semantic revision,
Task ID, source result SHA-256, and exact workflow, capability-map, and
quality-gate contract digests.

Candidate artifact records contain only bounded project-relative paths,
create/update intent, prior and proposed content SHA-256 values, and byte
counts. Review checkpoints contain the closed `Q1` through `Q4` gate IDs,
statuses, evidence hashes, and an independent `ACCEPT`, `REVISE`, or `BLOCK`
verdict hash. Prompt text, model output, candidate bodies, transcripts,
credentials, and absolute paths are not durable state.

Every transition requires the exact monotonic generation. A candidate becomes
`ready-for-apply` only when every required gate is `PASS` and the independent
review verdict is `ACCEPT`; warnings or failures cannot be overridden.
Canonical restore rejects unknown fields, non-canonical bytes, stale state,
binding substitution, duplicate gates or artifacts, and impossible terminal
combinations.

This batch deliberately grants no write authority. The next ORC-203 batch must
derive allowed artifact boundaries and required gates from the verified
embedded workflow and capability contracts, persist plans through a private
project CAS store, and expose preview-only discovery before implementing any
explicitly approved canonical mutation.

## R1 command contract (parser-retained compatibility)

The native executable composes the verified embedded pack and versioned global
config service through this first useful command surface:

```text
qiongli --help
qiongli --version
qiongli content --help
qiongli content list
qiongli config --help
qiongli config show
qiongli config set --expected-revision <revision> --default-profile <profile>
qiongli config backend status
qiongli status
qiongli doctor
qiongli paths
qiongli paths --json
qiongli doctor --paths exact
```

The former
`qiongli content materialize --profile <profile> --target <absolute-path>`
syntax remains parseable so upgrades fail with a stable
`managed-skills-plan-required` result, but it no longer writes. R5F Skills
mutations use the reviewed `qiongli app plan ...` / `qiongli app apply`
contract. A new custom destination is selected in the Desktop App so its
absolute path remains private to the native service; the resulting anonymous
target is then verifiable, updatable, and removable from either App or CLI.

Data commands emit a newline-terminated JSON object with `schema_version: 1`.
Usage failures return exit code 2, operation failures return exit code 1, and
public errors contain only allowlisted reason codes. Materialization paths,
config roots, environment values, provider identifiers, and document bytes are
not rendered. `config set` changes only the default profile, preserves provider
settings, and requires an optimistic expected revision. The R3Q Product Doctor
extends the original foundation with managed-content receipts, Codex and Claude
Code integration state, the Lite MCP offline contract, literature-provider
readiness, and update/recovery checks. R4 now owns the frozen ToolHost
contracts, shared read-only Full project dispatch, host runtime and handoff
contracts, and redacted local checkpoint controls. Direct provider execution
is non-default experimental evidence. Reserved-child project writes remain
unavailable until their later acceptance gates pass.

Ordinary `status` and `doctor` output remains path-redacted. `qiongli paths` is
the explicit human-readable exact-path view; `qiongli paths --json` emits its
versioned typed snapshot, and `qiongli doctor --paths exact` attaches the same
snapshot to Doctor output. Entries report source, scope, selection, existence,
file type, owner, writability, safety, and symlink/reparse resolution without
reading unrelated directory contents. App Diagnostics consumes the same native
snapshot and hides exact paths until the user explicitly reveals them.

Binary contract tests clear `PATH` before supported commands and also copy the
executable outside the checkout before listing and materializing embedded
content. This prevents developer or CI installations of Python, Node.js, Cargo,
or other tools—and a source-relative working directory—from masking a required
runtime dependency.

## Version and toolchain

`Cargo.toml` is the single native product-version source. The release channel
is explicit workspace metadata and must agree with the SemVer prerelease.
`rust-toolchain.toml` pins the build toolchain used by the Tier 1 CI matrix.

REL-201 exposes that identity through `scripts/release_version.py` and a
non-publishing dry-run. The dry-run writes a release plan, notes, a planned-only
target identity, and rollback metadata to an explicit directory outside the
checkout:

```bash
OUT_DIR="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-native-release.XXXXXX")"
python3 scripts/native_release_dry_run.py \
  --tag v2.0.0-alpha.1 \
  --out-dir "$OUT_DIR" \
  --json
```

This is planning evidence, not permission to publish. It does not build an
installer, modify Git, publish PyPI/npm or Marketplace records, or claim the
target-native, signing, SBOM, provenance, updater, and rollback gates owned by
later roadmap tasks.
The direct command leaves source-ref and cleanliness unassessed. Use
`scripts/release_automation.sh pre` from a clean `2.x` checkout when the plan
must carry an eligible source binding.

Run the foundation gates from the native workspace so Rustup applies the
workspace-local pinned toolchain:

```bash
cd packages/qiongli-native
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```
