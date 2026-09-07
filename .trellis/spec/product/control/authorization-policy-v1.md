# Authorization, Receipt, and Repository Review V1

## 1. Scope / Trigger

Use this contract when changing authorization roles, sensitive actions,
non-transitive authority, redacted decision receipts, CODEOWNERS, or the
local `2.x` integration and the separate remote review ruleset. It is governance policy-as-code; it does not
grant runtime authority and a receipt is never a bearer credential.

## 2. Signatures

```bash
python tooling/scripts/validate_authorization_policy.py
python -m unittest tests.test_authorization_policy -v
```

Canonical artifacts:

- `tooling/architecture/authorization-policy-v1.json`;
- `tooling/architecture/authorization-receipt-v1.schema.json`;
- `tooling/architecture/repository-review-policy-v1.json`;
- `.github/CODEOWNERS`;
- `.github/delivery-checklists.md`;
- `.github/pull_request_template.md`.

## 3. Contracts

- The policy has exactly three independent planes: research, repository, and
  publication; authority never flows between them implicitly.
- The eight v1 roles and twelve v1 actions are closed, ordered inventories.
- Each action binds its executor, human authorizer rule, object scope, exact
  revision, plan or artifact digest, constraints, expiry, and evidence.
- Agent/CI may execute an already-authorized mechanical action and emit
  evidence, but cannot authorize, self-review, or widen scope.
- Preview, edit, stage, commit, push, PR, merge, CI success, publication, and
  public announcement remain explicitly non-transitive as encoded by the
  policy. Merge and publication receipts never authorize announcement.
- The Draft 2020-12 receipt is closed, finite, redacted, and immutable evidence
  of one decision. A consumer must still verify current scope, revision,
  digest, decision, constraints, and expiry before acting.
- The five authorization lifecycle paths are closed and ordered: denial,
  expiry, revocation, emergency hotfix, and post-incident reconciliation.
  Denied, expired, or revoked authority blocks execution and requires a new
  authorization; the original receipt is never mutated, revived, or replayed.
  Denial records a safe next action without exposing policy or classified data;
  revocation safely cancels/blocks in-flight work without publishing partial
  canonical state.
- Emergency hotfix authority is limited to the repository plane. It requires a
  named incident and human decision, minimum scope, a new finite approval,
  exact-head evidence, rollback planning, full audit trail, the existing
  protected PR path, and required checks; it grants no force-push,
  research-data, publication, announcement, or failed-check concealment right.
- A completed or aborted emergency requires reconciliation before another
  emergency is accepted. Reconciliation records exact impact, verification,
  rollback, follow-up ownership/review, and reviewer/blocker truth; it is
  evidence, never retroactive authorization.
- Credential compromise revokes and rotates the credential separately from
  authorization receipts, then audits actions from the exposure window.
- Receipt validation enforces each action's declared plan and artifact digest
  bindings; announcement requires both the exact content plan and verified
  public artifact digests.
- A later App, CLI, or MCP receipt surface must use ADR 0216's Rust-owned public
  schema and compatibility path; this JSON Schema is not a product wire owner.
- Repository review policy has exactly six ordered sensitive domains: security,
  schema, migration, release, research-Gate, and authorization. Every v1 path is
  literal, repository-rooted, present, symlink-free, and owned by `@jxpeng98`.
- The recorded remote ruleset `18800504` targets only `2.x`, has no bypass, blocks deletion and
  non-fast-forward changes, requires PR/thread resolution, and keeps the native
  cross-platform checks plus `Evaluation Truth V1` strict and current.
- Exact-head identity is one full lowercase commit SHA. A new commit, amend,
  rebase, merge, or history rewrite invalidates CI and review evidence;
  authorization, package, and release evidence remains reusable only while its
  complete recorded bindings remain current, and release evidence never
  transfers to a replacement candidate.
- `2.x`, `release/*`, tags, and accepted-evidence heads are never rewritten or
  force-pushed. An unprotected, unpublished feature branch without accepted
  evidence may be rewritten only exceptionally with owner approval, before
  review or after explicit reviewer notice, using `--force-with-lease`, and all
  receipts for replaced commits are invalidated. Prefer a follow-up commit.
- Review state is `blocked` while only one eligible human exists: required
  approvals remain zero and CODEOWNER approval remains disabled. `enforced`
  requires at least two distinct owners on every path, one approval, CODEOWNER
  review, exact-head non-stale evidence, and no blocker.
- Routine requested development uses local feature branches and local `2.x`
  merges under the maintainer's standing instruction. This is local Git history
  management; the protected remote `repository.merge` action and its PR,
  required-check and CODEOWNER evidence are unchanged. No local merge receipt
  may be presented as authorization or evidence for that remote action.
- Local commits and merges use affected checks and a reviewed diff without
  per-step confirmation. Push, protected remote merge, remote-rule changes and
  publication retain their separate authority. PR workflows and the recorded
  independent-review blocker concern optional remote collaboration only.
- Delivery documents select Focused/Slice/Acceptance proportionately and preserve
  evidence and authority boundaries. Markdown section counts, order, checkbox
  labels and per-step human confirmations are not validation requirements.
- The PR template links the canonical checklist and records outcome, tests and
  affected risks/rollback. GitHub owns the head and required-check state; no
  duplicate SHA ledger is required. New pushes replace stale remote evidence;
  unchanged local test results can be reused.

## 4. Validation & Error Matrix

- missing, duplicate, reordered, or unknown plane/role/action -> fail;
- unknown action references, Agent/CI authorizer, or weakened binding -> fail;
- missing announcement content plan, verified artifact digest, channel,
  publication receipt, public verification, approval, or negative transition
  -> fail;
- missing, duplicate, reordered, unknown, or positive authority transition ->
  fail;
- missing, reordered, unknown, broadened, or weakened authorization lifecycle
  path -> fail;
- non-canonical, missing, linked, or non-file repository evidence -> fail;
- changed Draft, open receipt, unknown field/value, missing digest/expiry, or
  unsafe evidence reference -> fail;
- invalid or non-redacted synthetic example -> fail;
- missing/reordered review domain or owned path, malformed CODEOWNERS line,
  unknown owner, missing file/directory, symlink, glob, or policy drift -> fail;
- removed branch rule/check, bypass actor, widened branch target, or false
  blocked/enforced review state -> fail.
- missing or weakened head event, evidence invalidation, protected ref,
  no-force rule, feature eligibility, owner/reviewer notice, lease-only mode, or
  replaced-receipt invalidation -> fail.
- missing required evidence command, authorization warning or canonical
  checklist link -> fail; layout-only changes remain valid.

## 5. Good / Base / Bad Cases

- Good: a bounded decision names one permitted action, exact object/revision,
  appropriate actor and authorizer roles, digest, constraints, expiry, and
  redacted evidence.
- Lifecycle good: an invalid receipt blocks work; a repository incident obtains
  a new finite approval and preserves protected checks; its reconciliation is
  completed before another emergency.
- Base: policy and schema remain unchanged and Evaluation Truth validates them
  without creating any receipt or changing product state.
- Bad: green CI, a prior edit/commit/merge, or an old receipt is treated as
  authorization for a later action or changed input.
- Repository good: a scoped local merge has affected checks and a reviewed diff;
  CODEOWNERS and the separate remote ruleset snapshot remain truthful.
- Repository base: one maintainer receives path ownership routing, while the
  ledger truthfully retains the independent-reviewer blocker.
- Repository bad: CI/Agent approval, self-review, bypass, or an impossible
  one-person CODEOWNER requirement is presented as independent authorization.
- History good: an ordinary follow-up commit creates a new exact head and stale
  CI/review evidence is replaced; history bad: a protected or accepted-evidence
  head is rewritten, or a feature rewrite uses plain `--force`.

## 6. Tests Required

- Mutate each closed inventory and non-transitive rule set; assert fail-closed
  validation.
- Mutate lifecycle order, identity, emergency scope, protected-flow effect,
  required evidence, and reconciliation non-authorization; assert rejection.
- Mutate announcement authorizer, bindings, evidence, receipt digests, delivery
  wording, and schema action enum; assert fail-closed validation.
- Mutate role/action references and Agent/CI authorization; assert rejection.
- Across every v1 action, reject Agent/CI or `ci-green` authorizers, Agent/CI
  self-authorized receipts, and removal of any declared binding. Remove or
  positively reverse each `ci-green` rule for merge, publication, and
  announcement; assert rejection.
- Weaken the closed schema, digest, expiry, bounds, or enums; assert rejection.
- Mutate the example with missing/unknown fields, stale expiry, absent digests,
  unknown action, or absolute evidence path; assert rejection.
- Mutate domains, paths, owners, branch rules, required checks, bypass actors,
  CODEOWNERS text, and review-state coherence; assert rejection.
- Mutate exact-head events/evidence reuse, protected refs/force policy, feature
  rewrite eligibility, authority/notice, lease-only mode, or receipt
  invalidation; assert rejection.
- Remove required commands/warnings, evidence fields or the canonical link;
  assert rejection. Reformat headings/checkboxes without altering these
  boundaries; assert acceptance.
- Run the validator and focused tests in Evaluation Truth.

## 7. Wrong vs Correct

Wrong: infer merge, publication, or announcement authority from a successful
check, reuse publication authority for announcement, replay a receipt against
changed input, or enable self-blocking review and call it independent approval.

Wrong: revive a denied, expired, or revoked receipt, use an incident to bypass
required checks or leave the repository plane, or treat reconciliation as
permission after the fact.

Correct: verify existing scoped authority for each named action, including
a distinct announcement receipt after public verification, retain a redacted
receipt, verify every binding, and route sensitive paths to CODEOWNERS. Keep
independent-review status blocked until another eligible human exists; this
remote status does not prevent a scoped local merge.
For ordinary source changes, add a follow-up commit and replace current-head
evidence instead of rewriting history.
