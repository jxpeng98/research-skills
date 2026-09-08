# ADR 0219: CLI GitHub Release Distribution

- Status: Accepted
- Date: 2026-09-09
- Task ID: `CLI-410`
- Owners: Qiongli maintainers
- Decision authority: maintainer's explicit CLI publication request and correction
  that releases use GitHub Release, without App or Community Alpha promotion.
- Supersedes: ADR 0215's Community Alpha distribution prerequisite and ADR 0218's
  signed managed-install qualification prerequisite **for direct CLI Alpha
  downloads only**. Existing published App and managed-product contracts remain.

## Decision

The standalone CLI Alpha ships through an immutable Git tag and GitHub Release,
using the established GitHub distribution model from 1.x. It does not depend on
an App build, Linux Community Alpha promotion, launch grant, offline release key,
or registry login. macOS Apple Silicon is the first qualified binary target;
Windows remains partially complete/paused. Other targets require their own checks.

The local `release_ready.sh --cli-github` lane checks the clean merged source,
CLI-only Clippy/tests, embedded content, executable archive and local npm/wheel
installation. It records source identity, target, toolchain and artifact hashes.
The maintainer's current publication request authorizes this CLI release. Publish
the tag and exact qualified assets with `gh release`, then independently download
and verify them. This does not require pushing to or changing protected `2.x`.

The GitHub assets include a standalone executable archive and npm/Python binary
packages for installation from downloaded files. crates.io, npm and PyPI registry
publication remain separately tracked distribution work; attachment to a GitHub
Release does not claim an upload to any registry. No registry credential blocks
this GitHub-only lane. The frozen 1.x workflows remain unchanged.

## Boundaries and consequences

GitHub download authenticity and SHA-256 integrity do not grant signed managed
product authority. Existing managed Plugin/Skill activation, migration and update
commands retain their verification/refusal. Release notes must disclose those
limitations. Direct CLI/MCP and their existing approval/CAS owners are unchanged;
no runtime verification bypass, key replacement or private data access is added.
The release does not claim full 1.x replacement, App installation, notarization,
other target acceptance or completion of every CLI-410 program acceptance gate.

Upgrades use a new extracted directory or package-manager version. Rollback selects
the previous binary/path and preserves data; it does not reverse data migrations.
Published assets and tags remain immutable. A bad release is withdrawn or followed
by a new version, never silently replaced. Announcement remains separately scoped.

## Acceptance checks

1. The clean candidate passes CLI-only format/Clippy/tests and version checks.
2. The extracted executable and installed npm/wheel run CLI and real MCP smoke
   from isolated directories, without an App or signing key.
3. Public asset names, hashes and source tag match the qualified local packet;
   downloaded artifacts pass the same installation checks.
