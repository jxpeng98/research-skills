# Release Automation Runbook

This repository standardizes release with four scripts:

- `scripts/release_ready.sh`
- `scripts/release_preflight.sh`
- `scripts/release_postflight.sh`
- `scripts/release_automation.sh`

Draft release note generator:

- `scripts/generate_release_notes.sh`

## 1) Prepare a publish-ready local state

```bash
./scripts/release_ready.sh --version 0.1.0 --from-tag v0.1.0-beta.6
```

This is the recommended local entrypoint. It chains:

- `scripts/bump-version.sh`
- `scripts/release_automation.sh pre`
- `scripts/pypi_preflight.sh`

When it succeeds, the repository is in a publish-ready state with synchronized version files, validated release notes, and built package artifacts.

## 2) Manual pre-release gates (optional)

```bash
./scripts/release_automation.sh pre --tag v0.1.0 --from-tag v0.1.0-beta.6
```

Runs validator + repository unit tests + release smoke checks, verifies the tag is not already used, and auto-generates `release/<tag>.md` draft if missing.
After checks pass, preflight auto-fills validation evidence lines in the release note.

Manual draft generation (optional):

```bash
./scripts/generate_release_notes.sh --tag v0.1.0 --from-tag v0.1.0-beta.6
```

The draft generator supports both stable tags such as `v0.1.0` and prerelease tags such as `v0.1.1-beta.1`. Stage label is inferred from the tag unless you pass `--stage`.

## 3) Publish tag

```bash
git add pyproject.toml research_skills/__init__.py research-paper-workflow/VERSION skills/registry.yaml skills release/v0.1.0.md
git commit -m "chore: prepare release 0.1.0"
git tag -a v0.1.0 -m "research-skills release"
git push origin main --tags
```

## 4) Post-release checks

```bash
./scripts/release_automation.sh post --tag v0.1.0 --create-release
```

Runs local/remote consistency checks, attempts CI status verification, checks release notes + rollback docs, and generates:

- `release/acceptance/v0.1.0-receipt.md`

## Optional flags

- `--skip-smoke`: skip smoke stage during preflight.
- `--skip-note-gen`: skip draft generation of `release/<tag>.md`.
- `--note-overwrite`: overwrite existing `release/<tag>.md` when generating draft.
- `--from-tag <tag>`: choose baseline tag used for draft highlights.
- `--skip-bump`: start `release_ready.sh` from preflight/package checks only.
- `--allow-dirty`: let `release_ready.sh` continue on a dirty worktree.
- `--skip-remote`: skip remote ref checks in postflight.
- `--skip-ci-status`: skip GitHub Actions status checks in postflight.
- `--create-release`: if `gh auth` is available, create GitHub release page from `release/<tag>.md`.

## GitHub workflow entrypoint

- Workflow: `.github/workflows/release-automation.yml`
- Trigger: `workflow_dispatch`
- Inputs: `mode`, `tag`, and optional skip/create flags
