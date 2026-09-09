# ADR 0220: Three-platform CLI Registry Distribution

- Status: Accepted
- Date: 2026-09-09
- Task ID: CLI-410
- Owners: Qiongli maintainers
- Authority: maintainer requested Windows/Linux alongside macOS CI, standard
  versions, npm `next`, and PyPI Alpha/Beta publishing; Cargo is deferred.
- Supersedes: ADR 0219's macOS-only initial artifact scope and registry deferral.
  Managed-product trust and the historical alpha.6 release remain unchanged.

One canonical SemVer identity drives the binary, Git tag and npm version. PyPI
uses the PEP 440 projection (`2.0.0-alpha.7` → `2.0.0a7`,
`2.0.0-beta.1` → `2.0.0b1`). npm prereleases use `next`; no prerelease is promoted
to `latest`. Uppercase A/B input normalizes to lowercase PEP 440 output.

Native hosted runners qualify macOS ARM64, Windows x64 and Linux x64/glibc 2.35+.
They build the CLI, run CLI/MCP regression, extract and install their packages.
Linux wheels require auditwheel validation. Windows wheels carry `qiongli.exe`
and use the Windows virtual-environment Scripts directory. A final three-system
matrix installs the assembled release package; cross-compilation alone is not
runtime acceptance. This resumes Windows CLI package qualification; historical
App/managed installer and Parallels evidence are not retroactively accepted.

One npm `qiongli` package bundles three executables and a fixed OS/CPU selector.
Three PyPI wheels share the same version and use native platform selection.
No new npm package names, optional-package credential setup, postinstall downloads
or Cargo publication are needed. Existing `publish-npm.yml` / `publish-pypi.yml`
and their npm/pypi environments keep trusted-publisher identities. Native jobs
start on a published GitHub Release and verify its exact source, complete assets
and successful distribution CI. Legacy tag jobs retain the frozen 1.x package.

Acceptance requires all three target checks, combined-package installation,
version/channel negative tests, and post-publication registry installation.
Missing registry authentication is a publication blocker, not build acceptance.
Research approvals/CAS and managed-product refusal remain. Fixes ship under a
new version rather than replacing published assets. Announcement stays separate.
