# Repository delivery checklists

Default loop: local feature branch → edit → affected **Focused** checks → review
and commit → local merge into `2.x` → next increment. No PR, remote CI wait,
post-merge suite or **Slice** ceremony is required. **Acceptance** belongs to a
named candidate. Reuse results while source, dependencies and inputs are unchanged.

The maintainer's local-development instruction covers branch creation, scoped
commits and local merges for the requested outcome; do not ask again per step.
Push, remote-rule changes, private research access and publication need their own
scope. A green check is evidence, not authorization.

## Local development and merge

- [ ] Start from local `2.x` with `git switch -c <branch> 2.x`; preserve unrelated work, using a worktree if needed.
- [ ] Review the diff for correctness, credentials, private data and unrelated changes; run the affected checks and `git diff --cached --check`, then commit the named paths with a Conventional Commit message.
- [ ] Inspect `git diff --check 2.x...HEAD`, record `git rev-parse HEAD`, and run `./scripts/check_2x_native_change_boundary.sh --base-ref 2.x`. This fast local guard protects frozen source; its CI-routing output starts no tests.
- [ ] Switch to local `2.x` and run `git merge --ff-only <branch>`. If `2.x` advanced, merge it into the feature branch, resolve conflicts and rerun only checks affected by the combined change before retrying.

Every head change invalidates stale exact-head CI/review evidence. Review the
final diff; reuse local checks when their inputs are unchanged. The current
independent-reviewer blocker concerns optional remote review, not local merge.
No duplicate report, receipt ceremony or per-stage human approval is needed.

## Optional remote synchronization

Fetch and inspect remote divergence only when synchronization is requested.
The existing GitHub `2.x` ruleset still requires a PR and remote checks; that
remote-only snapshot does not govern local integration. Do not silently open a
PR, change the ruleset or treat local merge authority as push authority.
Plain `--force` is forbidden. Exceptional rewrites of unprotected, unpublished
feature branches without accepted evidence require owner approval and reviewer
notice, using `--force-with-lease` only. Never rewrite `2.x`, release refs, tags
or accepted-evidence heads.

## Release checklist

Run only for a named candidate under explicit release scope.

- [ ] Freeze version, claims, non-claims, channels, rollback and the merged source; run `./scripts/release_ready.sh --version <version> --staging-dir <external-dir>`.
- [ ] For a requested remote candidate run, synchronize the source under separate authority, dispatch `gh workflow run native-ci.yml --ref 2.x` and verify its source/run identity. Bind packages, checksums, SBOM, provenance, signatures and receipts to the same bytes.
- [ ] Obtain the named publication decision for those bytes and channels; after publication, independently download and verify advertised assets.
- [ ] Obtain a distinct announcement decision and receipt for the verified public bytes and claims. Publication authorization does not authorize announcement.

Denied, expired, or revoked authorization blocks execution; request renewed scope
only for the blocked action. Emergency hotfix authorization is repository-only
and retains a named incident, minimum scope, current-head checks, rollback and
the protected remote PR path. This exceptional policy is unchanged.
Post-incident reconciliation is evidence, not retroactive authorization.
Tags and published assets are immutable. Changed release bindings require fresh
qualification and authorization.
