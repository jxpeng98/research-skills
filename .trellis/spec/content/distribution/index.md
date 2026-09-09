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
require a successful exact-source distribution run. No App or Cargo upload is
part of this lane; product approval/CAS and managed trust remain unchanged.
