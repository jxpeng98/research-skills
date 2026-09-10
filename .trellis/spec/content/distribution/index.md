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

User-approved local Plugin sources (ADR 0222) reuse the native Codex/Claude
bundle projectors, include the current executable and use a dedicated
`qiongli-cli-local` marketplace. They are derived exports, not canonical content
or signed products. Their `user-local-host-full-mcp` receipts contain no signed
grant digest; signed bundle APIs reject them. Source updates/removal require the
expected receipt inside the existing bundle transaction. Host registration,
cache refresh and live readiness remain separately observed actions.

Public `qiongli-next` marketplace archives use `native_marketplace_plugins.py`
and the CLI's `export_marketplace_content` example. The existing resource-pack
loader/projector exports `marketplace-lite`; shared Skill resources retain their
exact bytes. Only platform manifests and a Node bridge are added. The bridge
pins the same npm SemVer and serves native Lite MCP (14 tools); Full MCP remains
available through explicit CLI/local Plugin configuration. No signed grant,
managed activation or Host registration is implied by a public archive.

The CLI release packet may carry the Codex/Claude archive pair. Verification
requires their version, release source, resource hashes and pack hash to agree
with all three native CLI observations. Publish immutable `codex/v<version>` and
`claude/v<version>` distribution refs before advancing marketplace catalogs;
catalogs reference `plugins/qiongli-next`. The archive names preserve the existing
Skillsplace release-sync contract. Node 18+ and access to npm on first MCP start
are explicit dependencies; user caches and model configuration are not edited.
