# Release Automation Runbook

This repository standardizes release with four scripts:

- `scripts/release_ready.sh`
- `scripts/release_preflight.sh`
- `scripts/release_postflight.sh`
- `scripts/release_automation.sh`

Prerelease note draft generator:

- `scripts/generate_release_notes.sh`

## 1) One-command publish

If you want the whole path chained together, use:

```bash
./scripts/release_automation.sh publish --version 0.1.0 --from-tag v0.1.0-beta.6
```

This mode runs:

- `scripts/release_ready.sh`
- release-prep commit creation
- annotated tag creation
- push of the primary branch + tag
- waiting for `CI` and `Install Check`
- `scripts/release_postflight.sh --create-release`

Stable tags become normal GitHub Releases. Beta tags become GitHub prereleases, so stable and beta releases can coexist without breaking `releases/latest`.

## 2) Prepare a publish-ready local state

```bash
./scripts/release_ready.sh --version 0.1.0 --from-tag v0.1.0-beta.6
```

This is the recommended local entrypoint. It chains:

- `scripts/bump-version.sh`
- `scripts/release_automation.sh pre`
- `scripts/pypi_preflight.sh`

When it succeeds, the repository is in a publish-ready state with synchronized version files, validated release docs, and built package artifacts.

## 3) Manual pre-release gates (optional)

```bash
./scripts/release_automation.sh pre --tag v0.1.0 --from-tag v0.1.0-beta.6
```

Runs validator + repository unit tests + release-tier smoke checks, verifies the tag is not already used, and then:

- beta / prerelease tags: auto-generate `release/<tag>.md` draft if missing
- stable tags: verify the matching version section already exists in `CHANGELOG.md`

After checks pass, preflight auto-fills validation evidence lines in prerelease notes.

Manual prerelease draft generation (optional):

```bash
./scripts/generate_release_notes.sh --tag v0.1.0 --from-tag v0.1.0-beta.6
```

The draft generator remains available, but the default policy is now:

- stable tags publish from `CHANGELOG.md`
- prerelease tags publish from `release/<tag>.md`

## 4) Publish tag

```bash
git add pyproject.toml research_skills/__init__.py research-paper-workflow/VERSION skills/registry.yaml skills CHANGELOG.md
git commit -m "chore: prepare release 0.1.0"
git tag -a v0.1.0 -m "research-skills release"
git push origin main --tags
```

## 5) Post-release checks

```bash
./scripts/release_automation.sh post --tag v0.1.0 --create-release
```

Runs local/remote consistency checks, attempts CI status verification, checks release docs + rollback docs, and generates:

- `release/acceptance/v0.1.0-receipt.md`

## Optional flags

- `--version <version>`: required by `publish`, accepts stable (`0.2.0`) and beta (`0.2.0b1`) forms.
- `--skip-smoke`: skip smoke stage during preflight.
- `--maintainer-smoke`: upgrade preflight smoke from the default release tier to the maintainer tier (`parallel` + `task-run` profile checks).
- `--skip-note-gen`: skip prerelease draft generation of `release/<tag>.md`.
- `--note-overwrite`: overwrite existing `release/<tag>.md` when generating prerelease draft.
- `--from-tag <tag>`: choose baseline tag used for prerelease draft highlights.
- `--skip-bump`: start `release_ready.sh` from preflight/package checks only.
- `--allow-dirty`: let `release_ready.sh` continue on a dirty worktree.
- `--commit-message <msg>`: override the release-prep commit message used by `publish`.
- `--push-remote <name>` / `--push-branch <name>`: override the remote/branch used by `publish`.
- `--wait-ci`: wait for required workflows to succeed before release creation.
- `--ci-timeout-seconds <n>` / `--ci-poll-interval-seconds <n>`: control publish wait behavior.
- `--skip-remote`: skip remote ref checks in postflight.
- `--skip-ci-status`: skip GitHub Actions status checks in postflight.
- `--create-release`: if `gh auth` is available, create GitHub release page from the prerelease note file or the matching `CHANGELOG.md` section for stable tags.

## GitHub workflow entrypoint

- Workflow: `.github/workflows/release-automation.yml`
- Trigger: `workflow_dispatch`
- Inputs: `mode`, plus `version` for `publish` or `tag` for `pre` / `post`
