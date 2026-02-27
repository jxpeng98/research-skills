# Beta Rollback Plan

This rollback plan applies to `v0.1.0-beta.2`.

## Trigger Conditions

- Validator or unit tests fail after release tag.
- Runtime hangs or major regression in `parallel` / `task-run`.
- Critical profile parsing regression.

## Immediate Mitigation

1. Pause external adoption announcement.
2. Stop creating new beta tags until root cause is isolated.
3. Keep issue log with failing command and reproduction input.

## Git Tag Rollback

If tag was pushed but should be withdrawn:

```bash
git tag -d v0.1.0-beta.2
git push origin :refs/tags/v0.1.0-beta.2
```

## Commit-Level Rollback

If release commit must be reverted on `main`:

```bash
git checkout main
git pull --ff-only
git revert <release_commit_sha>
git push origin main
```

## Recovery Validation

After rollback, run:

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest tests.test_orchestrator_workflows -v
./scripts/run_beta_smoke.sh
```

Rollback is complete only when all three commands pass.
