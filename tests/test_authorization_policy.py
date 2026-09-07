from __future__ import annotations

import copy
import tempfile
import unittest
from pathlib import Path

from tooling.scripts.validate_authorization_policy import (
    AuthorizationPolicyError,
    DEFAULT_CODEOWNERS,
    DEFAULT_DELIVERY_CHECKLISTS,
    DEFAULT_POLICY,
    DEFAULT_PR_TEMPLATE,
    DEFAULT_REVIEW_POLICY,
    DEFAULT_SCHEMA,
    EXPECTED_ACTIONS,
    EXPECTED_PLANES,
    EXPECTED_REVIEW_DOMAINS,
    EXPECTED_ROLES,
    REPO_ROOT,
    load_document,
    resolve_codeowner_pattern,
    validate_delivery_documents,
    validate_policy,
    validate_receipt,
    validate_review_policy,
)


class AuthorizationPolicyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.policy = load_document(DEFAULT_POLICY)
        self.schema = load_document(DEFAULT_SCHEMA)
        self.review_policy = load_document(DEFAULT_REVIEW_POLICY)
        self.codeowners = DEFAULT_CODEOWNERS.read_text(encoding="utf-8")
        self.delivery_checklists = DEFAULT_DELIVERY_CHECKLISTS.read_text(
            encoding="utf-8"
        )
        self.pr_template = DEFAULT_PR_TEMPLATE.read_text(encoding="utf-8")
        self.receipt = copy.deepcopy(self.schema["examples"][0])

    def assert_policy_error(
        self,
        policy: dict[str, object],
        message: str,
        schema: dict[str, object] | None = None,
    ) -> None:
        errors = validate_policy(
            REPO_ROOT,
            policy,
            schema if schema is not None else self.schema,
            self.review_policy,
            self.codeowners,
        )
        self.assertTrue(any(message in error for error in errors), errors)

    def assert_review_error(
        self,
        review_policy: dict[str, object],
        message: str,
        codeowners: str | None = None,
    ) -> None:
        errors = validate_review_policy(
            REPO_ROOT,
            review_policy,
            self.codeowners if codeowners is None else codeowners,
        )
        self.assertTrue(any(message in error for error in errors), errors)

    def assert_receipt_error(
        self, receipt: dict[str, object], message: str
    ) -> None:
        errors = validate_receipt(self.policy, receipt)
        self.assertTrue(any(message in error for error in errors), errors)

    def test_repository_policy_schema_and_example_are_valid(self) -> None:
        self.assertEqual(
            validate_policy(
                REPO_ROOT,
                self.policy,
                self.schema,
                self.review_policy,
                self.codeowners,
            ),
            [],
        )
        self.assertEqual(validate_receipt(self.policy, self.receipt), [])
        self.assertEqual(
            validate_review_policy(
                REPO_ROOT, self.review_policy, self.codeowners
            ),
            [],
        )
        self.assertEqual(
            validate_delivery_documents(
                self.delivery_checklists,
                self.pr_template,
            ),
            [],
        )
        self.assertEqual(
            tuple(plane["id"] for plane in self.policy["planes"]),
            EXPECTED_PLANES,
        )
        self.assertEqual(
            tuple(role["id"] for role in self.policy["roles"]), EXPECTED_ROLES
        )
        self.assertEqual(
            tuple(action["id"] for action in self.policy["actions"]),
            EXPECTED_ACTIONS,
        )
        self.assertEqual(
            tuple(self.review_policy["codeowners"]["domains"]),
            EXPECTED_REVIEW_DOMAINS,
        )

    def test_local_workflow_preserves_protected_remote_merge_authority(self) -> None:
        merge = next(action for action in self.policy["actions"] if action["id"] == "repository.merge")
        self.assertEqual(merge["default_rule"], "protected-pr-only")
        self.assertEqual(merge["authorizer_roles"], ["maintainer", "codeowner-specialist-reviewer"])
        self.assertIn("reviewer-codeowner-approval", merge["required_evidence"])
        self.assertIn("git merge --ff-only", self.delivery_checklists)
        self.assertIn("remote-only", self.delivery_checklists)
        self.assertNotIn("gh pr checks --required --watch", self.delivery_checklists)

    def test_delivery_layout_is_not_an_approval_gate(self) -> None:
        checklist = self.delivery_checklists.replace("## ", "### ").replace("- [ ]", "-")
        template = self.pr_template.replace("## ", "### ")
        self.assertEqual(validate_delivery_documents(checklist, template), [])

    def test_delivery_checklists_and_pr_template_fail_closed(self) -> None:
        cases = (
            (
                self.delivery_checklists.replace(
                    "git diff --cached --check", "git diff --cached", 1
                ),
                self.pr_template,
                "missing required marker",
            ),
            (
                self.delivery_checklists,
                self.pr_template.replace(
                    ".github/delivery-checklists.md",
                    "delivery-checklists",
                    1,
                ),
                "missing required marker",
            ),
            (
                self.delivery_checklists.replace(
                    "Plain `--force` is forbidden.",
                    "Plain `--force` is allowed.",
                    1,
                ),
                self.pr_template,
                "missing required marker",
            ),
            (
                self.delivery_checklists,
                self.pr_template.replace(
                    "Every head change invalidates stale exact-head",
                    "Every push",
                    1,
                ),
                "missing required marker",
            ),
            (
                self.delivery_checklists.replace(
                    "Denied, expired, or revoked authorization blocks execution",
                    "Invalid authorization may block execution",
                    1,
                ),
                self.pr_template,
                "missing required marker",
            ),
            (
                self.delivery_checklists.replace(
                    "Emergency hotfix authorization is repository-only",
                    "Emergency hotfix authorization may cross planes",
                    1,
                ),
                self.pr_template,
                "missing required marker",
            ),
            (
                self.delivery_checklists.replace(
                    "Post-incident reconciliation is evidence, not retroactive "
                    "authorization.",
                    "Post-incident reconciliation authorizes prior work.",
                    1,
                ),
                self.pr_template,
                "missing required marker",
            ),
        )
        for checklist, pr_template, message in cases:
            with self.subTest(message=message):
                errors = validate_delivery_documents(checklist, pr_template)
                self.assertTrue(any(message in error for error in errors), errors)

        policy = copy.deepcopy(self.policy)
        policy["evidence"].remove(".github/delivery-checklists.md")
        self.assert_policy_error(policy, "must name the delivery checklist")

        review_policy = copy.deepcopy(self.review_policy)
        review_policy["evidence"].remove(".github/pull_request_template.md")
        self.assert_review_error(
            review_policy,
            "must name the delivery checklist and PR template",
        )

    def test_review_policy_rejects_missing_or_weakened_controls(self) -> None:
        mutations = (
            (
                lambda policy: policy.update(unexpected=True),
                "contain exactly",
            ),
            (
                lambda policy: policy["codeowners"]["domains"].pop("authorization"),
                "six sensitive domains",
            ),
            (
                lambda policy: policy["codeowners"]["domains"]["security"].pop(),
                "exact v1 paths",
            ),
            (
                lambda policy: policy["ruleset"]["rules"].remove(
                    "non_fast_forward"
                ),
                "exact protected rules",
            ),
            (
                lambda policy: policy["ruleset"]["required_status_checks"].pop(),
                "exact native and Evaluation Truth contexts",
            ),
            (
                lambda policy: policy["ruleset"]["bypass_actors"].append(
                    {"actor_type": "RepositoryRole", "actor_id": 5}
                ),
                "must not define bypass actors",
            ),
            (
                lambda policy: policy["ruleset"]["pull_request"].update(
                    required_approving_review_count=1
                ),
                "blocked review enforcement",
            ),
            (
                lambda policy: policy["review_enforcement"].update(
                    state="enforced", blocker_reason_code="", blocker=""
                ),
                "requires at least one approval",
            ),
            (
                lambda policy: policy["codeowners"]["domains"]["security"].__setitem__(
                    0, "../outside"
                ),
                "rooted at the repository",
            ),
            (
                lambda policy: policy["codeowners"].update(
                    owners=["@another-maintainer"]
                ),
                "must retain @jxpeng98",
            ),
        )
        for mutate, message in mutations:
            with self.subTest(message=message):
                review_policy = copy.deepcopy(self.review_policy)
                mutate(review_policy)
                self.assert_review_error(review_policy, message)

    def test_history_policy_rejects_weakened_invalidation_or_rewrite_rules(self) -> None:
        mutations = (
            lambda policy: policy.pop("history_policy"),
            lambda policy: policy["history_policy"]["exact_head"][
                "head_change_events"
            ].pop(),
            lambda policy: policy["history_policy"]["exact_head"][
                "evidence_rules"
            ][0].update(reuse="previous-head-allowed"),
            lambda policy: policy["history_policy"]["protected_refs"][
                "patterns"
            ].pop(),
            lambda policy: policy["history_policy"]["protected_refs"].update(
                force_push="allowed"
            ),
            lambda policy: policy["history_policy"]["feature_branch_rewrite"].update(
                eligibility="any-feature-branch"
            ),
            lambda policy: policy["history_policy"]["feature_branch_rewrite"].update(
                authority="none"
            ),
            lambda policy: policy["history_policy"]["feature_branch_rewrite"].update(
                review_notice="none"
            ),
            lambda policy: policy["history_policy"]["feature_branch_rewrite"].update(
                push_mode="force"
            ),
            lambda policy: policy["history_policy"]["feature_branch_rewrite"].update(
                replaced_receipts="retain"
            ),
        )
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                review_policy = copy.deepcopy(self.review_policy)
                mutate(review_policy)
                self.assert_review_error(review_policy, "exact v1 invalidation")

    def test_codeowner_patterns_are_literal_contained_and_kind_safe(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            owned_file = root / "owned.txt"
            owned_file.write_text("owned\n", encoding="utf-8")
            owned_directory = root / "owned-directory"
            owned_directory.mkdir()
            link = root / "linked-directory"
            link.symlink_to(owned_directory, target_is_directory=True)

            self.assertEqual(
                resolve_codeowner_pattern(root, "/owned.txt"),
                owned_file.resolve(),
            )
            self.assertEqual(
                resolve_codeowner_pattern(root, "/owned-directory/"),
                owned_directory.resolve(),
            )
            cases = (
                ("/*.md", "literal path"),
                ("/missing", "resolve inside"),
                ("/owned.txt/", "directory pattern"),
                ("/owned-directory", "file pattern"),
                ("/linked-directory/", "symbolic link"),
            )
            for pattern, message in cases:
                with self.subTest(pattern=pattern):
                    with self.assertRaisesRegex(AuthorizationPolicyError, message):
                        resolve_codeowner_pattern(root, pattern)

    def test_codeowners_must_exactly_match_review_policy(self) -> None:
        cases = (
            self.codeowners.replace("/.github/ @jxpeng98\n", "", 1),
            self.codeowners + "/tooling/release/\n",
            self.codeowners.replace(
                "/tooling/release/ @jxpeng98",
                "/tooling/release/ @jxpeng98\n/tooling/release/ @jxpeng98",
                1,
            ),
        )
        for codeowners in cases:
            with self.subTest(codeowners=codeowners[-80:]):
                self.assert_review_error(
                    self.review_policy,
                    "CODEOWNERS",
                    codeowners,
                )

    def test_evaluation_truth_runs_authorization_checks(self) -> None:
        workflow = (REPO_ROOT / ".github/workflows/evaluation-truth.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            "python tooling/scripts/validate_authorization_policy.py", workflow
        )
        self.assertIn("tests.test_authorization_policy", workflow)

    def test_closed_inventories_reject_missing_duplicate_and_reordered_ids(self) -> None:
        for inventory in ("planes", "roles", "actions"):
            for mutation in ("missing", "duplicate", "reordered"):
                with self.subTest(inventory=inventory, mutation=mutation):
                    policy = copy.deepcopy(self.policy)
                    values = policy[inventory]
                    if mutation == "missing":
                        values.pop()
                    elif mutation == "duplicate":
                        values[-1] = copy.deepcopy(values[0])
                    else:
                        values[0], values[1] = values[1], values[0]
                    self.assert_policy_error(policy, "closed inventory")

    def test_authorization_lifecycle_is_closed_and_fail_closed(self) -> None:
        mutations = (
            lambda policy: policy.pop("authorization_lifecycle"),
            lambda policy: policy["authorization_lifecycle"].pop(),
            lambda policy: policy["authorization_lifecycle"].reverse(),
            lambda policy: policy["authorization_lifecycle"][0].update(
                id="unknown-path"
            ),
            lambda policy: policy["authorization_lifecycle"][3].update(
                scope="all-actions"
            ),
            lambda policy: policy["authorization_lifecycle"][3].update(
                effect="bypass-protected-pr"
            ),
            lambda policy: policy["authorization_lifecycle"][3][
                "required_evidence"
            ].remove("required-checks"),
            lambda policy: policy["authorization_lifecycle"][4].update(
                forbidden_transition="retroactive-authorization-allowed"
            ),
        )
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                policy = copy.deepcopy(self.policy)
                mutate(policy)
                self.assert_policy_error(policy, "closed fail-closed paths")

    def test_role_and_action_references_fail_closed(self) -> None:
        mutations = (
            (
                lambda policy: policy["actions"][0]["executor_roles"].append(
                    "unknown-role"
                ),
                "unknown role",
            ),
            (
                lambda policy: policy["actions"][0].update(plane="repository"),
                "action namespace",
            ),
            (
                lambda policy: policy["actions"][0].update(
                    default_rule="allow-by-default"
                ),
                "retain its v1 rule",
            ),
            (
                lambda policy: policy["actions"][0].update(unexpected=True),
                "contain exactly",
            ),
        )
        for mutate, message in mutations:
            with self.subTest(message=message):
                policy = copy.deepcopy(self.policy)
                mutate(policy)
                self.assert_policy_error(policy, message)

    def test_agent_ci_cannot_authorize_actions_or_widen_bindings(self) -> None:
        for action_index, action in enumerate(self.policy["actions"]):
            action_id = action["id"]
            with self.subTest(action=action_id, baseline="agent-executor"):
                self.assertIn("agent-ci-principal", action["executor_roles"])

            for authorizer, message in (
                (
                    "agent-ci-principal",
                    "Agent/CI principal cannot be an authorizer",
                ),
                ("ci-green", "exact authorizer rule"),
            ):
                with self.subTest(action=action_id, authorizer=authorizer):
                    policy = copy.deepcopy(self.policy)
                    policy["actions"][action_index]["authorizer_roles"].append(
                        authorizer
                    )
                    self.assert_policy_error(policy, message)

            receipt = copy.deepcopy(self.receipt)
            bindings = action["required_bindings"]
            receipt.update(
                action=action_id,
                actor_role="agent-ci-principal",
                authorizer_role=action["authorizer_roles"][0],
                plan_digest_sha256="1" * 64 if "plan-digest" in bindings else None,
                artifact_digests_sha256=(
                    ["2" * 64] if "artifact-digests" in bindings else []
                ),
            )
            with self.subTest(action=action_id, receipt="human-authorized"):
                self.assertEqual(validate_receipt(self.policy, receipt), [])
            receipt["authorizer_role"] = "agent-ci-principal"
            with self.subTest(action=action_id, receipt="self-authorized"):
                self.assert_receipt_error(receipt, "unknown or Agent/CI")

            for binding in bindings:
                with self.subTest(action=action_id, removed_binding=binding):
                    policy = copy.deepcopy(self.policy)
                    policy["actions"][action_index]["required_bindings"].remove(
                        binding
                    )
                    self.assertNotEqual(
                        validate_policy(
                            REPO_ROOT,
                            policy,
                            self.schema,
                            self.review_policy,
                            self.codeowners,
                        ),
                        [],
                    )

    def test_announcement_authorization_is_separate_and_bound(self) -> None:
        action_id = "publication.announce-release"
        action_index = EXPECTED_ACTIONS.index(action_id)
        policy_mutations = (
            (
                lambda policy: policy["actions"].pop(action_index),
                "closed inventory",
            ),
            (
                lambda policy: policy["actions"][action_index][
                    "authorizer_roles"
                ].__setitem__(0, "maintainer"),
                "exact authorizer rule",
            ),
            (
                lambda policy: policy["actions"][action_index][
                    "executor_roles"
                ].append("contributor-agent-operator"),
                "exact executor roles",
            ),
            (
                lambda policy: policy["actions"][action_index][
                    "required_bindings"
                ].remove("plan-digest"),
                "announcement plan, artifacts, and channels",
            ),
            (
                lambda policy: policy["actions"][action_index][
                    "required_bindings"
                ].remove("artifact-digests"),
                "announcement plan, artifacts, and channels",
            ),
            (
                lambda policy: policy["actions"][action_index][
                    "required_bindings"
                ].remove("channels"),
                "announcement plan, artifacts, and channels",
            ),
            (
                lambda policy: policy["actions"][action_index][
                    "required_evidence"
                ].remove("public-verification"),
                "missing required evidence",
            ),
            (
                lambda policy: policy["non_transitive_rules"].pop(),
                "closed negative transitions",
            ),
        )
        for mutate, message in policy_mutations:
            with self.subTest(policy=message):
                policy = copy.deepcopy(self.policy)
                mutate(policy)
                self.assert_policy_error(policy, message)

        schema = copy.deepcopy(self.schema)
        schema["properties"]["action"]["enum"].remove(action_id)
        self.assertNotEqual(validate_policy(REPO_ROOT, self.policy, schema), [])

        receipt = copy.deepcopy(self.receipt)
        receipt.update(
            action=action_id,
            actor_role="maintainer",
            authorizer_role="release-approver",
            artifact_digests_sha256=["2" * 64],
            data_classification="public",
            constraints=["verified-publication-only"],
            reason_code="announcement-approved",
            evidence_refs=[
                "publication-receipt",
                "public-verification",
                "announcement-content-digest",
                "announcement-approval",
            ],
        )
        self.assertEqual(validate_receipt(self.policy, receipt), [])
        for field, value, message in (
            ("plan_digest_sha256", None, "requires plan_digest_sha256"),
            ("artifact_digests_sha256", [], "requires artifact_digests_sha256"),
        ):
            with self.subTest(receipt=field):
                invalid_receipt = copy.deepcopy(receipt)
                invalid_receipt[field] = value
                self.assert_receipt_error(invalid_receipt, message)

        delivery_mutations = (
            (
                self.delivery_checklists.replace(
                    "distinct announcement decision and receipt",
                    "announcement decision",
                    1,
                ),
                self.pr_template,
            ),
            (
                self.delivery_checklists.replace(
                    "Publication authorization does not authorize announcement.",
                    "Publication authorization may authorize announcement.",
                    1,
                ),
                self.pr_template,
            ),
            (
                self.delivery_checklists,
                self.pr_template.replace(
                    "Green checks are evidence, not merge, release, or "
                    "announcement authorization.",
                    "Green checks are evidence.",
                    1,
                ),
            ),
        )
        for checklist, pr_template in delivery_mutations:
            with self.subTest(delivery=True):
                self.assertTrue(
                    any(
                        "missing required marker" in error
                        for error in validate_delivery_documents(
                            checklist, pr_template
                        )
                    )
                )

    def test_non_transitive_rules_are_exact_and_negative(self) -> None:
        for mutation in (
            "missing",
            "duplicate",
            "reordered",
            "unknown",
            "positive",
        ):
            with self.subTest(mutation=mutation):
                policy = copy.deepcopy(self.policy)
                rules = policy["non_transitive_rules"]
                if mutation == "missing":
                    rules.pop()
                elif mutation == "duplicate":
                    rules.append(copy.deepcopy(rules[-1]))
                elif mutation == "reordered":
                    rules[0], rules[1] = rules[1], rules[0]
                elif mutation == "unknown":
                    rules[0]["source"] = "ci-red"
                else:
                    target = rules[0].pop("does_not_authorize")
                    rules[0]["authorizes"] = target
                self.assert_policy_error(policy, "closed negative transitions")

        ci_targets = (
            "repository.merge",
            "publication.publish-release",
            "publication.announce-release",
        )
        for target in ci_targets:
            rule_index = next(
                index
                for index, rule in enumerate(self.policy["non_transitive_rules"])
                if rule == {"source": "ci-green", "does_not_authorize": target}
            )
            for mutation in ("missing", "positive"):
                with self.subTest(source="ci-green", target=target, mutation=mutation):
                    policy = copy.deepcopy(self.policy)
                    rules = policy["non_transitive_rules"]
                    if mutation == "missing":
                        rules.pop(rule_index)
                    else:
                        target_action = rules[rule_index].pop("does_not_authorize")
                        rules[rule_index]["authorizes"] = target_action
                    self.assert_policy_error(policy, "closed negative transitions")

    def test_schema_is_closed_bounded_and_requires_digest_and_expiry(self) -> None:
        mutations = (
            lambda schema: schema.update({"$schema": "draft-07"}),
            lambda schema: schema.update(additionalProperties=True),
            lambda schema: schema["required"].remove("expires_at"),
            lambda schema: schema["properties"]["authorization_id"].update(
                pattern=".*"
            ),
            lambda schema: schema["properties"]["action"]["enum"].append(
                "repository.force-push"
            ),
            lambda schema: schema["properties"]["authorizer_role"]["enum"].append(
                "agent-ci-principal"
            ),
            lambda schema: schema["allOf"][0]["anyOf"][1]["properties"][
                "artifact_digests_sha256"
            ].update(minItems=0),
        )
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                schema = copy.deepcopy(self.schema)
                mutate(schema)
                self.assertNotEqual(
                    validate_policy(REPO_ROOT, self.policy, schema), []
                )

    def test_receipt_rejects_missing_digest_expiry_and_unknown_or_unsafe_data(self) -> None:
        cases = (
            (
                lambda receipt: receipt.update(
                    plan_digest_sha256=None,
                    artifact_digests_sha256=[],
                ),
                "requires a plan or artifact digest",
            ),
            (
                lambda receipt: receipt.pop("expires_at"),
                "contain exactly",
            ),
            (
                lambda receipt: receipt.update(
                    expires_at=receipt["issued_at"]
                ),
                "later than issued_at",
            ),
            (
                lambda receipt: receipt.update(action="repository.force-push"),
                "action is unknown",
            ),
            (
                lambda receipt: receipt.update(unexpected=True),
                "contain exactly",
            ),
            (
                lambda receipt: receipt.update(evidence_refs=["/tmp/secret"]),
                "machine-absolute paths",
            ),
        )
        for mutate, message in cases:
            with self.subTest(message=message):
                receipt = copy.deepcopy(self.receipt)
                mutate(receipt)
                self.assert_receipt_error(receipt, message)

    def test_policy_evidence_paths_must_be_canonical_repository_files(self) -> None:
        for path in ("../outside.md", "/tmp/secret", "missing.md"):
            with self.subTest(path=path):
                policy = copy.deepcopy(self.policy)
                policy["evidence"] = [path]
                self.assert_policy_error(policy, "evidence[0]")

    def test_duplicate_json_keys_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "duplicate.json"
            path.write_text(
                '{"schema_version":"1.0","schema_version":"2.0"}',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(
                AuthorizationPolicyError, "duplicate JSON key"
            ):
                load_document(path)


if __name__ == "__main__":
    unittest.main()
