# Local Desktop Development and Packaging

This guide is the maintainer fast path for the Qiongli 2 Svelte/Tauri desktop
application. It covers source development, native execution, local package
assembly, and the difference between a source-built package and a releasable
product.

## Know Which Build You Are Running

| Build | Native services | Writes real Qiongli state | Packaged-product authority | Intended use |
|---|---:|---:|---:|---|
| Browser fixture | No | No | No | Fast layout, responsive, and localisation work |
| `cargo run` source App | Yes | After explicit preview/confirmation | No | Native UI, CLI, project, and service development |
| Local source package | Yes | After explicit preview/confirmation | No | Package layout and target-native launch testing |
| Product-controlled acceptance App | Yes | Only inside its isolated test home | Ephemeral test authority | Automated install/update/integration acceptance |
| Promoted release | Yes | After explicit preview/confirmation | Production or Community Alpha authority | Distribution to testers or users |

A normal source build intentionally has no packaged-product authority. Messages
such as `Confirmation is unavailable because this source build has no
packaged-product authority`, unavailable Apply operations, and unavailable
client installation or updates are therefore expected. Do not add a bypass to
make source builds look like released products.

## One-command macOS Build

When you only need to test the complete App on the current Mac—not Windows or
Linux, release acceptance, notarisation, or an installer—run this from the
repository root:

```bash
pnpm desktop:macos
```

Run `pnpm install --frozen-lockfile` once before the first build. The command
builds the Svelte static assets and the current Mac's Rust executable, then
creates a locally ad-hoc-signed App at:

```text
dist/macos/Qiongli.app
```

The App bundle contains both `Contents/MacOS/Qiongli` and the matching
`Contents/MacOS/qiongli-cli`. In **About → Qiongli CLI**, the App reports the
bundled version, the managed `~/.local/bin/qiongli` target, and whether another
pip/npm command shadows it on the observed `PATH`. The ordinary source package
can inspect this state but cannot write to the user CLI directory because it
has no packaged-product authority.

It does not run cross-platform gates, security scans, the release composer,
notarisation, or product-control acceptance. Build and open it in one command
with `pnpm desktop:macos:open`.

The command deliberately uses Cargo's release profile and enables Qiongli's
`custom-protocol` feature so Tauri serves the embedded Svelte assets instead of
connecting to `http://127.0.0.1:1420`. This is still a source App for local
Tauri/Svelte and native-service testing. It has no packaged-product authority
and is not a distributable release. The longer composer and signing flow later
in this guide remains the release-structure, update-chain, and
distribution-acceptance path.

### Test the host-driven model path from the source App

Qiongli no longer owns model credentials, provider connection tests, prompts,
or model conversations in its default product path. Codex, Claude Code, or
another supported host owns authentication and model execution. Qiongli
provides the installed Plugin, Full MCP tools, project state, checkpoint state,
and revision-bound handoff contracts.

Use this local acceptance sequence:

1. Run `pnpm desktop:macos:open`.
2. Open **Client Integrations** and verify that the intended host is detected
   and that its Plugin and Full MCP attachment are installed or previewable.
3. Create or register a project in **Research Library** and note its current
   revision.
4. Start Codex or Claude Code with the installed Qiongli integration.
5. Ask the host to read the project through Qiongli Full MCP and prepare a
   revision-bound candidate. The host, not Qiongli, must display and execute
   the model conversation.
6. Verify that Qiongli rejects stale project revisions, mismatched checkpoint
   digests, undeclared evidence references, and unknown handoff fields.
7. Return to the App and confirm that project and checkpoint state reflects
   only accepted local actions. The App must not display a provider key form,
   connection-test button, model prompt, or direct model answer.

`qiongli config backend status` remains a non-network, redacted migration view.
The default CLI rejects `config backend set` and `config backend test`.
The legacy backend page is reachable only as a cleanup surface and can remove
an old credential while disabling the old backend.

The former R4D live-provider acceptance command is not part of the host-driven
acceptance gate. Do not use it to validate the default architecture. The
replacement generic acceptance is:

```bash
cargo test -p qiongli --test mcp_stdio \
  copied_full_binary_completes_host_handoff_round_trip_without_model_transport
```

It binds the host runtime descriptor, orchestration handoff, host-produced
candidate, project revision, checkpoint digest, and declared MCP evidence
without storing host credentials or conversation text. It runs a copied native
binary with an empty `PATH`; it does not validate a real Codex or Claude plugin
installation, which belongs to the next host-adapter batch.

## One-command macOS Install Acceptance

The ordinary source App cannot install Qiongli Skills or client plugins because
it deliberately has no signed product authority. To test those actions without
touching the real `~/.codex` or `~/.claude` directories, run:

```bash
pnpm desktop:macos:acceptance:open
```

The acceptance command requires a clean Git worktree. This prevents a package
containing uncommitted code from presenting the previous commit as its build
identity. Commit the intended source first; use `pnpm desktop:macos:open` while
iterating on uncommitted UI or native changes.

This command builds the same embedded Svelte application with an ephemeral
development authority, composes and ad-hoc-signs a non-publishing package, and
then completes automated Skills materialize/verify/refresh plus Codex and Claude
Code install/verify/repair/remove acceptance. It also creates a separate
nine-surface Qiongli 1.x fixture (eight client-integration surfaces plus the
legacy provider configuration), runs the complete
preview/stage/verification/cleanup/finalize migration, verifies the converted
2.x provider settings, verifies that no recognized 1.x surface remains, and
re-verifies both 2.x client installations.
Only after all checks pass does it publish the test App under:

```text
dist/macos-acceptance/current/extracted/Qiongli.app
```

Automated destructive checks use
`dist/macos-acceptance/current/automated-home`. The opened process receives a
separate clean `HOME` at `dist/macos-acceptance/current/manual-home`; it
therefore discovers test-only Codex and Claude directories and cannot write
integration state to the real user home. Use the App's normal preview and
confirmation UI to test installation again interactively. Evidence is recorded in
`qiongli-packaged-product-acceptance.receipt.json` beside that test home.
The same isolated App can preview and install its bundled native CLI from
**About → Qiongli CLI**. The managed target is inside `manual-home/.local/bin`,
so this acceptance path never replaces a real pip/npm installation.
The dedicated replacement fixture uses
`dist/macos-acceptance/current/legacy-migration-home`; it is never opened as
the manual UI home.

Legacy research projects are deliberately not found by scanning `HOME`.
In **Research Library**, choose **Migrate 1.x project**, enter the new 2.x
project identity, then select the legacy source and a new empty destination.
The confirmation preview reports copied files, bytes, exclusions, source
retention, and the exact plan digest. Confirmation registers the destination
and rebuilds the Academic Graph index twice; the completion notice distinguishes
a verified deterministic rebuild from a rebuild that still needs attention.

If the App or CLI stopped after the destination files were committed but before
Research Library registration finished, choose **Migrate 1.x project → Resume
migration** after restart and reselect the unchanged source plus the committed
destination. Recovery validates the original receipt and completes registration
without copying again. The equivalent recovery CLI is:

```bash
qiongli project migrate recover preview \
  --source <legacy-project> --root <committed-2x-project>
qiongli project migrate recover apply \
  --source <legacy-project> --root <committed-2x-project> \
  --expected-plan-digest <preview-digest> --approve-filesystem-write
```

The source remains untouched, and only the verified new 2.x project is
registered. Keep the source until the migrated project has been reopened and
its graph has been inspected.

To inspect incomplete migration markers or derived-index state without
modifying a project, run:

```bash
qiongli project doctor
```

To test rollback, create a disposable fixture rather than selecting a real
project. In **Research Library → Migrate 1.x project**, choose **Rollback
migrated copy**, reselect the unchanged 1.x source and the exact 2.x destination,
and review the item-scoped reconciliation. The equivalent CLI is:

```bash
qiongli project migrate rollback preview \
  --source <unchanged-legacy-project> --root <migration-owned-2x-project>
qiongli project migrate rollback apply \
  --source <unchanged-legacy-project> --root <migration-owned-2x-project> \
  --expected-plan-digest <preview-digest> --approve-filesystem-write
```

Rollback first revalidates the receipt, Library revision, registration marker,
manifest, and every migrated artifact. It unregisters and removes only the
exact unchanged migration-owned 2.x directory and never modifies the 1.x
source. If the 2.x project has changed, preview is blocked; export or explicitly
resolve that project before retrying. The `2.0.0-alpha.2` project-data flow was
accepted with an ad-hoc release-profile App and disposable isolated-home
fixtures; its non-publishing interaction receipt is
`dist/macos-r5a-manual/current/r5a-project-manual-acceptance.receipt.json`.

The package is labelled by its acceptance output location, uses ad-hoc signing,
sets `publication_allowed` to `false`, and has install grants that expire after
one hour. Re-run the command after expiry. Do not copy it into `/Applications`,
open it through Finder, or distribute it: either route would discard the
isolated launch environment or misrepresent non-publishing test evidence.

## Current-host Claude Desktop Full MCPB

Claude Desktop supports local binary MCP servers through installable `.mcpb`
Desktop Extensions. Build the separately labelled Qiongli Full package for the
current operating system and architecture with:

```bash
pnpm mcpb:pack:full
```

The command builds the Rust `qiongli` release executable, launches it with an
empty `PATH` and an isolated configuration home, verifies the exact 30-tool
Lite + project + host-orchestration inventory, and writes:

```text
dist/qiongli-full-runtime-2.0.0-alpha.2.mcpb
dist/qiongli-full-runtime-2.0.0-alpha.2.receipt.json
```

Install the MCPB manually from **Claude Desktop → Settings → Extensions →
Advanced settings → Install Extension…**. The host owns install, trust,
enablement, restart, live attachment, and tool approval; the local build
receipt proves only package bytes, target identity, and runtime inventory. It
therefore records `publication_allowed: false` and never claims that Claude
Desktop is connected.

The existing `qiongli-literature-provider-*.mcpb` remains Marketplace Lite.
Neither MCPB activates Claude Web, Codex Cloud, or another remote worker.
Official Claude Desktop installation guidance is maintained in
[Getting Started with Local MCP Servers on Claude
Desktop](https://support.claude.com/en/articles/10949351-getting-started-with-local-mcp-servers-on-claude-desktop).

## Prerequisites

Use the versions exercised by native CI:

- Node.js 24;
- pnpm 11.13.1, as pinned by the root `packageManager` field;
- Rust 1.97.0 with `rustfmt` and `clippy`, pinned by
  `packages/qiongli-native/rust-toolchain.toml`;
- the platform WebView and native build prerequisites required by Tauri 2.

The repository does not require a globally installed Tauri CLI for the
supported commands below. On macOS, install Xcode Command Line Tools. On
Windows, install the MSVC C++ build tools, Windows SDK, and WebView2. Debian or
Ubuntu developers can match CI with:

```bash
sudo apt-get install --no-install-recommends \
  libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev patchelf
```

From the repository root, install the locked frontend dependencies and verify
the selected tools:

```bash
pnpm install --frozen-lockfile
node --version
pnpm --version
(cd packages/qiongli-native && rustc --version)
```

## Fast Svelte UI Loop

Run the Svelte application with its read-only development transport:

```bash
pnpm --dir packages/qiongli-desktop dev
```

Open
`http://127.0.0.1:1420/?fixture=source-read-only`. The fixture provides typed,
deterministic sample data and never invokes native commands or writes project
state. A plain browser URL without the fixture is not a substitute for the
Tauri host because the native IPC bridge is absent.

Use this loop for styling, compact layouts, responsive behaviour, localisation,
and component states. Confirm native actions in the full App before considering
the work complete.

## Build and Run the CLI without the Desktop

The default `qiongli` package has no GUI feature. It builds and tests without
Svelte output, Tauri, webview libraries, or a graphical file picker:

```bash
cargo build --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --bin qiongli --locked
cargo test --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --all-targets --no-default-features --locked
cargo run --manifest-path packages/qiongli-native/Cargo.toml -p qiongli --locked -- --help
```

No arguments print help in this build. Existing JSON commands and
`mcp serve --profile <lite|marketplace-lite|full> --transport stdio` retain
their existing interfaces. `ui` cannot launch a window; `ui --startup-check`
still checks the shared service, not window availability. App inspection and
managed-operation commands retain their shared owners and approval checks.

Select `desktop` for Tauri development, or `custom-protocol` (which also
selects `desktop`) for embedded frontend assets. The `qiongli-desktop`
launcher requires `desktop`; existing package builds already select
`custom-protocol`. Whole-workspace/all-feature tests include GUI dependencies
and do not prove the CLI-only dependency boundary.

The CLI still verifies its embedded content, Companion artifact, and available
release authority. A source build does not qualify a standalone managed install;
independent package/install work remains CLI-403.

## Run the Full Source App

Build the static Svelte assets, then run the canonical Rust executable:

```bash
pnpm desktop:build
cargo run \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --features custom-protocol \
  --locked
```

With `custom-protocol` selected, `cargo run` opens the desktop window because
no CLI arguments were supplied. After changing Svelte code, run
`pnpm desktop:build` again before restarting the native App. Rust changes are
rebuilt by Cargo automatically.

The source App uses native services and can discover actual local clients and
projects. Mutating actions still require the application's preview and
confirmation flow. To inspect the same native state without opening a window:

```bash
cargo run \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --locked \
  -- doctor
```

After a successful build, the direct development executable is at
`packages/qiongli-native/target/debug/qiongli` (or `qiongli.exe` on Windows).
It still depends on neither Node nor pnpm at runtime; those tools are only
needed to rebuild the embedded frontend.

## Validate a Desktop Change

Run the smallest relevant checks while iterating:

```bash
pnpm desktop:check
pnpm desktop:test
pnpm desktop:build
cargo test \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --locked
```

Before submitting a native change, run the complete native gates documented in
`CONTRIBUTING.md`. Before submitting documentation changes, also run
`pnpm docs:build`.

## Measure the Opt-in Platform Capacity Baseline

The capacity baseline is a release-mode observation, not a daily test or a
performance gate. Run it only when collecting explicit Linux, macOS, or Windows
capacity evidence. Use a clean committed checkout for a trustworthy source
binding. On macOS or Linux, use a fresh output directory and bind the receipt to
the current commit and a positive run ID:

```bash
CAPACITY_OUTPUT_DIR="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-capacity.XXXXXX")"
export QIONGLI_CAPACITY_OUTPUT_DIR="$CAPACITY_OUTPUT_DIR"
export QIONGLI_CAPACITY_SOURCE_COMMIT="$(git rev-parse HEAD)"
export QIONGLI_CAPACITY_RUN_ID="$(date -u +%s)"
```

On Windows PowerShell, set the same contract fields:

```powershell
$CapacityOutputDir = Join-Path $env:TEMP ("qiongli-capacity-" + [guid]::NewGuid())
$env:QIONGLI_CAPACITY_OUTPUT_DIR = $CapacityOutputDir
$env:QIONGLI_CAPACITY_SOURCE_COMMIT = (git rev-parse HEAD).Trim()
$env:QIONGLI_CAPACITY_RUN_ID = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds().ToString()
```

Then run the same single command on either platform:

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --lib --release --locked platform_capacity_baseline -- --ignored --test-threads=1
```

The output directory must contain both `qiongli-project-capacity.json` and
`qiongli-desktop-capacity.json`. Together they bind the exact 40-character
source commit, positive run ID, target, deterministic fixture identities, raw
samples, and P50/P95 observations. Missing or malformed evidence fails the run;
the values do not define latency, memory, or payload budgets.

Native CI runs this command only for an explicit `workflow_dispatch` and
uploads one `qiongli-capacity-<platform>-<source>` artifact per Tier 1 target.
Ordinary pull requests compile the ignored harness but do not execute or upload
it. A feature-branch dispatch does not authorize package, candidate, signing,
promotion, or publication jobs.

## Assemble a Local Source Package

Qiongli does not use the generic Tauri bundler as its authoritative packaging
boundary: `packages/qiongli-native/apps/qiongli/tauri.conf.json` keeps
`bundle.active` disabled. The repository's native composer binds the canonical
runtime, thin desktop launcher, update helper, embedded resource pack,
application metadata, target, and source commit into one verified archive and
receipt.

Use a clean committed checkout for a trustworthy source binding. From the
repository root on macOS or Linux:

```bash
set -euo pipefail

REPO_ROOT="$(pwd -P)"
SOURCE_COMMIT="$(git rev-parse HEAD)"
PACKAGE_PARENT="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-local-package.XXXXXX")"
PACKAGE_ROOT="$PACKAGE_PARENT/artifact"
TARGET_DIR="$REPO_ROOT/packages/qiongli-native/target/release"

pnpm desktop:build
QIONGLI_NATIVE_SOURCE_COMMIT="$SOURCE_COMMIT" cargo build \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --release \
  --bins \
  --features custom-protocol \
  --locked

QIONGLI_NATIVE_SOURCE_COMMIT="$SOURCE_COMMIT" cargo run \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --example native_desktop_package \
  --release \
  --locked \
  -- \
  --canonical "$TARGET_DIR/qiongli" \
  --launcher "$TARGET_DIR/qiongli-desktop" \
  --update-helper "$TARGET_DIR/qiongli-update-helper" \
  --output "$PACKAGE_ROOT" \
  --source-commit "$SOURCE_COMMIT"

printf 'Local package: %s\n' "$PACKAGE_ROOT"
```

The output must be a new absolute path outside the checkout. The composer
rejects an existing output directory, so create only its private parent in
advance.

On Windows PowerShell, use the same composer with `.exe` inputs:

```powershell
$RepoRoot = (Get-Location).Path
$SourceCommit = (git rev-parse HEAD).Trim()
$PackageParent = Join-Path $env:TEMP ("qiongli-local-package-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $PackageParent | Out-Null
$PackageRoot = Join-Path $PackageParent "artifact"
$TargetDir = Join-Path $RepoRoot "packages\qiongli-native\target\release"
$env:QIONGLI_NATIVE_SOURCE_COMMIT = $SourceCommit

pnpm desktop:build
cargo build `
  --manifest-path packages/qiongli-native/Cargo.toml `
  --package qiongli --release --bins --features custom-protocol --locked
cargo run `
  --manifest-path packages/qiongli-native/Cargo.toml `
  --package qiongli --example native_desktop_package --release --locked -- `
  --canonical "$TargetDir\qiongli.exe" `
  --launcher "$TargetDir\qiongli-desktop.exe" `
  --update-helper "$TargetDir\qiongli-update-helper.exe" `
  --output "$PackageRoot" `
  --source-commit "$SourceCommit"

Remove-Item Env:QIONGLI_NATIVE_SOURCE_COMMIT
Write-Output "Local package: $PackageRoot"
```

The package directory contains exactly:

- the target archive;
- `qiongli-desktop-package.manifest.json`;
- `qiongli-desktop-package.receipt.json` with
  `status: assembled-unpublished`.

The native output is target-specific:

| Host | Composer output | Local launch form |
|---|---|---|
| macOS | `Qiongli-<version>-macOS-<arch>.source.zip` | Convert the accepted arm64 source package to an ad-hoc test ZIP/DMG as shown below |
| Windows | `Qiongli-<version>-Windows-x64.zip` | Extract the whole `Qiongli` directory and run `Qiongli.exe` |
| Linux | `Qiongli-<version>-Linux-x64.zip` | Extract and run `Qiongli.AppDir/AppRun`; CI performs the additional pinned AppImage conversion |

### Create a macOS ad-hoc test DMG

On a supported macOS arm64 host, turn the verified source package into a local,
ad-hoc-signed test ZIP and DMG:

```bash
PACKAGE_SHA256="$(plutil -extract package_sha256 raw \
  "$PACKAGE_ROOT/qiongli-desktop-package.receipt.json")"
SIGNED_ROOT="$PACKAGE_PARENT/ad-hoc"

tooling/scripts/macos_native_sign_notarize.sh \
  --artifact-dir "$PACKAGE_ROOT" \
  --expected-source-commit "$SOURCE_COMMIT" \
  --expected-package-sha256 "$PACKAGE_SHA256" \
  --output-dir "$SIGNED_ROOT" \
  --test-only-ad-hoc
```

The resulting ZIP and DMG are non-publishing engineering evidence. Do not send
them to users or attach them to a release. `--community-alpha` and
`--production` belong to the controlled promotion/signing workflow, not normal
local development.

## Prepare the Alpha.2 Host Acceptance Fixture

Run the offline preflight without starting Codex, Claude Code, Claude Desktop,
or a model provider:

```bash
pnpm acceptance:host:preflight
```

The command validates the canonical fixed fixture, its source-fact and
source-anchor digests, required project-read tool, schema-2 candidate contract,
and checkpoint transition sequence. Its output is
`fixture-ready-manual-host-required` with `publication_allowed: false`; it is
not an accepted host receipt.

The native receipt validator is documented in
`tooling/release/acceptance/fixtures/README.md`. A later manual host session
must provide exact host, adapter, Plugin, binary, and protocol identities,
checkpoint hashes and counts, and zero direct-model/model-CLI verdicts. The
receipt cannot contain a prompt, candidate body, model response, conversation
ID, project ID/path, provider credential, or tool result.

## What a Local Package Does Not Prove

A local source package is self-contained and does not need the checkout, Rust,
Node.js, pnpm, Cargo, Python, or another Qiongli installation to start on the
target machine. The target operating system still supplies its native WebView
and window facilities.

However, self-contained does not mean release-authorised. Without embedded
release authority and product control, integration Apply operations, client
plugin installation, and automatic updates remain unavailable. The macOS-only
`native_packaged_product_acceptance` example exercises those paths with
ephemeral keys and an isolated home; it is an automated acceptance harness,
not a distributable App.

For distribution classes, target support, platform trust prompts, signing,
notarisation, and release acceptance, continue with
[Native Desktop Alpha Packages](/advanced/native-desktop-alpha).

## Common Failures

| Symptom | Cause and action |
|---|---|
| The browser page cannot load native state | Use `?fixture=source-read-only`, or run the full Tauri App |
| Svelte changes do not appear in `cargo run` | Run `pnpm desktop:build`, then restart the App |
| A release App opens as an empty frame | Build `qiongli` with `--features custom-protocol`; `pnpm desktop:macos` already does this |
| `frontendDist` or asset files are missing | Run `pnpm install --frozen-lockfile` and `pnpm desktop:build` from the repository root |
| Rust tries to use the wrong compiler | Run Cargo through the native manifest/workspace so the pinned `rust-toolchain.toml` is selected |
| `desktop-package-source-commit-unbound` | Set the same `QIONGLI_NATIVE_SOURCE_COMMIT` for the release build and composer run |
| `desktop-package-output-invalid` | Use a new absolute output path outside the checkout under an existing private parent |
| Apply or update is unavailable | Expected for a normal source build or source package without product authority |
| Codex or Claude Code is Missing/Unavailable | Run `qiongli doctor` and distinguish client discovery from plugin source, registration, activation, and MCP attachment; they are separate states |
