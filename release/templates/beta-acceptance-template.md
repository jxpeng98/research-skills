# Beta Acceptance Receipt — {{TAG}}

- Date: {{DATE}}
- Release Tag: {{TAG}}
- Commit: {{COMMIT}}
- CI Status: {{CI_STATUS}}

## Publish Preconditions (Preflight)

- [ ] `python3 scripts/validate_research_standard.py --strict`
- [ ] `python3 -m unittest tests.test_orchestrator_workflows -v`
- [ ] `./scripts/run_beta_smoke.sh`

## Publish Actions

- [ ] `git tag -a {{TAG}} -m "research-skills beta release"`
- [ ] `git push origin {{TAG}}`

## Post-Release Verification

- [ ] Remote branch/tag consistency verified.
- [ ] GitHub Actions CI status verified for release commit.
- [ ] GitHub Release page exists and notes are attached.
- [ ] Rollback path validated (`release/rollback.md`).

## Collaboration Validation (Codex / Claude / Gemini)

- [ ] `parallel` triad path works or degrades safely to dual/single.
- [ ] `task-run` stage routing works with capability map fallback.
- [ ] Profile overrides (`--draft-profile` / `--review-profile` / `--triad-profile`) are effective.

## Sign-off

- Owner:
- Reviewer:
- Notes:
