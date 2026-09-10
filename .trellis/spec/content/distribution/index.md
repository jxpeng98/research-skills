# Content And Distribution

Canonical academic content and public capability contracts live under
`content/`. Plugin directories, installed Skills, embedded packs, and release
payloads are generated outputs.

## Local Pattern

- Edit workflow/Skill source under `content/workflow/` and `content/skills/`.
- Edit Plugin metadata in `content/distribution/plugins.yaml`.
- Edit MCP public profiles and schemas under `content/mcp-contracts/`.
- Materialize payloads through `tooling/scripts/`; do not patch `dist/`,
  installed client directories, or generated plugin trees.
- A Skill may name a tool only when the selected MCP profile exposes it. If a
  runtime cannot provide the operation, the Skill must define a truthful safe
  fallback instead of assuming another product line is installed.

User-edited Plugin/Skill variants are managed project/user outputs. They do not
replace canonical content and must retain preview, receipt, and exact-removal
boundaries.

## Pre-Development Checklist

- Identify the canonical source and every generated consumer.
- Compare Skill tool names with the v2 registry and native tool registries.
- Decide whether the change affects embedded-pack or release inputs.

## Quality Check

- `python3 scripts/validate_capability_contract.py`
- Run the closest materialization or payload audit only when its inputs changed.
- Confirm generated outputs were not edited directly.

## CLI registry packages

`native_cli_release.py` owns standalone GitHub CLI archives and their generated,
target-specific README. Each archive carries the executable with embedded content,
README and LICENSE; the assembly owner retains all three platform archives and
their hashes in the release packet. Direct-download instructions must identify
the platform asset, checksum verification, extraction, executable name and PATH
option without requiring a package manager or confusing GitHub source archives
with runnable binaries. Published archives remain immutable.

Standalone binaries require no separately installed language runtime or package
manager. The Windows release owner sets `target-feature=+crt-static` only for
the explicit target and checks the final PE imports with the build machine's
LLVM tools, rejecting non-system DLLs. Extracted CLI/Lite/Full MCP smoke checks
run with empty PATH on every target. New native release packets require that
evidence and the Windows import list. LLVM is a build check, not a user runtime
dependency; supported OS libraries remain required (Linux x64: glibc 2.35+).
Public Windows alpha.8 still imports `VCRUNTIME140.dll`; download instructions
must disclose that exception until a newly qualified version replaces the link.

`release_version.py` owns SemVer/Git/npm and PEP 440 version projections;
prerelease npm publication uses `next`. `native_registry_packages.py` owns fixed
OS/CPU dispatch, executable bytes and platform wheels. The three-platform
`native-cli-distribution.yml` builds and installs on each target, assembles one
npm package and three wheels, and tests the combined package on each target.
`native_release_assets.py` refuses mixed source/version, missing targets and
changed bytes. Registry jobs reuse existing workflow filenames/environments and
require a successful exact-source distribution run. No App upload is part of this lane; product approval/CAS and managed trust remain unchanged.

Cargo uses the staged workspace and existing archive install checker (ADR 0221).
`publish-cargo.yml` runs native source verification on three systems, publishes
only on a qualified native GitHub Release, and checks public registry installs.
Cargo uses exact SemVer and both `qiongli`/`ql`; only the staged manifests permit
publication. Credentials and successful registry resolution are separate gates.
Staged manifests normalize CRLF input and write LF explicitly on every platform.
Cargo uploads run through GitHub Actions with the maintainer-configured
`CARGO_REGISTRY_TOKEN` repository or `crates-io` environment secret. Missing
credentials fail the publication job; local upload is not a fallback. Manual
`verify_public` dispatch checks an already published version without uploading.
The beta.1 local bootstrap remains historical evidence. Trusted Publishing is
deferred until the maintainer requests that migration.

User-approved local Plugin sources (ADR 0222) reuse the native Codex/Claude
bundle projectors, include the current executable and use a dedicated
`qiongli-cli-local` marketplace. They are derived exports, not canonical content
or signed products. Their `user-local-host-full-mcp` receipts contain no signed
grant digest; signed bundle APIs reject them. Source updates/removal require the
expected receipt inside the existing bundle transaction. Host registration,
cache refresh and live readiness remain separately observed actions.

Public Marketplace Plugins (ADR 0223) use `native_marketplace_plugins.py` and
the CLI's `export_marketplace_content` example. Shared research resources retain
the exact `marketplace-lite` pack bytes. Each Codex/Claude archive bundles its
qualified target's CLI and directly starts Lite MCP (14 tools) without Node,
npm, Python, a shell bridge or executable downloads. Full MCP remains available
through explicit CLI/local Plugin configuration; packaging does not expand tools.

The three targets use explicit `qiongli-next-macos-arm64`,
`qiongli-next-windows-x64` and `qiongli-next-linux-x64` identities. There is no
automatic OS selection in a generic Host manifest. `marketplace-plugins.json`
maps all six archives to Host, target, digest, plugin path and immutable
`<host>/<target>/v<version>` distribution ref. External marketplace catalogs must
consume that mapping and present the platform choices before public rollout.
Do not point an unqualified generic entry at one platform's binary.

Schema-2 archive receipts bind the target, executable and resource bytes. The
release owner verifies the executable against the corresponding CLI/npm bytes,
requires the same embedded pack across targets and binds target-native empty-PATH
CLI/MCP checks to each archive. The final combined-package matrix exercises the
extracted MCP manifest on each system. Preserve executable mode 0755 in tarballs
and Git distribution trees; research resources use 0644. Unknown platforms,
wrong executable formats, missing/changed bytes, altered profiles or permissions,
and incomplete/index-mismatched packets fail qualification.

Schema-1 alpha.8 npm-bridge archives remain verifiable with their historical
names and contents. New builds emit only schema 2. Public changes require a new
version and six qualified immutable distributions; existing alpha.8 refs and
assets remain unchanged. No signed grant, managed activation, Host registration,
private research access or model configuration change is implied by an archive.

Beta.2 permits exactly one npm `postinstall` script: the canonical terminal-only
installation review launcher. Asset verification checks its command and bytes;
arbitrary scripts remain rejected. It performs no download or cleanup, tolerates
cancellation, and skips non-terminal streams. `--ignore-scripts` remains supported.
Python wheels and Cargo retain standard installation; the installed native CLI
owns their subsequent interactive review.

An explicit `release-automation.yml` post dispatch at an immutable native
prerelease tag qualifies the same tag through Native CLI distribution, verifies
its packet, creates a GitHub prerelease, then dispatches the existing registry
workflows at that tag. Dispatch uploads are opt-in and tag-only; existing
credential environments and exact-source CI gates still apply. GITHUB_TOKEN
release events do not chain jobs, so publisher dispatch is explicit. The local
Agent may end after submission when requested; no public success is inferred.
