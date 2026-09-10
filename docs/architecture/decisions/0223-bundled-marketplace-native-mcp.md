# ADR 0223: Platform-specific Marketplace Plugins with bundled native MCP

- Status: Accepted
- Date: 2026-09-10
- Task: CLI-410
- Authority: maintainer approved platform-specific Plugins carrying the existing native executable.
- Extends ADR 0220's native distribution and ADR 0222's direct executable projection.
  Replaces the public npm-bridge packaging default; historical alpha.8 remains immutable.

Marketplace installation should require no additional Node, npm, Python, Cargo,
standalone Qiongli installation or first-start executable download. Reuse the
existing three qualified CLI binaries and embedded research resource pack. Each
Plugin's MCP manifest directly launches its bundled executable with
`mcp serve --profile lite --transport stdio`. Lite remains 14 tools; explicit
CLI/local Full configuration remains 32. Hosts continue to own models/accounts.

Build separate Codex and Claude archives for each target:

| Target | Plugin name |
|---|---|
| `aarch64-apple-darwin` | `qiongli-next-macos-arm64` |
| `x86_64-pc-windows-msvc` | `qiongli-next-windows-x64` |
| `x86_64-unknown-linux-gnu` | `qiongli-next-linux-x64` |

Archive names include Host, version and target. Release assembly emits
`marketplace-plugins.json`, mapping all six packages to their digests,
`plugins/<plugin-name>` paths and `<host>/<target>/v<version>` distribution refs.
Catalogs present an explicit platform choice. A portable generic manifest cannot
be assumed to select the user's OS/CPU. Unsupported platforms get no native
entry; Linux retains the glibc 2.35+ boundary. Larger plugin downloads are the
accepted cost of carrying the executable. Only the selected platform is needed.

`native_marketplace_plugins.py` owns resource projection and schema-2 receipts;
`native_cli_release.py` builds/checks each target; `native_release_assets.py`
binds executable bytes to the same target's CLI/npm artifacts and requires all
six checked Plugins and a reproducible index. Reuse the current CI matrix;
no separate runtime, downloader or model service is introduced.

Acceptance checks:

- Canonical resource bytes reproduce the native pack digest; the packaged binary
  matches the target CLI byte-for-byte and retains executable mode 0755.
- Empty-PATH isolated CLI and extracted-manifest MCP initialization, discovery
  and a non-network configuration read pass on each native target. No language
  runtime, shell bridge, user credentials or normal Host profile is supplied.
- Wrong OS/architecture, altered/missing binary, resource, profile, permissions,
  index entries or target smoke evidence fail verification. Missing any of the
  six packages, including omission of the entire set, fails new-packet checks.
- Old schema-1 npm-bridge receipts remain verifiable. New package generation
  cannot silently fall back to npm. CLI contracts and project approval/CAS stay
  unchanged; a valid archive does not grant managed installation authority.

Local code and diagnostic macOS repacking are not public release qualification.
Publish a fresh version only after native checks on all three systems, publish
its six distribution refs, then update external marketplace catalogs. Host
registration and live research acceptance remain separate. Roll back catalog
selection to the preceding immutable release; do not overwrite published assets
or remove existing user projects/configuration as part of a package rollback.
