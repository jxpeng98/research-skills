# ADR 0221: Cargo CLI publication

- Status: Accepted
- Date: 2026-09-09
- Task ID: CLI-410
- Authority: maintainer requested Cargo alongside npm/PyPI while proceeding with
  the CLI-to-Plugin/Host onboarding plan.
- Supersedes: ADR 0220's Cargo deferral only.

Cargo distributes native CLI source using the same canonical SemVer as the
executable and npm. Prereleases require an explicit version; Cargo has no `next`
dist-tag. `qiongli` and `ql` compile the same entrypoint. No App is required.

Reuse `native_registry_packages.py` to stage the existing nine workspace crates,
pin internal dependencies to the same version and include embedded content and
Zotero source assets. Cargo owns package normalization, verification and workspace
publication order. Do not introduce a second dependency resolver or copy runtime
code into a separate distribution implementation.

`publish-cargo.yml` qualifies the source archives on macOS ARM64, Windows x64 and
Linux x64. Official `cargo publish --dry-run` verifies packages without uploading;
the existing install checker also installs the checked archives with only internal
crate path patches and exercises both command names and Lite/Full MCP. This
offline closure check does not claim public registry resolution.

Only a published native GitHub Release may trigger upload, after source checks
and the existing exact-source native distribution CI gate. Publication uses
normal Cargo verification; a staged `--no-verify` archive alone is not evidence.
The `crates-io` environment supplies `CARGO_REGISTRY_TOKEN` for first publication.
Token absence blocks upload. Credentials must never appear in arguments, logs,
source or artifacts. Ownership and account configuration require actual evidence.
Subsequent registry installation is checked on all three native runners.

The staged manifests are the publication input; repository workspace publication
remains disabled to prevent accidental incomplete uploads. Published assets and
tags are immutable. A partial upload must be inspected before resuming only the
missing packages; do not replace existing crate versions or blindly retry uploads.

Installing from Cargo does not grant managed-product authority. Plugin registration
remains a separate increment using existing preview/approval/CAS and receipt owners.
No product acceptance, App release or announcement is implied by this channel.
