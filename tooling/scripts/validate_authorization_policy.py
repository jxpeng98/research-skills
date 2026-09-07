#!/usr/bin/env python3
from __future__ import annotations

import argparse
from datetime import datetime
import json
from pathlib import Path, PurePosixPath
import re
import sys
from typing import Any, Sequence


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tooling.scripts.validate_arc_201_adrs import is_canonical_repository_path


DEFAULT_POLICY = REPO_ROOT / "tooling/architecture/authorization-policy-v1.json"
DEFAULT_SCHEMA = (
    REPO_ROOT / "tooling/architecture/authorization-receipt-v1.schema.json"
)
DEFAULT_REVIEW_POLICY = (
    REPO_ROOT / "tooling/architecture/repository-review-policy-v1.json"
)
DEFAULT_CODEOWNERS = REPO_ROOT / ".github/CODEOWNERS"
DEFAULT_DELIVERY_CHECKLISTS = REPO_ROOT / ".github/delivery-checklists.md"
DEFAULT_PR_TEMPLATE = REPO_ROOT / ".github/pull_request_template.md"
SCHEMA_DRAFT = "https://json-schema.org/draft/2020-12/schema"
SCHEMA_ID = "https://qiongli.dev/schemas/authorization-receipt/v1"

ROOT_KEYS = {
    "schema_version",
    "record_type",
    "branch",
    "receipt_schema",
    "evidence",
    "planes",
    "roles",
    "actions",
    "authorization_lifecycle",
    "evidence_signals",
    "non_transitive_rules",
}
PLANE_KEYS = {"id", "description"}
ROLE_KEYS = {"id", "primary_authority", "explicit_limits"}
ACTION_KEYS = {
    "id",
    "plane",
    "default_rule",
    "executor_roles",
    "authorizer_mode",
    "authorizer_roles",
    "required_bindings",
    "required_evidence",
}
RULE_KEYS = {"source", "does_not_authorize"}

EXPECTED_PLANES = ("research", "repository", "publication")
EXPECTED_ROLES = (
    "requester-task-owner",
    "contributor-agent-operator",
    "maintainer",
    "codeowner-specialist-reviewer",
    "researcher-reviewer-pi",
    "data-steward-ethics-authority",
    "release-approver",
    "agent-ci-principal",
)
EXPECTED_ACTIONS = (
    "research.preview-mutation",
    "research.apply-mutation",
    "research.destructive-mutation",
    "research.move-restricted-data",
    "repository.edit",
    "repository.stage",
    "repository.commit",
    "repository.push",
    "repository.pr-open-update",
    "repository.merge",
    "publication.publish-release",
    "publication.announce-release",
)
EXPECTED_SIGNALS = ("ci-green",)
EXPECTED_RULES = (
    ("research.preview-mutation", "research.apply-mutation"),
    ("repository.edit", "repository.stage"),
    ("repository.edit", "repository.commit"),
    ("repository.stage", "repository.commit"),
    ("repository.commit", "repository.push"),
    ("repository.push", "repository.merge"),
    ("repository.pr-open-update", "repository.merge"),
    ("repository.merge", "publication.publish-release"),
    ("ci-green", "repository.merge"),
    ("ci-green", "publication.publish-release"),
    ("repository.merge", "publication.announce-release"),
    ("publication.publish-release", "publication.announce-release"),
    ("ci-green", "publication.announce-release"),
)
EXPECTED_AUTHORIZATION_LIFECYCLE = [
    {
        "id": "denial",
        "scope": "all-actions",
        "trigger": "denied-receipt-issued",
        "effect": "action-blocked",
        "next_step": "new-authorization-required",
        "required_evidence": [
            "immutable-denial-receipt",
            "reason-code",
            "safe-next-action",
        ],
        "forbidden_transition": (
            "leak-policy-or-classified-content-or-revive-original-receipt"
        ),
    },
    {
        "id": "expiry",
        "scope": "all-actions",
        "trigger": "current-time-at-or-after-expires-at",
        "effect": "action-blocked-no-grace-or-replay",
        "next_step": "new-authorization-required",
        "required_evidence": ["original-receipt", "expires-at"],
        "forbidden_transition": "reuse-or-revive-expired-receipt",
    },
    {
        "id": "revocation",
        "scope": "all-actions",
        "trigger": "revocation-receipt-issued",
        "effect": (
            "future-use-blocked-in-flight-safe-cancel-no-partial-canonical-state"
        ),
        "next_step": "new-authorization-required",
        "required_evidence": [
            "original-receipt-reference",
            "immutable-revocation-receipt",
            "exact-binding-match",
            "reason-code",
        ],
        "forbidden_transition": "reuse-revive-or-widen-original-receipt",
    },
    {
        "id": "emergency-hotfix",
        "scope": "repository-plane-only",
        "trigger": "incident-declared",
        "effect": "new-finite-approval-through-protected-pr",
        "next_step": "post-incident-reconciliation-required",
        "required_evidence": [
            "incident-reference",
            "named-human-decision",
            "minimum-scope",
            "new-approved-receipt",
            "exact-head",
            "required-checks",
            "rollback-plan",
            "finite-expiry",
            "full-audit-trail",
        ],
        "forbidden_transition": (
            "force-push-required-check-bypass-or-non-repository-action"
        ),
    },
    {
        "id": "post-incident-reconciliation",
        "scope": "repository-plane-only",
        "trigger": "emergency-hotfix-completed-or-aborted",
        "effect": "audit-evidence-only-block-next-emergency-until-complete",
        "next_step": "normal-authorization-workflow",
        "required_evidence": [
            "incident-reference",
            "exact-action-and-impact",
            "verification-results",
            "rollback-status",
            "follow-up-owner",
            "follow-up-review",
            "reviewer-blocker-truth",
        ],
        "forbidden_transition": "retroactive-authorization-or-scope-widening",
    },
]
EXPECTED_DIGEST_RULE = [
    {
        "anyOf": [
            {"properties": {"plan_digest_sha256": {"type": "string"}}},
            {"properties": {"artifact_digests_sha256": {"minItems": 1}}},
        ]
    }
]
EXPECTED_AUTHORIZERS = {
    "research.preview-mutation": (
        "any-of",
        ("requester-task-owner", "researcher-reviewer-pi"),
    ),
    "research.apply-mutation": ("all-of", ("researcher-reviewer-pi",)),
    "research.destructive-mutation": (
        "any-of",
        ("researcher-reviewer-pi", "data-steward-ethics-authority"),
    ),
    "research.move-restricted-data": (
        "all-of",
        ("data-steward-ethics-authority",),
    ),
    "repository.edit": (
        "any-of",
        ("requester-task-owner", "maintainer"),
    ),
    "repository.stage": (
        "any-of",
        ("requester-task-owner", "maintainer"),
    ),
    "repository.commit": (
        "any-of",
        ("requester-task-owner", "maintainer"),
    ),
    "repository.push": (
        "any-of",
        ("requester-task-owner", "maintainer"),
    ),
    "repository.pr-open-update": (
        "any-of",
        ("requester-task-owner", "maintainer"),
    ),
    "repository.merge": (
        "all-of",
        ("maintainer", "codeowner-specialist-reviewer"),
    ),
    "publication.publish-release": ("all-of", ("release-approver",)),
    "publication.announce-release": ("all-of", ("release-approver",)),
}
EXPECTED_DEFAULT_RULES = {
    "research.preview-mutation": "scoped-preview-only",
    "research.apply-mutation": "preview-before-apply",
    "research.destructive-mutation": "deny-by-default",
    "research.move-restricted-data": "deny-by-default",
    "repository.edit": "explicit-write-set",
    "repository.stage": "explicit-staged-paths",
    "repository.commit": "explicit-commit",
    "repository.push": "explicit-push-intent",
    "repository.pr-open-update": "draft-first",
    "repository.merge": "protected-pr-only",
    "publication.publish-release": "separate-release-authorization",
    "publication.announce-release": "verified-publication-before-announcement",
}
MINIMUM_EVIDENCE = {
    "research.preview-mutation": {"task-scope", "redacted-preview"},
    "research.apply-mutation": {"redacted-preview", "human-decision"},
    "research.destructive-mutation": {
        "human-decision",
        "backup-rollback",
        "specialist-review",
    },
    "research.move-restricted-data": {
        "data-steward-ethics-record",
        "human-decision",
    },
    "repository.edit": {"task-scope"},
    "repository.stage": {"task-scope", "staged-diff"},
    "repository.commit": {"staged-diff", "local-checks"},
    "repository.push": {"clean-checkpoint", "pre-push-checklist"},
    "repository.pr-open-update": {"exact-head", "scope-nonclaims"},
    "repository.merge": {
        "exact-head",
        "required-checks",
        "reviewer-codeowner-approval",
    },
    "publication.publish-release": {
        "exact-commit",
        "asset-digests",
        "release-approval",
    },
    "publication.announce-release": {
        "publication-receipt",
        "public-verification",
        "announcement-content-digest",
        "announcement-approval",
    },
}
COMMON_BINDINGS = {
    "object-scope",
    "project-or-source-revision",
    "constraints",
    "expiry",
}
ALLOWED_BINDINGS = COMMON_BINDINGS | {
    "plan-digest",
    "artifact-digests",
    "data-classification",
    "destination",
    "rollback",
    "channels",
}
DATA_CLASSIFICATIONS = (
    "public",
    "internal",
    "confidential",
    "restricted",
    "regulated",
)
DECISIONS = ("approved", "denied", "revoked", "expired")
RECEIPT_FIELDS = (
    "schema_version",
    "record_type",
    "authorization_id",
    "action",
    "object_scope",
    "actor_role",
    "authorizer_role",
    "project_or_source_revision",
    "plan_digest_sha256",
    "artifact_digests_sha256",
    "data_classification",
    "decision",
    "constraints",
    "reason_code",
    "issued_at",
    "expires_at",
    "evidence_refs",
)
SCHEMA_KEYS = {
    "$schema",
    "$id",
    "title",
    "description",
    "type",
    "additionalProperties",
    "required",
    "properties",
    "allOf",
    "examples",
}
REVIEW_ROOT_KEYS = {
    "schema_version",
    "record_type",
    "repository",
    "branch",
    "evidence",
    "ruleset",
    "history_policy",
    "codeowners",
    "review_enforcement",
}
REVIEW_RULESET_KEYS = {
    "id",
    "name",
    "target",
    "enforcement",
    "ref_include",
    "ref_exclude",
    "bypass_actors",
    "rules",
    "pull_request",
    "strict_required_status_checks_policy",
    "do_not_enforce_on_create",
    "required_status_checks",
}
REVIEW_PULL_REQUEST_KEYS = {
    "required_approving_review_count",
    "dismiss_stale_reviews_on_push",
    "require_code_owner_review",
    "require_last_push_approval",
    "required_review_thread_resolution",
    "allowed_merge_methods",
}
REVIEW_CODEOWNERS_KEYS = {"path", "owners", "domains"}
REVIEW_STATUS_CHECK_KEYS = {"context", "integration_id"}
REVIEW_ENFORCEMENT_KEYS = {
    "state",
    "blocker_reason_code",
    "blocker",
    "activation_requirements",
}

EXPECTED_REVIEW_DOMAINS = (
    "security",
    "schema",
    "migration",
    "release",
    "research-gate",
    "authorization",
)
EXPECTED_REVIEW_PATTERNS = {
    "security": (
        "/.github/",
        "/packages/qiongli-native/crates/qiongli-windows-security/",
        "/packages/qiongli-native/crates/qiongli-config/src/secret.rs",
        "/packages/qiongli-native/crates/qiongli-config/src/redaction.rs",
        "/packages/qiongli-native/crates/qiongli-execution/src/policy.rs",
        "/packages/qiongli-native/crates/qiongli-execution/src/tool_host.rs",
        "/packages/qiongli-native/crates/qiongli-runtime/src/providers/access.rs",
        "/packages/qiongli-native/crates/qiongli-platform/src/grant.rs",
    ),
    "schema": (
        "/packages/qiongli-native/apps/qiongli/src/",
        "/packages/qiongli-native/crates/qiongli-runtime/src/contract.rs",
        "/packages/qiongli-app-api/src/",
        "/content/mcp-contracts/v2/",
        "/content/schemas/",
        "/tooling/architecture/public-schema-policy.json",
        "/tooling/scripts/validate_public_schema_policy.py",
        "/tests/test_public_schema_policy.py",
        "/docs/reference/cli.md",
    ),
    "migration": (
        "/tooling/migration/",
        "/packages/qiongli-native/crates/qiongli-project/src/migration.rs",
        "/packages/qiongli-native/crates/qiongli-platform/src/legacy_migration.rs",
        "/packages/qiongli-native/apps/qiongli/src/legacy_migration_cli.rs",
    ),
    "release": (
        "/.github/workflows/",
        "/tooling/release/",
        "/scripts/release_automation.sh",
        "/scripts/release_preflight.sh",
        "/scripts/release_postflight.sh",
        "/scripts/release_ready.sh",
        "/scripts/release_version.py",
        "/scripts/release_upload_assets.py",
        "/packages/qiongli-native/release/",
        "/packages/qiongli-native/crates/qiongli-platform/src/release_authority.rs",
        "/packages/qiongli-native/crates/qiongli-platform/src/release_candidate.rs",
        "/packages/qiongli-native/crates/qiongli-platform/src/native_release.rs",
    ),
    "research-gate": (
        "/content/standards/quality-gate-contract.yaml",
        "/content/templates/quality-gate-report.md",
        "/tests/test_quality_gate_contract.py",
    ),
    "authorization": (
        "/.github/CODEOWNERS",
        "/tooling/architecture/authorization-policy-v1.json",
        "/tooling/architecture/authorization-receipt-v1.schema.json",
        "/tooling/architecture/repository-review-policy-v1.json",
        "/tooling/scripts/validate_authorization_policy.py",
        "/tests/test_authorization_policy.py",
        "/.trellis/spec/product/control/authorization-policy-v1.md",
        "/docs/superpowers/roadmaps/qiongli-program-ledger-v1.json",
        "/tooling/scripts/update_program_roadmap.py",
        "/tests/test_program_roadmap.py",
    ),
}
EXPECTED_RULESET_RULES = (
    "deletion",
    "non_fast_forward",
    "pull_request",
    "required_status_checks",
)
EXPECTED_REQUIRED_STATUS_CHECKS = (
    ("Native 2.x change boundary", 15368),
    ("Rust native foundation (Linux)", 15368),
    ("Rust native foundation (macOS)", 15368),
    ("Rust native foundation (Windows)", 15368),
    ("Evaluation Truth V1", 15368),
)
EXPECTED_ACTIVATION_REQUIREMENTS = (
    "at-least-two-distinct-write-authorized-human-codeowners",
    "required-approving-review-count-at-least-one",
    "required-codeowner-review-enabled",
    "non-stale-independent-approval-on-exact-head",
)
EXPECTED_HISTORY_POLICY = {
    "exact_head": {
        "identity": "full-lowercase-commit-sha",
        "head_change_events": [
            "new-commit",
            "amend",
            "rebase",
            "merge",
            "history-rewrite",
        ],
        "evidence_rules": [
            {
                "id": "exact-head-ci",
                "invalidated_by": "any-head-change",
                "reuse": "current-head-only",
            },
            {
                "id": "review",
                "invalidated_by": "any-head-change",
                "reuse": "current-head-only",
            },
            {
                "id": "authorization",
                "invalidated_by": (
                    "bound-revision-scope-plan-or-artifact-digest-change"
                ),
                "reuse": "all-receipt-bindings-current",
            },
            {
                "id": "package",
                "invalidated_by": "bound-package-input-or-digest-change",
                "reuse": "all-package-bindings-identical",
            },
            {
                "id": "release",
                "invalidated_by": (
                    "source-version-target-digest-metadata-destination-channel-"
                    "or-claim-change"
                ),
                "reuse": "never-transfer-to-replacement-candidate",
            },
        ],
    },
    "protected_refs": {
        "patterns": [
            "refs/heads/2.x",
            "refs/heads/release/*",
            "refs/tags/*",
        ],
        "update_path": "protected-pr-or-authorized-release-only",
        "force_push": "forbidden",
        "history_rewrite": "forbidden",
        "accepted_evidence_head_rewrite": "forbidden",
    },
    "feature_branch_rewrite": {
        "eligibility": "unprotected-unpublished-no-accepted-evidence",
        "mode": "exceptional",
        "authority": "owner-approval",
        "review_notice": "before-review-or-explicit-reviewer-notice",
        "push_mode": "force-with-lease-only",
        "replaced_receipts": "invalidate-all",
        "preferred_alternative": "ordinary-follow-up-commit",
    },
}
EXPECTED_AUTHORIZATION_DELIVERY_EVIDENCE = {".github/delivery-checklists.md"}
EXPECTED_REVIEW_DELIVERY_EVIDENCE = {
    ".github/delivery-checklists.md",
    ".github/pull_request_template.md",
}
DELIVERY_REQUIRED_MARKERS = (
    "**Focused**",
    "**Slice**",
    "**Acceptance**",
    "git diff --cached --check",
    "git diff --check 2.x...HEAD",
    "git rev-parse HEAD",
    "./scripts/check_2x_native_change_boundary.sh --base-ref 2.x",
    "git merge --ff-only",
    "./scripts/release_ready.sh --version <version> --staging-dir <external-dir>",
    "gh workflow run native-ci.yml --ref 2.x",
    "Every head change invalidates stale exact-head",
    "Plain `--force` is forbidden.",
    "`--force-with-lease` only",
    "A green check is evidence, not authorization",
    "distinct announcement decision and receipt",
    "Publication authorization does not authorize announcement.",
    "Denied, expired, or revoked authorization blocks execution",
    "Emergency hotfix authorization is repository-only",
    "Post-incident reconciliation is evidence, not retroactive authorization.",
    "Tags and published assets are immutable.",
)
PR_TEMPLATE_REQUIRED_MARKERS = (
    ".github/delivery-checklists.md",
    "- Focused commands/results:",
    "- Required checks/run links:",
    "Every head change invalidates stale exact-head",
    "Green checks are evidence, not merge, release, or announcement authorization.",
)

IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9._-]{0,127}$")
CODE = re.compile(r"^[a-z0-9][a-z0-9-]{0,63}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")
AUTHORIZATION_ID = re.compile(r"^auth_[0-9a-f]{32}$")
REVISION = re.compile(r"^(?:[0-9a-f]{40}|project-revision:[1-9][0-9]{0,19})$")
EVIDENCE_REF = re.compile(r"^[a-z0-9][a-z0-9._:#/-]{0,255}$")
CODEOWNER = re.compile(r"^@[A-Za-z0-9](?:[A-Za-z0-9-]{0,37}[A-Za-z0-9])?$")


class AuthorizationPolicyError(ValueError):
    pass


def _reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise AuthorizationPolicyError(f"duplicate JSON key: {key}")
        value[key] = item
    return value


def load_document(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"), object_pairs_hook=_reject_duplicate_keys
        )
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise AuthorizationPolicyError(f"cannot load {path}: {error}") from error
    if not isinstance(value, dict):
        raise AuthorizationPolicyError(f"{path} must contain a JSON object")
    return value


def load_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise AuthorizationPolicyError(f"cannot load {path}: {error}") from error


def resolve_repository_file(repo_root: Path, relative: str) -> Path:
    if not is_canonical_repository_path(relative):
        raise AuthorizationPolicyError(
            "path must be a canonical repository-relative POSIX path"
        )
    root = repo_root.resolve(strict=True)
    candidate = repo_root
    for part in PurePosixPath(relative).parts:
        candidate = candidate / part
        if candidate.is_symlink():
            raise AuthorizationPolicyError("path must not contain a symbolic link")
    try:
        resolved = candidate.resolve(strict=True)
        resolved.relative_to(root)
    except (OSError, ValueError, RuntimeError) as error:
        raise AuthorizationPolicyError("path must resolve inside the repository") from error
    if not resolved.is_file():
        raise AuthorizationPolicyError("path must resolve to a regular file")
    return resolved


def resolve_codeowner_pattern(repo_root: Path, pattern: str) -> Path:
    if not pattern.startswith("/") or pattern.startswith("//"):
        raise AuthorizationPolicyError(
            "CODEOWNERS pattern must be rooted at the repository"
        )
    if any(character in pattern for character in "*?[]#") or any(
        character.isspace() for character in pattern
    ):
        raise AuthorizationPolicyError(
            "CODEOWNERS pattern must be a literal path without glob syntax"
        )
    is_directory = pattern.endswith("/")
    relative = pattern[1:-1] if is_directory else pattern[1:]
    if not is_canonical_repository_path(relative):
        raise AuthorizationPolicyError(
            "CODEOWNERS pattern must be a canonical literal repository path"
        )
    root = repo_root.resolve(strict=True)
    candidate = repo_root
    for part in PurePosixPath(relative).parts:
        candidate = candidate / part
        if candidate.is_symlink():
            raise AuthorizationPolicyError(
                "CODEOWNERS pattern must not contain a symbolic link"
            )
    try:
        resolved = candidate.resolve(strict=True)
        resolved.relative_to(root)
    except (OSError, ValueError, RuntimeError) as error:
        raise AuthorizationPolicyError(
            "CODEOWNERS pattern must resolve inside the repository"
        ) from error
    if is_directory and not resolved.is_dir():
        raise AuthorizationPolicyError(
            "CODEOWNERS directory pattern must resolve to a directory"
        )
    if not is_directory and not resolved.is_file():
        raise AuthorizationPolicyError(
            "CODEOWNERS file pattern must resolve to a regular file"
        )
    return resolved


def _exact_keys(value: object, expected: set[str], label: str, errors: list[str]) -> bool:
    if not isinstance(value, dict):
        errors.append(f"{label} must be an object")
        return False
    if set(value) != expected:
        errors.append(f"{label} must contain exactly {sorted(expected)}")
    return True


def _nonempty_text(value: object) -> bool:
    return isinstance(value, str) and 0 < len(value.strip()) <= 512


def _unique_strings(
    value: object,
    label: str,
    errors: list[str],
    *,
    allow_empty: bool = False,
) -> list[str] | None:
    if not isinstance(value, list) or (not value and not allow_empty):
        requirement = "possibly empty" if allow_empty else "non-empty"
        errors.append(f"{label} must be a {requirement} array")
        return None
    if any(not isinstance(item, str) for item in value):
        errors.append(f"{label} must contain only strings")
        return None
    if len(value) != len(set(value)):
        errors.append(f"{label} must not contain duplicates")
    return value


def _validate_path(repo_root: Path, value: object, label: str, errors: list[str]) -> None:
    if not isinstance(value, str):
        errors.append(f"{label} must be a string path")
        return
    try:
        resolve_repository_file(repo_root, value)
    except AuthorizationPolicyError as error:
        errors.append(f"{label}: {error}")


def _validate_inventory(
    value: object,
    expected_ids: tuple[str, ...],
    keys: set[str],
    label: str,
    errors: list[str],
) -> list[dict[str, Any]]:
    if not isinstance(value, list):
        errors.append(f"{label} must be an array")
        return []
    ids = [item.get("id") for item in value if isinstance(item, dict)]
    if tuple(ids) != expected_ids:
        errors.append(f"{label} must contain its closed inventory exactly once and in order")
    valid: list[dict[str, Any]] = []
    for index, item in enumerate(value):
        if _exact_keys(item, keys, f"{label}[{index}]", errors):
            assert isinstance(item, dict)
            valid.append(item)
    return valid


def _expected_plane(action_id: str) -> str:
    return action_id.split(".", 1)[0]


def validate_policy_document(repo_root: Path, policy: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    _exact_keys(policy, ROOT_KEYS, "authorization policy", errors)
    if policy.get("schema_version") != "1.0":
        errors.append("authorization policy schema_version must be '1.0'")
    if policy.get("record_type") != "qiongli-authorization-policy":
        errors.append("authorization policy record_type is invalid")
    if policy.get("branch") != "2.x":
        errors.append("authorization policy must be bound to branch '2.x'")
    if policy.get("receipt_schema") != (
        "tooling/architecture/authorization-receipt-v1.schema.json"
    ):
        errors.append("authorization policy must name the canonical receipt schema")
    else:
        _validate_path(repo_root, policy.get("receipt_schema"), "receipt_schema", errors)

    evidence = _unique_strings(policy.get("evidence"), "evidence", errors)
    if evidence is not None:
        for index, path in enumerate(evidence):
            _validate_path(repo_root, path, f"evidence[{index}]", errors)
        if not EXPECTED_AUTHORIZATION_DELIVERY_EVIDENCE <= set(evidence):
            errors.append("authorization policy must name the delivery checklist")

    planes = _validate_inventory(
        policy.get("planes"), EXPECTED_PLANES, PLANE_KEYS, "planes", errors
    )
    for index, plane in enumerate(planes):
        if not _nonempty_text(plane.get("description")):
            errors.append(f"planes[{index}].description must be bounded text")

    roles = _validate_inventory(
        policy.get("roles"), EXPECTED_ROLES, ROLE_KEYS, "roles", errors
    )
    for index, role in enumerate(roles):
        if not _nonempty_text(role.get("primary_authority")):
            errors.append(f"roles[{index}].primary_authority must be bounded text")
        if not _nonempty_text(role.get("explicit_limits")):
            errors.append(f"roles[{index}].explicit_limits must be bounded text")

    actions = _validate_inventory(
        policy.get("actions"), EXPECTED_ACTIONS, ACTION_KEYS, "actions", errors
    )
    role_ids = set(EXPECTED_ROLES)
    for index, action in enumerate(actions):
        label = f"actions[{index}]"
        action_id = action.get("id")
        if not isinstance(action_id, str) or action_id not in EXPECTED_ACTIONS:
            errors.append(f"{label}.id is unknown")
            continue
        if action.get("plane") != _expected_plane(action_id):
            errors.append(f"{action_id}.plane does not match its action namespace")
        if action.get("default_rule") != EXPECTED_DEFAULT_RULES[action_id]:
            errors.append(f"{action_id}.default_rule must retain its v1 rule")

        executors = _unique_strings(
            action.get("executor_roles"), f"{action_id}.executor_roles", errors
        )
        if executors is not None and not set(executors) <= role_ids:
            errors.append(f"{action_id}.executor_roles contains an unknown role")
        if action_id == "publication.announce-release" and tuple(
            executors or ()
        ) != ("maintainer", "agent-ci-principal"):
            errors.append(f"{action_id} must retain its exact executor roles")

        authorizers = _unique_strings(
            action.get("authorizer_roles"), f"{action_id}.authorizer_roles", errors
        )
        expected_mode, expected_roles = EXPECTED_AUTHORIZERS[action_id]
        if (
            action.get("authorizer_mode") != expected_mode
            or tuple(authorizers or ()) != expected_roles
        ):
            errors.append(f"{action_id} must retain its exact authorizer rule")
        if authorizers is not None:
            if not set(authorizers) <= role_ids:
                errors.append(f"{action_id}.authorizer_roles contains an unknown role")
            if "agent-ci-principal" in authorizers:
                errors.append("Agent/CI principal cannot be an authorizer")

        bindings = _unique_strings(
            action.get("required_bindings"), f"{action_id}.required_bindings", errors
        )
        if bindings is not None:
            binding_set = set(bindings)
            if not binding_set <= ALLOWED_BINDINGS:
                errors.append(f"{action_id}.required_bindings contains an unknown binding")
            if not COMMON_BINDINGS <= binding_set:
                errors.append(f"{action_id} is missing a common scope/revision/time binding")
            if not {"plan-digest", "artifact-digests"} & binding_set:
                errors.append(f"{action_id} must require a plan or artifact digest")
            if (
                action_id == "research.destructive-mutation"
                and "rollback" not in binding_set
            ):
                errors.append(f"{action_id} must bind rollback")
            if action_id == "research.move-restricted-data" and not {
                "data-classification",
                "destination",
            } <= binding_set:
                errors.append(f"{action_id} must bind classification and destination")
            if action_id == "publication.publish-release" and not {
                "artifact-digests",
                "channels",
            } <= binding_set:
                errors.append(f"{action_id} must bind exact artifacts and channels")
            if action_id == "publication.announce-release" and not {
                "plan-digest",
                "artifact-digests",
                "channels",
            } <= binding_set:
                errors.append(
                    f"{action_id} must bind announcement plan, artifacts, and channels"
                )

        required_evidence = _unique_strings(
            action.get("required_evidence"), f"{action_id}.required_evidence", errors
        )
        if required_evidence is not None:
            if any(not CODE.fullmatch(item) for item in required_evidence):
                errors.append(f"{action_id}.required_evidence contains an invalid code")
            if not MINIMUM_EVIDENCE[action_id] <= set(required_evidence):
                errors.append(f"{action_id} is missing required evidence")

    if policy.get("authorization_lifecycle") != EXPECTED_AUTHORIZATION_LIFECYCLE:
        errors.append(
            "authorization_lifecycle must retain the closed fail-closed paths "
            "exactly once and in order"
        )

    signals = policy.get("evidence_signals")
    if not isinstance(signals, list) or tuple(signals) != EXPECTED_SIGNALS:
        errors.append("evidence_signals must contain ci-green exactly once and in order")

    rules = policy.get("non_transitive_rules")
    if not isinstance(rules, list):
        errors.append("non_transitive_rules must be an array")
    else:
        pairs: list[tuple[object, object]] = []
        for index, rule in enumerate(rules):
            if _exact_keys(rule, RULE_KEYS, f"non_transitive_rules[{index}]", errors):
                assert isinstance(rule, dict)
                pairs.append((rule.get("source"), rule.get("does_not_authorize")))
        if tuple(pairs) != EXPECTED_RULES:
            errors.append(
                "non_transitive_rules must contain the closed negative transitions "
                "exactly once and in order"
            )
        known_sources = set(EXPECTED_ACTIONS) | set(EXPECTED_SIGNALS)
        if any(
            source not in known_sources or target not in EXPECTED_ACTIONS
            for source, target in pairs
        ):
            errors.append("non_transitive_rules contains an unknown source or action")
    return errors


def _parse_codeowners(
    content: str, errors: list[str]
) -> list[tuple[str, tuple[str, ...]]]:
    lines = content.splitlines()
    if not lines or lines[0] != "# qiongli-repository-review-policy/v1":
        errors.append("CODEOWNERS must declare its v1 policy header")
    entries: list[tuple[str, tuple[str, ...]]] = []
    for line_number, line in enumerate(lines, start=1):
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        parts = stripped.split()
        if len(parts) < 2:
            errors.append(
                f"CODEOWNERS line {line_number} must contain a pattern and owner"
            )
            continue
        pattern, *owners = parts
        if any(not CODEOWNER.fullmatch(owner) for owner in owners):
            errors.append(f"CODEOWNERS line {line_number} contains an invalid owner")
        entries.append((pattern, tuple(owners)))
    patterns = [pattern for pattern, _owners in entries]
    if len(patterns) != len(set(patterns)):
        errors.append("CODEOWNERS must not contain duplicate patterns")
    return entries


def validate_review_policy(
    repo_root: Path,
    review_policy: dict[str, Any],
    codeowners_content: str,
) -> list[str]:
    errors: list[str] = []
    _exact_keys(review_policy, REVIEW_ROOT_KEYS, "repository review policy", errors)
    if review_policy.get("schema_version") != "1.0":
        errors.append("repository review policy schema_version must be '1.0'")
    if review_policy.get("record_type") != "qiongli-repository-review-policy":
        errors.append("repository review policy record_type is invalid")
    if review_policy.get("repository") != "jxpeng98/qiongli":
        errors.append("repository review policy repository is invalid")
    if review_policy.get("branch") != "2.x":
        errors.append("repository review policy must be bound to branch '2.x'")
    evidence = _unique_strings(
        review_policy.get("evidence"), "review policy evidence", errors
    )
    if evidence is not None:
        for index, path in enumerate(evidence):
            _validate_path(repo_root, path, f"review policy evidence[{index}]", errors)
        if not EXPECTED_REVIEW_DELIVERY_EVIDENCE <= set(evidence):
            errors.append(
                "review policy must name the delivery checklist and PR template"
            )

    if review_policy.get("history_policy") != EXPECTED_HISTORY_POLICY:
        errors.append(
            "repository history policy must retain its exact v1 invalidation and "
            "rewrite rules"
        )

    ruleset = review_policy.get("ruleset")
    approval_count: object = None
    code_owner_review: object = None
    if _exact_keys(ruleset, REVIEW_RULESET_KEYS, "review policy ruleset", errors):
        assert isinstance(ruleset, dict)
        expected_scalars = {
            "id": 18800504,
            "name": "2.x protected native development",
            "target": "branch",
            "enforcement": "active",
        }
        for key, expected in expected_scalars.items():
            if ruleset.get(key) != expected:
                errors.append(f"review policy ruleset.{key} must remain {expected!r}")
        for key in (
            "strict_required_status_checks_policy",
            "do_not_enforce_on_create",
        ):
            if ruleset.get(key) is not True:
                errors.append(f"review policy ruleset.{key} must remain True")
        if ruleset.get("ref_include") != ["refs/heads/2.x"]:
            errors.append("review policy ruleset must target only refs/heads/2.x")
        if ruleset.get("ref_exclude") != []:
            errors.append("review policy ruleset ref_exclude must remain empty")
        if ruleset.get("bypass_actors") != []:
            errors.append("review policy ruleset must not define bypass actors")
        protected_rules = _unique_strings(
            ruleset.get("rules"), "review policy ruleset.rules", errors
        )
        if tuple(protected_rules or ()) != EXPECTED_RULESET_RULES:
            errors.append("review policy ruleset must retain its exact protected rules")

        pull_request = ruleset.get("pull_request")
        if _exact_keys(
            pull_request,
            REVIEW_PULL_REQUEST_KEYS,
            "review policy pull_request",
            errors,
        ):
            assert isinstance(pull_request, dict)
            approval_count = pull_request.get("required_approving_review_count")
            code_owner_review = pull_request.get("require_code_owner_review")
            if type(approval_count) is not int or not 0 <= approval_count <= 10:
                errors.append(
                    "required_approving_review_count must be an integer from 0 to 10"
                )
            if type(code_owner_review) is not bool:
                errors.append("require_code_owner_review must be a boolean")
            expected_pull_request = {
                "dismiss_stale_reviews_on_push": True,
                "require_last_push_approval": False,
                "required_review_thread_resolution": True,
            }
            for key, expected in expected_pull_request.items():
                if pull_request.get(key) is not expected:
                    errors.append(
                        f"review policy pull_request.{key} must remain {expected!r}"
                    )
            if pull_request.get("allowed_merge_methods") != [
                "merge",
                "squash",
                "rebase",
            ]:
                errors.append(
                    "review policy pull_request.allowed_merge_methods must retain "
                    "['merge', 'squash', 'rebase']"
                )

        status_checks = ruleset.get("required_status_checks")
        pairs: list[tuple[object, object]] = []
        if not isinstance(status_checks, list):
            errors.append("required_status_checks must be an array")
        else:
            for index, status_check in enumerate(status_checks):
                if _exact_keys(
                    status_check,
                    REVIEW_STATUS_CHECK_KEYS,
                    f"required_status_checks[{index}]",
                    errors,
                ):
                    assert isinstance(status_check, dict)
                    pairs.append(
                        (status_check.get("context"), status_check.get("integration_id"))
                    )
            if tuple(pairs) != EXPECTED_REQUIRED_STATUS_CHECKS:
                errors.append(
                    "required_status_checks must retain the exact native and "
                    "Evaluation Truth contexts"
                )

    flattened: list[tuple[str, tuple[str, ...]]] = []
    owners: list[str] | None = None
    codeowners = review_policy.get("codeowners")
    if _exact_keys(codeowners, REVIEW_CODEOWNERS_KEYS, "codeowners", errors):
        assert isinstance(codeowners, dict)
        if codeowners.get("path") != ".github/CODEOWNERS":
            errors.append("repository review policy must name .github/CODEOWNERS")
        else:
            _validate_path(repo_root, codeowners.get("path"), "codeowners.path", errors)

        owners = _unique_strings(codeowners.get("owners"), "codeowners.owners", errors)
        if owners is not None:
            if any(not CODEOWNER.fullmatch(owner) for owner in owners):
                errors.append("codeowners.owners contains an invalid owner")
            if "@jxpeng98" not in owners:
                errors.append("codeowners.owners must retain @jxpeng98")

        domains = codeowners.get("domains")
        if not isinstance(domains, dict):
            errors.append("codeowners.domains must be an object")
        else:
            if tuple(domains) != EXPECTED_REVIEW_DOMAINS:
                errors.append(
                    "codeowners.domains must contain the six sensitive domains "
                    "exactly once and in order"
                )
            for domain_id in EXPECTED_REVIEW_DOMAINS:
                patterns = _unique_strings(
                    domains.get(domain_id), f"codeowners.domains.{domain_id}", errors
                )
                if tuple(patterns or ()) != EXPECTED_REVIEW_PATTERNS[domain_id]:
                    errors.append(f"{domain_id} must retain its exact v1 paths")
                for index, pattern in enumerate(patterns or []):
                    try:
                        resolve_codeowner_pattern(repo_root, pattern)
                    except AuthorizationPolicyError as error:
                        errors.append(
                            f"codeowners.domains.{domain_id}[{index}]: {error}"
                        )
                    if owners is not None:
                        flattened.append((pattern, tuple(owners)))
            flattened_patterns = [pattern for pattern, _owners in flattened]
            if len(flattened_patterns) != len(set(flattened_patterns)):
                errors.append("codeowners.domains must not contain duplicate patterns")

    parsed_codeowners = _parse_codeowners(codeowners_content, errors)
    if parsed_codeowners != flattened:
        errors.append("CODEOWNERS entries must exactly match the repository review policy")

    enforcement = review_policy.get("review_enforcement")
    if _exact_keys(
        enforcement,
        REVIEW_ENFORCEMENT_KEYS,
        "review_enforcement",
        errors,
    ):
        assert isinstance(enforcement, dict)
        activation_requirements = _unique_strings(
            enforcement.get("activation_requirements"),
            "review_enforcement.activation_requirements",
            errors,
        )
        if tuple(activation_requirements or ()) != EXPECTED_ACTIVATION_REQUIREMENTS:
            errors.append(
                "review_enforcement must retain the exact activation requirements"
            )
        state = enforcement.get("state")
        blocker_reason = enforcement.get("blocker_reason_code")
        blocker = enforcement.get("blocker")
        if state == "blocked":
            if approval_count != 0 or code_owner_review is not False:
                errors.append(
                    "blocked review enforcement must keep approvals and CODEOWNER "
                    "review disabled"
                )
            if blocker_reason != "independent-reviewer-unavailable":
                errors.append("blocked review enforcement has an invalid reason code")
            if not _nonempty_text(blocker):
                errors.append("blocked review enforcement requires a bounded blocker")
        elif state == "enforced":
            if type(approval_count) is not int or approval_count < 1:
                errors.append("enforced review policy requires at least one approval")
            if code_owner_review is not True:
                errors.append("enforced review policy requires CODEOWNER review")
            if blocker_reason != "" or blocker != "":
                errors.append("enforced review policy must clear its blocker")
            if owners is None or len(set(owners)) < 2:
                errors.append(
                    "enforced review policy requires two distinct owners on every path"
                )
        else:
            errors.append("review_enforcement.state must be blocked or enforced")
    return errors


def validate_delivery_documents(
    checklist_content: str,
    pr_template_content: str,
) -> list[str]:
    errors: list[str] = []
    # Validate authority/evidence boundaries, not Markdown layout or approval rituals.
    for marker in DELIVERY_REQUIRED_MARKERS:
        if marker not in checklist_content:
            errors.append(f"delivery checklist is missing required marker: {marker}")

    for marker in PR_TEMPLATE_REQUIRED_MARKERS:
        if marker not in pr_template_content:
            errors.append(f"PR template is missing required marker: {marker}")
    return errors


def _parse_timestamp(value: object, label: str, errors: list[str]) -> datetime | None:
    if not isinstance(value, str):
        errors.append(f"{label} must be an RFC 3339 timestamp")
        return None
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        errors.append(f"{label} must be an RFC 3339 timestamp")
        return None
    if parsed.tzinfo is None:
        errors.append(f"{label} must include a timezone")
        return None
    return parsed


def validate_receipt(policy: dict[str, Any], receipt: object) -> list[str]:
    errors: list[str] = []
    if not _exact_keys(receipt, set(RECEIPT_FIELDS), "authorization receipt", errors):
        return errors
    assert isinstance(receipt, dict)
    if receipt.get("schema_version") != "1.0":
        errors.append("authorization receipt schema_version must be '1.0'")
    if receipt.get("record_type") != "qiongli-authorization-receipt":
        errors.append("authorization receipt record_type is invalid")
    if not isinstance(
        receipt.get("authorization_id"), str
    ) or not AUTHORIZATION_ID.fullmatch(receipt["authorization_id"]):
        errors.append("authorization_id must be an opaque auth_ identifier")

    policy_actions = policy.get("actions")
    actions = {
        action.get("id"): action
        for action in (policy_actions if isinstance(policy_actions, list) else [])
        if isinstance(action, dict) and isinstance(action.get("id"), str)
    }
    action_id = receipt.get("action")
    action = actions.get(action_id)
    if action is None:
        errors.append("authorization receipt action is unknown")

    scope = _unique_strings(receipt.get("object_scope"), "object_scope", errors)
    if scope is not None:
        if len(scope) > 32 or any(not IDENTIFIER.fullmatch(item) for item in scope):
            errors.append("object_scope must contain bounded redacted identifiers")

    actor_role = receipt.get("actor_role")
    authorizer_role = receipt.get("authorizer_role")
    if actor_role not in EXPECTED_ROLES:
        errors.append("actor_role is unknown")
    if authorizer_role not in EXPECTED_ROLES[:-1]:
        errors.append("authorizer_role is unknown or Agent/CI")
    if action is not None:
        executor_roles = action.get("executor_roles")
        authorizer_roles = action.get("authorizer_roles")
        if not isinstance(executor_roles, list) or actor_role not in executor_roles:
            errors.append("actor_role cannot execute the receipt action")
        if (
            not isinstance(authorizer_roles, list)
            or authorizer_role not in authorizer_roles
        ):
            errors.append("authorizer_role cannot authorize the receipt action")

    revision = receipt.get("project_or_source_revision")
    if not isinstance(revision, str) or not REVISION.fullmatch(revision):
        errors.append("project_or_source_revision is invalid")

    plan_digest = receipt.get("plan_digest_sha256")
    if plan_digest is not None and (
        not isinstance(plan_digest, str) or not SHA256.fullmatch(plan_digest)
    ):
        errors.append("plan_digest_sha256 must be null or lowercase SHA-256")
    artifacts = _unique_strings(
        receipt.get("artifact_digests_sha256"),
        "artifact_digests_sha256",
        errors,
        allow_empty=True,
    )
    if artifacts is not None:
        if len(artifacts) > 64 or any(
            not SHA256.fullmatch(item) for item in artifacts
        ):
            errors.append("artifact_digests_sha256 must contain lowercase SHA-256 values")
        if plan_digest is None and not artifacts:
            errors.append("authorization receipt requires a plan or artifact digest")
    if action is not None:
        required_bindings = action.get("required_bindings")
        if isinstance(required_bindings, list):
            if "plan-digest" in required_bindings and plan_digest is None:
                errors.append(f"{action_id} requires plan_digest_sha256")
            if "artifact-digests" in required_bindings and not artifacts:
                errors.append(f"{action_id} requires artifact_digests_sha256")

    if receipt.get("data_classification") not in DATA_CLASSIFICATIONS:
        errors.append("data_classification is unknown")
    if receipt.get("decision") not in DECISIONS:
        errors.append("decision is unknown")

    constraints = _unique_strings(
        receipt.get("constraints"), "constraints", errors, allow_empty=True
    )
    if constraints is not None:
        if len(constraints) > 32 or any(
            not CODE.fullmatch(item) for item in constraints
        ):
            errors.append("constraints must contain bounded reason-code tokens")
    reason_code = receipt.get("reason_code")
    if not isinstance(reason_code, str) or not CODE.fullmatch(reason_code):
        errors.append("reason_code is invalid")

    issued = _parse_timestamp(receipt.get("issued_at"), "issued_at", errors)
    expires = _parse_timestamp(receipt.get("expires_at"), "expires_at", errors)
    if issued is not None and expires is not None and expires <= issued:
        errors.append("expires_at must be later than issued_at")

    evidence = _unique_strings(receipt.get("evidence_refs"), "evidence_refs", errors)
    if evidence is not None:
        if len(evidence) > 32 or any(
            not EVIDENCE_REF.fullmatch(item) for item in evidence
        ):
            errors.append("evidence_refs must contain bounded redacted references")
        if any(item.startswith(("/", "file:")) or "\\" in item for item in evidence):
            errors.append("evidence_refs must not contain machine-absolute paths")
    return errors


def validate_receipt_schema(policy: dict[str, Any], schema: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    _exact_keys(schema, SCHEMA_KEYS, "authorization receipt schema", errors)
    if schema.get("$schema") != SCHEMA_DRAFT:
        errors.append("authorization receipt schema must declare Draft 2020-12")
    if schema.get("$id") != SCHEMA_ID:
        errors.append("authorization receipt schema $id is invalid")
    if schema.get("type") != "object" or schema.get("additionalProperties") is not False:
        errors.append("authorization receipt schema must be a closed object")
    required = schema.get("required")
    if not isinstance(required, list) or tuple(required) != RECEIPT_FIELDS:
        errors.append("authorization receipt schema must require every v1 field in order")
    properties = schema.get("properties")
    if not isinstance(properties, dict) or tuple(properties) != RECEIPT_FIELDS:
        errors.append("authorization receipt schema properties must match the closed v1 fields")
    expected_properties = {
        "schema_version": {"const": "1.0"},
        "record_type": {"const": "qiongli-authorization-receipt"},
        "authorization_id": {
            "type": "string",
            "pattern": AUTHORIZATION_ID.pattern,
        },
        "action": {"type": "string", "enum": list(EXPECTED_ACTIONS)},
        "object_scope": {
            "type": "array",
            "minItems": 1,
            "maxItems": 32,
            "uniqueItems": True,
            "items": {"type": "string", "pattern": IDENTIFIER.pattern},
        },
        "actor_role": {"type": "string", "enum": list(EXPECTED_ROLES)},
        "authorizer_role": {
            "type": "string",
            "enum": list(EXPECTED_ROLES[:-1]),
        },
        "project_or_source_revision": {
            "type": "string",
            "pattern": REVISION.pattern,
        },
        "plan_digest_sha256": {
            "type": ["string", "null"],
            "pattern": SHA256.pattern,
        },
        "artifact_digests_sha256": {
            "type": "array",
            "maxItems": 64,
            "uniqueItems": True,
            "items": {"type": "string", "pattern": SHA256.pattern},
        },
        "data_classification": {
            "type": "string",
            "enum": list(DATA_CLASSIFICATIONS),
        },
        "decision": {"type": "string", "enum": list(DECISIONS)},
        "constraints": {
            "type": "array",
            "maxItems": 32,
            "uniqueItems": True,
            "items": {"type": "string", "pattern": CODE.pattern},
        },
        "reason_code": {"type": "string", "pattern": CODE.pattern},
        "issued_at": {"type": "string", "format": "date-time"},
        "expires_at": {"type": "string", "format": "date-time"},
        "evidence_refs": {
            "type": "array",
            "minItems": 1,
            "maxItems": 32,
            "uniqueItems": True,
            "items": {"type": "string", "pattern": EVIDENCE_REF.pattern},
        },
    }
    if properties != expected_properties:
        errors.append("authorization receipt schema property constraints are invalid")

    if schema.get("allOf") != EXPECTED_DIGEST_RULE:
        errors.append("receipt schema must require a plan or artifact digest")

    examples = schema.get("examples")
    if not isinstance(examples, list) or len(examples) != 1:
        errors.append("authorization receipt schema must contain one redacted example")
    else:
        errors.extend(
            f"example: {error}" for error in validate_receipt(policy, examples[0])
        )
    return errors


def validate_policy(
    repo_root: Path,
    policy: dict[str, Any],
    schema: dict[str, Any],
    review_policy: dict[str, Any] | None = None,
    codeowners_content: str | None = None,
) -> list[str]:
    errors = [
        *validate_policy_document(repo_root, policy),
        *validate_receipt_schema(policy, schema),
    ]
    if (review_policy is None) != (codeowners_content is None):
        errors.append("review policy and CODEOWNERS must be validated together")
    elif review_policy is not None and codeowners_content is not None:
        errors.extend(
            validate_review_policy(repo_root, review_policy, codeowners_content)
        )
    return errors


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Validate Qiongli authorization, receipt, review, and delivery policy."
        )
    )
    parser.add_argument("--policy", type=Path, default=DEFAULT_POLICY)
    parser.add_argument("--schema", type=Path, default=DEFAULT_SCHEMA)
    parser.add_argument(
        "--review-policy", type=Path, default=DEFAULT_REVIEW_POLICY
    )
    parser.add_argument("--codeowners", type=Path, default=DEFAULT_CODEOWNERS)
    parser.add_argument(
        "--delivery-checklists", type=Path, default=DEFAULT_DELIVERY_CHECKLISTS
    )
    parser.add_argument("--pr-template", type=Path, default=DEFAULT_PR_TEMPLATE)
    args = parser.parse_args(argv)
    try:
        policy = load_document(args.policy)
        schema = load_document(args.schema)
        review_policy = load_document(args.review_policy)
        codeowners_content = load_text(args.codeowners)
        delivery_checklist_content = load_text(args.delivery_checklists)
        pr_template_content = load_text(args.pr_template)
    except AuthorizationPolicyError as error:
        print(f"[authorization-policy] {error}", file=sys.stderr)
        return 2
    errors = [
        *validate_policy(
            REPO_ROOT,
            policy,
            schema,
            review_policy,
            codeowners_content,
        ),
        *validate_delivery_documents(
            delivery_checklist_content,
            pr_template_content,
        ),
    ]
    if errors:
        for error in errors:
            print(f"[authorization-policy] FAIL: {error}", file=sys.stderr)
        print(
            f"[authorization-policy] {len(errors)} validation error(s)",
            file=sys.stderr,
        )
        return 1
    print(
        "[authorization-policy] PASS: 3 planes, 8 roles, 12 actions, "
        "13 non-transitive rules, 1 redacted receipt schema, 6 review domains, "
        "1 history policy, "
        "delivery boundaries, and 1 PR template"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
