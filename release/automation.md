# Release Automation Runbook

This repository standardizes release with three scripts:

- `scripts/release_preflight.sh`
- `scripts/release_postflight.sh`
- `scripts/release_automation.sh`

Draft release note generator:

- `scripts/generate_release_notes.sh`

## 1) Pre-release gates

```bash
./scripts/release_automation.sh pre --tag v0.1.0-beta.2
```

Runs validator + workflow unit tests + beta smoke checks, verifies the tag is not already used, and auto-generates `release/<tag>.md` draft if missing.
After checks pass, preflight auto-fills validation evidence lines in the release note.

Manual draft generation (optional):

```bash
./scripts/generate_release_notes.sh --tag v0.1.0-beta.3 --from-tag v0.1.0-beta.2
```

## 2) Publish tag

```bash
git tag -a v0.1.0-beta.2 -m "research-skills beta release"
git push origin v0.1.0-beta.2
```

## 3) Post-release checks

```bash
./scripts/release_automation.sh post --tag v0.1.0-beta.2
```

Runs local/remote consistency checks, attempts CI status verification, checks release notes + rollback docs, and generates:

- `release/acceptance/v0.1.0-beta.2-receipt.md`

## Optional flags

- `--skip-smoke`: skip smoke stage during preflight.
- `--skip-note-gen`: skip draft generation of `release/<tag>.md`.
- `--note-overwrite`: overwrite existing `release/<tag>.md` when generating draft.
- `--from-tag <tag>`: choose baseline tag used for draft highlights.
- `--skip-remote`: skip remote ref checks in postflight.
- `--skip-ci-status`: skip GitHub Actions status checks in postflight.
- `--create-release`: if `gh auth` is available, create GitHub release page from `release/<tag>.md`.

## GitHub workflow entrypoint

- Workflow: `.github/workflows/release-automation.yml`
- Trigger: `workflow_dispatch`
- Inputs: `mode`, `tag`, and optional skip/create flags
