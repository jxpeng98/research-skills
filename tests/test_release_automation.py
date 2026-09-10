from __future__ import annotations

import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)
RELEASE_AUTOMATION = LAYOUT.scripts / "release_automation.sh"
RELEASE_READY = LAYOUT.scripts / "release_ready.sh"
RELEASE_PREFLIGHT = LAYOUT.scripts / "release_preflight.sh"
RELEASE_POSTFLIGHT = LAYOUT.scripts / "release_postflight.sh"
RELEASE_LOCAL_INSTALL_CHECK = LAYOUT.scripts / "release_local_install_check.py"
BETA_SMOKE = REPO_ROOT / "tooling" / "scripts" / "run_beta_smoke.sh"
PYPI_PREFLIGHT = LAYOUT.scripts / "pypi_preflight.sh"
RELEASE_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "release-automation.yml"
CI_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "ci.yml"
INSTALL_CHECK_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "install-check.yml"
MACOS_INSTALL_CHECK_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "install-check-macos.yml"
AUTO_RERUN_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "auto-rerun-failed-actions.yml"
PUBLISH_PYPI_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "publish-pypi.yml"
PUBLISH_TESTPYPI_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "publish-testpypi.yml"
PUBLISH_NPM_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "publish-npm.yml"
VERIFY_RELEASE_TAG = LAYOUT.scripts / "verify_release_tag_version.sh"
CHANGELOG_SECTION = LAYOUT.scripts / "changelog_section.py"
RELEASE_AUTOMATION_DOC = REPO_ROOT / "tooling" / "release" / "automation.md"
PUBLISH_PYPI_DOC = REPO_ROOT / "docs" / "advanced" / "publish-pypi.md"
PUBLISH_PYPI_ZH_DOC = REPO_ROOT / "docs" / "zh" / "advanced" / "publish-pypi.md"
RELEASE_BRANCH_POLICY_DOC = REPO_ROOT / "docs" / "maintainer" / "release-branch-policy.md"
RELEASE_BRANCH_POLICY_ZH_DOC = REPO_ROOT / "docs" / "zh" / "maintainer" / "release-branch-policy.md"
ACCEPTANCE_TEMPLATE = REPO_ROOT / "tooling" / "release" / "templates" / "beta-acceptance-template.md"
WINDOWS_A1_ACCEPTANCE = REPO_ROOT / "tooling" / "scripts" / "windows_a1_acceptance.ps1"


class ReleaseAutomationTests(unittest.TestCase):
    def test_release_automation_script_supports_publish_mode(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        self.assertIn("<pre|post|publish>", content)
        self.assertIn("publish --version 0.1.0", content)
        self.assertIn("publish --tag v0.1.0", content)
        self.assertIn('./scripts/release_ready.sh "${release_ready_args[@]}"', content)
        self.assertIn("publish mode requires --version or --tag", content)
        self.assertIn('version_from_tag="$2"', content)
        self.assertIn('repo_tag_from_version="$(normalize_field "$version" repo_version)"', content)
        self.assertIn('repo_tag_from_tag="$(normalize_field "$version_from_tag" repo_version)"', content)
        self.assertIn('if [[ -n "$version" && -n "$version_from_tag" && "$repo_tag_from_version" != "$repo_tag_from_tag" ]]; then', content)
        self.assertIn("--maintainer-smoke", content)
        self.assertIn("git add CHANGELOG.md", content)
        self.assertIn('git add "tooling/release/${repo_tag}.md"', content)
        self.assertIn("README.md", content)
        self.assertIn("README_CN.md", content)
        self.assertIn("docs/index.md", content)
        self.assertIn("docs/zh/index.md", content)
        self.assertIn("docs/guide/install.md", content)
        self.assertIn("docs/zh/guide/install.md", content)
        self.assertIn('git tag -a "$repo_tag"', content)
        self.assertIn('git push "$push_remote" "$push_branch"', content)
        self.assertIn('git push "$push_remote" "$repo_tag"', content)
        self.assertNotIn('git push "$push_remote" "$push_branch" "$repo_tag"', content)
        self.assertIn('acceptance_out="tooling/release/acceptance/${repo_tag}-receipt.md"', content)
        self.assertIn('./scripts/release_postflight.sh --tag "$repo_tag" --acceptance-out "$acceptance_out"', content)
        self.assertIn('git add "$acceptance_out"', content)
        self.assertIn('chore: record release ${repo_tag} acceptance', content)
        self.assertIn('git push "$push_remote" "$push_branch"', content)
        self.assertIn('content/distribution/plugins.yaml', content)
        self.assertIn('tooling/scripts/build_plugin_artifacts.py', content)
        self.assertIn('tooling/scripts/materialize_distribution_payloads.py', content)
        self.assertNotIn('packages/qiongli-plugin/.codex-plugin/plugin.json', content)
        self.assertNotIn('packages/qiongli-next-plugin', content)
        self.assertIn('content/workflow/SKILL.md', content)
        self.assertIn('content/workflow/VERSION', content)
        self.assertIn('content/skills/registry.yaml', content)
        self.assertIn('docs/reference/skills.md', content)
        self.assertIn('docs/zh/reference/skills.md', content)
        self.assertIn('uv.lock', content)
        self.assertNotIn('qiongli-workflow/VERSION', content)
        self.assertNotIn('qiongli-workflow/skills/registry.yaml', content)
        self.assertNotIn('      skills/registry.yaml \\', content)
        self.assertIn('packages/npm-qiongli', content)
        self.assertIn('package-lock.json', content)
        self.assertIn('npm_preflight.sh', content)
        self.assertIn('release_ready_args=(--version "$version_input")', content)
        self.assertNotIn("python3 scripts/materialize_distribution_payloads.py --target all --in-place", content)
        self.assertIn('./scripts/release_postflight.sh --tag "$repo_tag"', content)

    def test_docs_define_optional_beta_channel_policy(self) -> None:
        docs = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (
                RELEASE_AUTOMATION_DOC,
                PUBLISH_PYPI_DOC,
                PUBLISH_PYPI_ZH_DOC,
                RELEASE_BRANCH_POLICY_DOC,
                RELEASE_BRANCH_POLICY_ZH_DOC,
            )
        )

        self.assertIn("Beta releases are optional validation releases", docs)
        self.assertIn("Beta channel policy", docs)
        self.assertIn("Beta 通道策略", docs)
        self.assertIn("beta 不是每个 stable release 的必经步骤", docs)
        self.assertIn("npm `latest` advances", docs)
        self.assertIn("npm `next` remains on the previous beta", docs)
        self.assertIn("不要为了移动 `next` 而机械发 beta", docs)
        self.assertIn("before tag creation", docs)
        self.assertIn("创建 tag 前", docs)
        self.assertIn("--resume-after-ready", docs)

    def test_beta_smoke_uses_usable_rg_fallback(self) -> None:
        content = BETA_SMOKE.read_text(encoding="utf-8")

        self.assertIn("find_usable_rg()", content)
        self.assertIn("type -P -a rg", content)
        self.assertIn('"$candidate" --version >/dev/null 2>&1', content)
        self.assertIn('RG_BIN="$(find_usable_rg || true)"', content)
        self.assertIn('grep -q "$pattern"', content)
        self.assertNotIn("command -v rg >/dev/null 2>&1", content)

    def test_publish_mode_allows_beta_release_from_dev_only(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        self.assertIn('DEV_PRERELEASE_BRANCH="dev"', content)
        self.assertIn('release_branch="$primary_branch"', content)
        self.assertIn('if is_prerelease_tag "$repo_tag" && [[ "$current_branch" == "$DEV_PRERELEASE_BRANCH" ]]; then', content)
        self.assertIn('release_branch="$DEV_PRERELEASE_BRANCH"', content)
        self.assertIn('Current branch: $current_branch; push branch: $push_branch; expected release branch: $release_branch', content)
        self.assertIn('release_line="$(normalize_field "$version_input" release_line)"', content)
        self.assertIn('source_branch="$(normalize_field "$version_input" source_branch)"', content)
        self.assertIn('if [[ "$release_line" == "native-2x" ]]; then', content)
        self.assertIn("RLS-201/PKG gate: native", content)

    def test_publish_mode_uses_release_ready_staging_before_commit_and_tag(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        release_ready = '"${release_ready_args[@]}"'
        git_add = "git add \\"
        tag = 'git tag -a "$repo_tag"'

        self.assertNotIn("sync_generated_distribution_payloads", content)
        self.assertNotIn("materialize_distribution_payloads.py --target all --in-place", content)
        self.assertIn('release_ready_args=(--version "$version_input")', content)
        self.assertIn(release_ready, content)
        self.assertLess(content.index(release_ready), content.index(git_add))
        self.assertLess(content.index(release_ready), content.index(tag))

    def test_publish_mode_gates_tag_publish_on_branch_checks(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        branch_push = 'git push "$push_remote" "$push_branch"'
        branch_gate = 'wait_for_required_workflows "$repo_slug" "$push_branch" "$release_commit" "$ci_timeout_seconds" "$ci_poll_interval_seconds" "${BRANCH_REQUIRED_WORKFLOWS[@]}"'
        tag_create = 'git tag -a "$repo_tag" -m "$tag_message"'
        tag_push = 'git push "$push_remote" "$repo_tag"'
        postflight = './scripts/release_postflight.sh --tag "$repo_tag" --acceptance-out "$acceptance_out"'

        self.assertIn('BRANCH_REQUIRED_WORKFLOWS=("CI" "Checkout Install Check")', content)
        self.assertIn('release_commit="$(git rev-parse HEAD)"', content)
        self.assertIn('repo_slug="$(derive_repo_slug || true)"', content)
        self.assertIn(branch_push, content)
        self.assertIn(branch_gate, content)
        self.assertIn(tag_create, content)
        self.assertIn(tag_push, content)
        self.assertIn(postflight, content)
        self.assertIn('publish mode cannot skip CI status checks before tag creation', content)
        self.assertIn('publish mode cannot disable CI waiting before tag creation', content)
        self.assertIn('publish mode requires hard CI timeout mode before tag creation', content)
        self.assertLess(content.index(branch_push), content.index(branch_gate))
        self.assertLess(content.index(branch_gate), content.index(tag_create))
        self.assertLess(content.index(tag_create), content.index(tag_push))
        self.assertLess(content.index(tag_push), content.index(postflight))

    def test_publish_mode_can_resume_after_release_ready(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        release_ready = './scripts/release_ready.sh "${release_ready_args[@]}"'
        release_commit = 'release_commit="$(git rev-parse HEAD)"'
        branch_push = 'git push "$push_remote" "$push_branch"'
        tag_create = 'git tag -a "$repo_tag" -m "$tag_message"'
        postflight = './scripts/release_postflight.sh --tag "$repo_tag" --acceptance-out "$acceptance_out"'

        self.assertIn("--resume-after-ready", content)
        self.assertIn("resume_after_ready=0", content)
        self.assertIn('if [[ "$resume_after_ready" -eq 0 ]]; then', content)
        self.assertIn("ensure_clean_resume_worktree", content)
        self.assertIn('git status --porcelain --untracked-files=normal', content)
        self.assertNotIn('if ! git diff --quiet || ! git diff --cached --quiet; then', content)
        self.assertIn('echo "[release-automation] resuming after release_ready; skipping preflight and release-prep commit"', content)
        self.assertIn('ensure_tag_matches_release_commit "$repo_tag" "$release_commit" "$push_remote"', content)
        self.assertIn('local_tag_target="$(resolve_local_tag_target "$repo_tag")"', content)
        self.assertIn('remote_tag_target="$(resolve_remote_tag_target "$push_remote" "$repo_tag")"', content)
        self.assertIn('echo "[release-automation] tag already exists remotely at release commit; skipping tag push"', content)
        self.assertLess(content.index(release_ready), content.index(release_commit))
        self.assertLess(content.index(release_commit), content.index(branch_push))
        self.assertLess(content.index(branch_push), content.index(tag_create))
        self.assertLess(content.index(tag_create), content.index(postflight))

    def test_release_postflight_waits_for_branch_and_tag_workflows(self) -> None:
        content = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")

        self.assertIn('BRANCH_REQUIRED_WORKFLOWS=("CI" "Checkout Install Check")', content)
        self.assertIn('TAG_REQUIRED_WORKFLOWS=("Publish to PyPI" "Publish to npm")', content)
        self.assertNotIn('REQUIRED_WORKFLOWS=("CI" "Install Check")', content)
        self.assertIn("--wait-ci", content)
        self.assertIn("query_actions_status", content)
        self.assertIn('ci_json_file="$(mktemp)"', content)
        self.assertNotIn("CI_JSON_PAYLOAD=", content)
        self.assertIn('observed = sorted({r.get("name") or "unknown" for r in runs if r.get("head_sha") == commit})', content)
        self.assertIn('labels.append("observed=" + ",".join(observed))', content)
        self.assertIn('query_actions_status "$REPO_SLUG" "$RELEASE_BRANCH" "$LOCAL_TAG_COMMIT" "${BRANCH_REQUIRED_WORKFLOWS[@]}"', content)
        self.assertIn('query_actions_status "$REPO_SLUG" "$TAG" "$LOCAL_TAG_COMMIT" "${TAG_REQUIRED_WORKFLOWS[@]}"', content)
        self.assertIn('CI_STATUS="success:branch-and-tag"', content)
        self.assertIn('refs/remotes/origin/$branch', content)
        self.assertIn('refresh_branch_ref "$RELEASE_BRANCH" "$RELEASE_BRANCH_REF"', content)
        self.assertIn('git fetch --force --no-tags origin "$fetch_ref"', content)
        self.assertIn("python3 scripts/generate_stable_release_notes.py \\", content)
        self.assertIn('--repo "${REPO_SLUG:-jxpeng98/qiongli}" \\', content)
        self.assertIn('--output "$TEMP_RELEASE_NOTES"', content)
        self.assertIn('RELEASE_NOTES_LABEL="stable notes: CHANGELOG.md [${version}] + download guide"', content)
        self.assertIn('POSTFLIGHT_STAGING_DIR=""', content)
        self.assertIn('python3 scripts/materialize_distribution_payloads.py --target all --out "$POSTFLIGHT_STAGING_DIR" --force', content)
        self.assertIn('bash ./scripts/verify_release_tag_version.sh --root "$POSTFLIGHT_STAGING_DIR" --tag "$TAG"', content)
        self.assertIn("gh release view", content)
        self.assertIn("--prerelease", content)
        self.assertIn('scripts/build_plugin_artifacts.py --root "$POSTFLIGHT_STAGING_DIR" --tag "$TAG" --dist-dir dist', content)
        self.assertIn("python3 scripts/build_literature_mcpb.py --dist-dir dist >/dev/null", content)
        self.assertIn("python3 scripts/generate_release_downloads.py --tag \"$TAG\" --out-dir dist", content)
        self.assertIn('UPLOAD_ASSETS_FILE=""', content)
        self.assertIn('python3 scripts/release_upload_assets.py --tag "$TAG" --dist-dir dist >"$UPLOAD_ASSETS_FILE"', content)
        self.assertIn('mapfile -t PLUGIN_ARTIFACTS <"$UPLOAD_ASSETS_FILE"', content)
        self.assertNotIn('if [[ "${TAG#v}" == *-* ]]; then\n  PLUGIN_ARTIFACTS=(', content)
        self.assertNotIn('"dist/qiongli-core-codex-plugin-${TAG}.tar.gz"', content)
        self.assertIn('gh release upload "$TAG" --repo "$REPO_SLUG" --clobber "${PLUGIN_ARTIFACTS[@]}"', content)
        self.assertIn('release_args+=("${PLUGIN_ARTIFACTS[@]}")', content)
        native_gate = 'if [[ "$RELEASE_LINE" == "native-2x" ]]; then'
        materialize = 'python3 scripts/materialize_distribution_payloads.py --target all --out "$POSTFLIGHT_STAGING_DIR" --force'
        self.assertIn(native_gate, content)
        self.assertIn("no materialization, dist-ref update, asset upload, or GitHub release mutation was attempted", content)
        self.assertLess(content.index(native_gate), content.index(materialize))

    def test_release_postflight_uploads_zotero_companion(self) -> None:
        content = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")

        build_companion = """python3 scripts/build_zotero_companion.py \\
  --dist-dir dist \\
  --release-tag "$TAG" \\
  --repo "$REPO_SLUG" >/dev/null"""
        self.assertIn(build_companion, content)
        self.assertIn('python3 scripts/release_upload_assets.py --tag "$TAG" --dist-dir dist >"$UPLOAD_ASSETS_FILE"', content)
        self.assertLess(
            content.index(build_companion),
            content.index('python3 scripts/release_upload_assets.py --tag "$TAG" --dist-dir dist >"$UPLOAD_ASSETS_FILE"'),
        )

    def test_release_postflight_injects_subject_runtime_evidence_into_acceptance_receipt(self) -> None:
        content = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")
        template = ACCEPTANCE_TEMPLATE.read_text(encoding="utf-8")

        evidence = 'python3 scripts/release_acceptance_evidence.py --root "$ROOT_DIR" --out "$ACCEPTANCE_EVIDENCE_FILE"'
        template_write = 'python3 - "$TEMPLATE_PATH" "$ACCEPTANCE_OUT" "$TAG" "$RELEASE_DATE" "$LOCAL_TAG_COMMIT" "$CI_STATUS" "$ACCEPTANCE_EVIDENCE_FILE" "$DOWNLOAD_INDEX"'

        self.assertIn("{{SUBJECT_RUNTIME_EVIDENCE}}", template)
        self.assertIn("{{COMPONENT_VERSION_MAP}}", template)
        self.assertIn('ACCEPTANCE_EVIDENCE_FILE=""', content)
        self.assertIn('rm -f "$ACCEPTANCE_EVIDENCE_FILE"', content)
        self.assertIn(evidence, content)
        self.assertIn(template_write, content)
        self.assertIn('subject_runtime_evidence = evidence.read_text(encoding="utf-8")', content)
        self.assertIn('release_index.get("component_versions")', content)
        self.assertIn('.replace("{{COMPONENT_VERSION_MAP}}", component_version_map)', content)
        self.assertIn('.replace("{{SUBJECT_RUNTIME_EVIDENCE}}", subject_runtime_evidence)', content)
        self.assertLess(content.index(evidence), content.index(template_write))

    def test_release_postflight_guards_generic_platform_dist_refs(self) -> None:
        content = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")

        self.assertIn("publish_plugin_dist_refs()", content)
        self.assertIn('codex_slug="qiongli"', content)
        self.assertIn('codex_slug="qiongli-next"', content)
        self.assertIn('claude_slug="qiongli"', content)
        self.assertIn('claude_slug="qiongli-next"', content)
        self.assertIn('node scripts/publish-codex-dist-ref.mjs \\', content)
        self.assertIn('--channel "$channel" \\', content)
        self.assertIn('--version "${TAG#v}" \\', content)
        self.assertIn('--slug "$platform_slug" \\', content)
        self.assertIn('--source "$platform_source"', content)
        self.assertIn('! -name "qiongli-workflow"', content)
        self.assertIn('publish_plugin_dist_refs "$TAG"', content)
        self.assertIn("native_plugin_dist_ref_policy()", content)
        self.assertIn('NATIVE_PLUGIN_DIST_REF_POLICY="$(native_plugin_dist_ref_policy "$TAG")"', content)
        self.assertIn('if [[ "$NATIVE_PLUGIN_DIST_REF_POLICY" == "multi-target" ]]; then', content)
        self.assertIn("generic plugin dist refs skipped: native policy is", content)
        self.assertLess(
            content.index('python3 scripts/build_plugin_artifacts.py --root "$POSTFLIGHT_STAGING_DIR" --tag "$TAG" --dist-dir dist'),
            content.index('NATIVE_PLUGIN_DIST_REF_POLICY="$(native_plugin_dist_ref_policy "$TAG")"'),
        )
        self.assertLess(
            content.index('publish_plugin_dist_refs "$TAG"'),
            content.index('gh release upload "$TAG" --repo "$REPO_SLUG" --clobber "${PLUGIN_ARTIFACTS[@]}"'),
        )

    def test_checkout_install_check_runs_all_platforms_on_push_and_pr(self) -> None:
        main_content = INSTALL_CHECK_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("name: Legacy Checkout Install Check", main_content)
        self.assertIn("push:", main_content)
        self.assertIn("pull_request:", main_content)
        self.assertIn("workflow_dispatch:", main_content)
        self.assertIn('branches: ["main", "master", "dev"]', main_content)
        self.assertNotIn('branches: ["2.x"]', main_content)
        self.assertIn("os: [ubuntu-latest, macos-latest]", main_content)
        self.assertIn("runs-on: windows-latest", main_content)

        self.assertFalse(
            MACOS_INSTALL_CHECK_WORKFLOW.exists(),
            msg="macOS checkout checks should stay in the main workflow to avoid duplicate checks.",
        )

    def test_ci_runs_windows_a1_acceptance_against_the_built_artifact(self) -> None:
        workflow = CI_WORKFLOW.read_text(encoding="utf-8")
        windows_job = workflow.split("  rust-lite-mcp-windows:\n", 1)[1].split(
            "  cross-platform-tests:\n", 1
        )[0]
        acceptance = WINDOWS_A1_ACCEPTANCE.read_text(encoding="utf-8")

        self.assertIn("components: rustfmt, clippy", windows_job)
        self.assertIn("Run Windows provider-config ACL evidence", windows_job)
        self.assertIn("--lib windows_ -- --nocapture", windows_job)
        self.assertIn("Run legacy Node MCPB Windows tests", windows_job)
        self.assertIn("npm --prefix packages/qiongli-literature-mcpb test", windows_job)
        self.assertIn("Run built Windows artifact A1 acceptance", windows_job)
        self.assertIn("tooling/scripts/windows_a1_acceptance.ps1", windows_job)
        self.assertIn("windows-a1-acceptance.json", windows_job)
        self.assertIn("if: success()", windows_job)
        self.assertIn("if-no-files-found: error", windows_job)
        self.assertIn("Upload partial Windows diagnostics", windows_job)
        self.assertIn("if: failure()", windows_job)
        self.assertIn("qiongli-lite-mcp-windows-x86_64-partial", windows_job)
        self.assertIn("if-no-files-found: warn", windows_job)

        self.assertIn('acceptance = "qiongli_windows_a1_release_artifact"', acceptance)
        self.assertIn("$runningOnWindows =", acceptance)
        self.assertNotIn("$isWindows =", acceptance)
        self.assertIn('$saveJsonRpc -ceq "2.0" -and $saveResponseId -eq 1', acceptance)
        self.assertIn('$statusJsonRpc -ceq "2.0" -and $statusResponseId -eq 2', acceptance)
        self.assertIn(
            '$redactedApiKeyProperty = $redactedFields.PSObject.Properties["api_key"]',
            acceptance,
        )
        self.assertIn('$null -eq $redactedApiKeyProperty', acceptance)
        self.assertNotIn('$redactedApiKey -ceq "configured"', acceptance)
        self.assertIn('GetEnvironmentVariable("GITHUB_SHA")', acceptance)
        self.assertIn("AreAccessRulesProtected", acceptance)
        self.assertIn("owner_is_current_user", acceptance)
        self.assertIn("current_user_full_control_only", acceptance)
        self.assertIn("canary_redacted", acceptance)
        self.assertIn("$null -eq $originalValue", acceptance)
        self.assertIn("[System.Management.Automation.Language.NullString]::Value", acceptance)
        self.assertNotIn("[object]::Equals($restoredValue", acceptance)
        self.assertIn("temporary_config_removed", acceptance)

    def test_failed_ci_and_checkout_runs_are_rerun_once(self) -> None:
        content = AUTO_RERUN_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("name: Auto Rerun Failed Actions", content)
        self.assertIn("workflow_run:", content)
        self.assertIn("workflows:", content)
        self.assertIn("- CI", content)
        self.assertIn("- Checkout Install Check", content)
        self.assertNotIn("- Release Automation", content)
        self.assertNotIn("- Publish to PyPI", content)
        self.assertIn("actions: write", content)
        self.assertIn("github.event.workflow_run.conclusion == 'failure'", content)
        self.assertIn("github.event.workflow_run.run_attempt < 2", content)
        self.assertIn('gh run rerun "$RUN_ID" --repo "$REPO" --failed', content)

    def test_release_postflight_supports_soft_ci_timeout_and_gh_api_fallback(self) -> None:
        content = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")

        self.assertIn('CI_TIMEOUT_MODE="hard"', content)
        self.assertIn("--ci-timeout-mode <hard|soft>", content)
        self.assertIn('fetch_actions_runs()', content)
        self.assertIn('gh api "repos/${repo_slug}/actions/runs?branch=${ref_name}&per_page=20"', content)
        self.assertIn('curl -fsSL -H "Authorization: Bearer ${GH_TOKEN}" "$api_url"', content)
        self.assertIn('[[ "$CI_TIMEOUT_MODE" == "hard" || "$CI_TIMEOUT_MODE" == "soft" ]]', content)
        self.assertIn('CI_STATUS="pending:timeout-after-${CI_TIMEOUT_SECONDS}s"', content)
        self.assertIn('CI_STATUS="skipped:query-unavailable"', content)
        self.assertIn('if [[ "$CI_TIMEOUT_MODE" == "soft" ]]; then', content)

    def test_publish_mode_passes_ci_timeout_mode_to_postflight(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        self.assertIn('ci_timeout_mode="hard"', content)
        self.assertIn("--ci-timeout-mode", content)
        self.assertIn('post_args+=(--wait-ci --ci-timeout-seconds "$ci_timeout_seconds" --ci-timeout-mode hard --ci-poll-interval-seconds "$ci_poll_interval_seconds")', content)

    def test_release_postflight_accepts_beta_tags_reachable_from_dev(self) -> None:
        content = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")

        self.assertIn('DEV_PRERELEASE_BRANCH="dev"', content)
        self.assertIn('select_release_branch_ref()', content)
        self.assertIn('if is_prerelease_tag "$tag" && branch_ref="$(detect_branch_ref "$DEV_PRERELEASE_BRANCH")"; then', content)
        self.assertIn('RELEASE_BRANCH="${release_branch_record%%$\'\\t\'*}"', content)
        self.assertIn('refresh_branch_ref "$RELEASE_BRANCH" "$RELEASE_BRANCH_REF"', content)
        self.assertIn('git merge-base --is-ancestor "$LOCAL_TAG_COMMIT" "$RELEASE_BRANCH_REF"', content)
        self.assertIn('query_actions_status "$REPO_SLUG" "$RELEASE_BRANCH" "$LOCAL_TAG_COMMIT" "${BRANCH_REQUIRED_WORKFLOWS[@]}"', content)

    def test_release_ready_includes_plugin_distribution_versions(self) -> None:
        content = RELEASE_READY.read_text(encoding="utf-8")

        self.assertIn("README.md", content)
        self.assertIn("README_CN.md", content)
        self.assertIn("docs/index.md", content)
        self.assertIn("docs/zh/index.md", content)
        self.assertIn("docs/guide/install.md", content)
        self.assertIn("docs/zh/guide/install.md", content)
        self.assertIn('content/distribution/plugins.yaml', content)
        self.assertIn('tooling/scripts/build_plugin_artifacts.py', content)
        self.assertIn('tooling/scripts/materialize_distribution_payloads.py', content)
        self.assertNotIn('packages/qiongli-plugin/.codex-plugin/plugin.json', content)
        self.assertNotIn('packages/qiongli-next-plugin|packages/qiongli-next-plugin/*', content)
        self.assertIn('content/workflow/SKILL.md', content)
        self.assertIn('content/workflow/VERSION', content)
        self.assertIn('content/skills/registry.yaml', content)
        self.assertIn('packages/npm-qiongli/package.json', content)
        self.assertIn('package-lock.json', content)
        self.assertIn('uv.lock', content)
        self.assertIn('docs/reference/skills.md', content)
        self.assertIn('docs/zh/reference/skills.md', content)
        self.assertNotIn('qiongli-workflow/VERSION', content)
        self.assertNotIn('qiongli-workflow/VERSION|skills/registry.yaml', content)
        self.assertNotIn('skills/*)', content)
        self.assertNotIn('packages/python-qiongli/src/qiongli/payload|packages/python-qiongli/src/qiongli/payload/*', content)
        self.assertNotIn('plugins/qiongli/skills/qiongli-workflow|plugins/qiongli/skills/qiongli-workflow/*', content)
        self.assertNotIn('qiongli-workflow/skills/registry.yaml', content)

    def test_release_ready_updates_stable_download_sections_before_preflight(self) -> None:
        content = RELEASE_READY.read_text(encoding="utf-8")

        stable_guard = 'if [[ "$RELEASE_LINE" == "legacy-1x" ]] && ! is_prerelease_tag "$REPO_TAG"; then'
        updater = 'python3 scripts/update_stable_download_sections.py --tag "$REPO_TAG" --root "$ROOT_DIR"'
        preflight = './scripts/release_automation.sh pre "${PRE_ARGS[@]}" --materialize-out "$RELEASE_STAGING_DIR"'

        self.assertIn(stable_guard, content)
        self.assertIn(updater, content)
        self.assertIn(preflight, content)
        self.assertLess(content.index(updater), content.index(preflight))

    def test_release_ready_runs_package_preflights_from_staging_root(self) -> None:
        content = RELEASE_READY.read_text(encoding="utf-8")

        self.assertNotIn("python3 scripts/materialize_distribution_payloads.py --target next-plugin --in-place", content)
        preflight = './scripts/release_automation.sh pre "${PRE_ARGS[@]}" --materialize-out "$RELEASE_STAGING_DIR"'
        verify = 'bash ./scripts/verify_release_tag_version.sh --root "$VERIFY_ROOT" --tag "$REPO_TAG"'
        local_install = 'python3 scripts/release_local_install_check.py --root "$RELEASE_STAGING_DIR"'
        pypi = 'bash ./scripts/pypi_preflight.sh --root "$RELEASE_STAGING_DIR" "${PYPI_ARGS[@]}"'
        npm = 'bash ./scripts/npm_preflight.sh --root "$RELEASE_STAGING_DIR"'

        self.assertIn('RELEASE_STAGING_DIR=""', content)
        self.assertIn('--staging-dir <dir>', content)
        self.assertIn('mktemp -d "${TMPDIR:-/tmp}/qiongli-release-ready.XXXXXX"', content)
        self.assertIn(preflight, content)
        self.assertIn(verify, content)
        self.assertIn(local_install, content)
        self.assertIn(pypi, content)
        self.assertIn(npm, content)
        self.assertTrue(RELEASE_LOCAL_INSTALL_CHECK.is_file())
        self.assertLess(content.index(preflight), content.index(verify))
        self.assertLess(content.index(verify), content.index(local_install))
        self.assertLess(content.index(local_install), content.index(pypi))
        self.assertLess(content.index(pypi), content.index(npm))

    def test_release_ready_checks_experience_schema_compatibility(self) -> None:
        content = RELEASE_READY.read_text(encoding="utf-8")

        verify = 'bash ./scripts/verify_release_tag_version.sh --root "$VERIFY_ROOT" --tag "$REPO_TAG"'
        checker = 'python3 scripts/check_experience_schema_compatibility.py --root "$RELEASE_STAGING_DIR"'
        local_install = 'python3 scripts/release_local_install_check.py --root "$RELEASE_STAGING_DIR"'

        self.assertIn(checker, content)
        self.assertLess(content.index(verify), content.index(checker))
        self.assertLess(content.index(checker), content.index(local_install))

    def test_release_ready_does_not_print_manual_publish_steps(self) -> None:
        content = RELEASE_READY.read_text(encoding="utf-8")

        self.assertIn("prepare+verify completed; publish mode owns commit/tag/push", content)
        self.assertNotIn('git tag -a ${REPO_TAG}', content)
        self.assertNotIn('git push origin main --tags', content)

    def test_release_preflight_fails_fast_on_logged_stage_errors(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("run_logged_stage()", content)
        self.assertIn('statuses=("${PIPESTATUS[@]}")', content)
        self.assertIn('"[preflight] FAIL: ${label} failed with exit code ${command_status}"', content)
        self.assertIn('run_logged_stage "validator" "$validator_log" "${validate_cmd[@]}"', content)
        self.assertIn('run_logged_stage "unit tests" "$unit_log" python3 -m unittest discover -s tests -v', content)
        self.assertIn('run_logged_stage "smoke (${smoke_tier} tier)" "$smoke_log" ./scripts/run_beta_smoke.sh --tier "$smoke_tier"', content)

    def test_release_preflight_supports_quick_ci_gate(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("QUICK_MODE=0", content)
        self.assertIn("RUN_UNIT_TESTS=1", content)
        self.assertIn("RUN_CONTROLLER_EVALS=1", content)
        self.assertIn("--quick", content)
        self.assertIn("--skip-unit-tests", content)
        self.assertIn("--skip-controller-evals", content)
        self.assertIn('echo "[preflight] unit tests skipped"', content)
        self.assertIn('echo "[preflight] controller-mode evals skipped"', content)
        self.assertIn('unittest_summary="skipped"', content)

    def test_release_preflight_syncs_npm_payload_before_tests(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        materialize = 'python3 scripts/materialize_distribution_payloads.py --target all --out "$MATERIALIZE_OUT" --force'
        self.assertIn('echo "[preflight] materialize distribution payloads"', content)
        self.assertIn(materialize, content)
        self.assertIn('MATERIALIZE_OUT="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-release-preflight.XXXXXX")"', content)
        self.assertIn('if [[ "$MATERIALIZE_IN_PLACE" -eq 1 ]]; then', content)
        self.assertIn('echo "[preflight] sync skill reference docs"', content)
        self.assertIn("python3 scripts/generate_skill_docs.py", content)
        self.assertLess(
            content.index(materialize),
            content.index("python3 scripts/generate_skill_docs.py"),
        )
        self.assertLess(
            content.index("python3 scripts/generate_skill_docs.py"),
            content.index('run_logged_stage "validator" "$validator_log" "${validate_cmd[@]}"'),
        )
        self.assertLess(
            content.index("python3 scripts/generate_skill_docs.py"),
            content.index('run_logged_stage "unit tests" "$unit_log" python3 -m unittest discover -s tests -v'),
        )

    def test_release_preflight_validates_platform_target_registry_before_standard_validator(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        platform_registry_gate = 'python3 scripts/validate_platform_targets.py --root "$PREFLIGHT_ROOT"'
        standard_validator = 'run_logged_stage "validator" "$validator_log" "${validate_cmd[@]}"'

        self.assertIn('echo "[preflight] release target registries schema"', content)
        self.assertNotIn('echo "[preflight] platform target registry schema"', content)
        self.assertIn(platform_registry_gate, content)
        self.assertLess(content.index(platform_registry_gate), content.index(standard_validator))

    def test_release_preflight_validates_capability_contract_before_standard_validator(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        capability_gate = (
            'python3 scripts/validate_capability_contract.py --root "$PREFLIGHT_ROOT" --require-complete'
        )
        standard_validator = 'run_logged_stage "validator" "$validator_log" "${validate_cmd[@]}"'

        self.assertIn('echo "[preflight] capability contract v2"', content)
        self.assertIn(capability_gate, content)
        self.assertLess(content.index(capability_gate), content.index(standard_validator))

    def test_release_preflight_supports_staged_materialization_for_ci(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        staged_materialize = 'python3 scripts/materialize_distribution_payloads.py --target all --out "$MATERIALIZE_OUT" --force'
        staged_validate = 'validate_cmd=(python3 scripts/validate_research_standard.py --root "$PREFLIGHT_ROOT")'

        self.assertIn("--materialize-out <dir>", content)
        self.assertIn("--in-place", content)
        self.assertIn("MATERIALIZE_OUT=\"\"", content)
        self.assertIn("MATERIALIZE_IN_PLACE=0", content)
        self.assertIn("PREFLIGHT_ROOT=\"$ROOT_DIR\"", content)
        self.assertIn(staged_materialize, content)
        self.assertIn('PREFLIGHT_ROOT="$MATERIALIZE_OUT"', content)
        self.assertIn('if [[ "$MATERIALIZE_IN_PLACE" -eq 1 ]]; then', content)
        self.assertIn('[preflight] in-place materialization requires explicit --in-place', content)
        self.assertIn(staged_validate, content)
        self.assertLess(content.index(staged_materialize), content.index(staged_validate))

    def test_release_preflight_runs_controller_mode_evals_as_warning_stage(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("run_warning_stage()", content)
        self.assertIn('eval_log="$(mktemp -t qiongli-controller-evals.XXXXXX.log)"', content)
        self.assertIn(
            'run_warning_stage "controller-mode evals" "$eval_log" python3 scripts/run_controller_mode_evals.py evals/controller_modes',
            content,
        )
        self.assertIn('"[preflight] WARN: ${label} failed with exit code ${command_status}"', content)

    def test_release_preflight_preserves_stage_logs_on_failure(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("cleanup_logs()", content)
        self.assertIn('FAILED_STAGE=""', content)
        self.assertIn('FAILED_LOG=""', content)
        self.assertIn('FAILED_STATUS=""', content)
        self.assertIn('FAILED_STAGE="$label"', content)
        self.assertIn('FAILED_LOG="$log_file"', content)
        self.assertIn('FAILED_STATUS="$command_status"', content)
        self.assertIn('"[preflight] failure summary: ${FAILED_STAGE} exited with ${FAILED_STATUS}"', content)
        self.assertIn('tail -n 120 "$FAILED_LOG"', content)
        self.assertIn('local status="$?"', content)
        self.assertIn('if [[ "$status" -eq 0 ]]; then', content)
        self.assertNotIn('trap \'rm -f "$validator_log" "$unit_log" "$smoke_log"\' EXIT', content)

    def test_release_preflight_reports_missing_pyyaml_dependency(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn('require_python_module yaml PyYAML', content)
        self.assertIn("[preflight] missing Python dependency: ${package} (module: ${module})", content)
        self.assertIn("python3 -m pip install -e .", content)

    def test_release_preflight_does_not_print_manual_publish_steps(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("preflight completed; publish mode owns tag/push", content)
        self.assertNotIn('git tag -a $TAG', content)
        self.assertNotIn('git push origin $TAG', content)

    def test_pypi_preflight_checks_release_build_dependencies(self) -> None:
        content = PYPI_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("require_python_module build build", content)
        self.assertIn("require_python_module twine twine", content)
        self.assertIn("python3 -m pip install -e . build twine", content)

    def test_pypi_preflight_materializes_payloads_before_build(self) -> None:
        content = PYPI_PREFLIGHT.read_text(encoding="utf-8")

        materialize = 'python3 scripts/materialize_distribution_payloads.py --target all --out "$PREFLIGHT_ROOT" --force'
        build = "python3 -m build"

        self.assertIn("--root <dir>", content)
        self.assertIn('ROOT_DIR="$(cd "$2" && pwd)"', content)
        self.assertIn("--in-place", content)
        self.assertIn("PREFLIGHT_ROOT=\"\"", content)
        self.assertIn('PREFLIGHT_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-pypi-preflight-root.XXXXXX")"', content)
        self.assertIn('if [[ "$PREFLIGHT_IN_PLACE" -eq 1 ]]; then', content)
        self.assertIn(materialize, content)
        self.assertIn(build, content)
        self.assertIn('cd "$PREFLIGHT_ROOT"', content)
        self.assertLess(content.index(materialize), content.index(build))

    def test_npm_preflight_accepts_staging_root(self) -> None:
        content = (LAYOUT.scripts / "npm_preflight.sh").read_text(encoding="utf-8")

        self.assertIn("--root <dir>", content)
        self.assertIn('ROOT_DIR="$(cd "$2" && pwd)"', content)
        self.assertIn("--in-place", content)
        self.assertIn("PREFLIGHT_ROOT=\"\"", content)
        self.assertIn('PREFLIGHT_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-npm-preflight-root.XXXXXX")"', content)
        self.assertIn('if [[ "$PREFLIGHT_IN_PLACE" -eq 1 ]]; then', content)
        self.assertIn('PKG_DIR="$ROOT_DIR/packages/npm-qiongli"', content)
        self.assertIn('PKG_DIR="$PREFLIGHT_ROOT/packages/npm-qiongli"', content)
        self.assertIn('python3 scripts/materialize_distribution_payloads.py --target all --out "$PREFLIGHT_ROOT" --force', content)
        self.assertIn('cd "$PREFLIGHT_ROOT"', content)

    def test_pypi_preflight_does_not_print_manual_publish_steps(self) -> None:
        content = PYPI_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("package preflight completed; publish mode owns tag/release flow", content)
        self.assertNotIn("git tag v<version>", content)
        self.assertNotIn("Publish to TestPyPI", content)

    def test_changelog_section_script_extracts_versioned_sections(self) -> None:
        content = CHANGELOG_SECTION.read_text(encoding="utf-8")

        self.assertIn('HEADING_RE = re.compile(r"^## \\[(?P<version>[^\\]]+)\\](?P<suffix>.*)$")', content)
        self.assertIn('parser.add_argument("--version", required=True', content)
        self.assertIn('print(f"[changelog] missing version section: {args.version}"', content)
        self.assertIn('Path(args.output).write_text(section, encoding="utf-8")', content)

    def test_prerelease_note_generator_points_to_publish_mode(self) -> None:
        content = (LAYOUT.scripts / "generate_release_notes.sh").read_text(encoding="utf-8")

        self.assertIn('PUBLISH_CMD="./scripts/release_automation.sh publish --tag ${TAG} --skip-bump"', content)
        self.assertNotIn('PUBLISH_CMD="./scripts/release_automation.sh publish --version ${VERSION_HINT} --skip-bump"', content)
        self.assertIn('PUBLISH_CMD="${PUBLISH_CMD} --from-tag ${FROM_TAG}"', content)
        self.assertIn('${PUBLISH_CMD}', content)
        self.assertIn('release_ready.sh --version', content)
        self.assertNotIn('git push origin main --tags', content)

    @unittest.skipIf(os.name == "nt", "requires POSIX Bash release entrypoints")
    def test_native_note_generator_is_truthful_and_rejects_legacy_customization(self) -> None:
        manifest = (REPO_ROOT / "packages/qiongli-native/Cargo.toml").read_text(encoding="utf-8")
        version = re.search(
            r'(?ms)^\[workspace\.package\]\s*$.*?^version\s*=\s*"([^"]+)"',
            manifest,
        )
        self.assertIsNotNone(version)
        native_tag = f"v{version.group(1)}"
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "native-alpha.md"
            generated = subprocess.run(
                [
                    "bash",
                    "scripts/generate_release_notes.sh",
                    "--tag",
                    native_tag,
                    "--output",
                    str(output),
                ],
                cwd=REPO_ROOT,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            self.assertEqual(generated.returncode, 0, generated.stderr)
            content = output.read_text(encoding="utf-8")
            for token in (
                "Release Notes",
                "Stage: Alpha",
                "Validation Evidence",
                "Publish Steps",
                "rollback.md",
                "publication is not allowed",
            ):
                self.assertIn(token, content)

            rejected = subprocess.run(
                [
                    "bash",
                    "scripts/generate_release_notes.sh",
                    "--tag",
                    native_tag,
                    "--output",
                    str(Path(directory) / "custom.md"),
                    "--from-tag",
                    "v1.19.0-beta.1",
                ],
                cwd=REPO_ROOT,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            self.assertEqual(rejected.returncode, 2)
            self.assertIn("reject legacy", rejected.stderr)

    def test_release_workflow_separates_diagnostics_from_authorized_native_publication(self) -> None:
        content = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("workflow_dispatch:", content)
        self.assertNotIn("push:", content)
        self.assertNotIn('tags:\n      - "v*"', content)
        self.assertNotIn("- publish", content)
        self.assertIn("maintainer_smoke:", content)
        self.assertNotIn("      version:\n", content)
        self.assertNotIn('if [[ -n "${{ inputs.version }}" ]]; then', content)
        self.assertNotIn('elif [[ -n "$tag" ]]; then', content)
        self.assertIn('args+=(--tag "$tag")', content)
        self.assertNotIn("publish mode requires 'version' input", content)
        self.assertIn("fetch-depth: 0", content)
        self.assertIn('git fetch --force --prune origin +refs/heads/*:refs/remotes/origin/* +refs/tags/*:refs/tags/*', content)
        self.assertNotIn('if [[ "${{ github.event_name }}" == "push" ]]; then', content)
        self.assertNotIn('mode="post"', content)
        self.assertIn('args+=(--maintainer-smoke)', content)
        self.assertNotIn("if: ${{ github.event_name == 'push' || inputs.mode != 'publish' }}", content)
        self.assertNotIn('if [[ "${{ github.event_name }}" == "push" || "${{ inputs.mode }}" != "publish" ]]; then', content)
        self.assertNotIn('if [[ "$mode" == "publish"', content)
        self.assertIn('args+=(--create-release)', content)
        self.assertIn('bash scripts/verify_release_tag_version.sh --root "$RUNNER_TEMP/qiongli-release-dist" --tag "$tag"', content)
        self.assertIn('git config user.name "github-actions[bot]"', content)
        self.assertIn("python -m pip install -e . build twine", content)
        self.assertIn("./scripts/release_automation.sh \"$mode\" \"${args[@]}\"", content)
        self.assertIn("Classify release", content)
        self.assertIn('--print-field package_version', content)
        self.assertIn("native-release-dry-run:", content)
        self.assertIn("legacy-release-automation:", content)
        native_jobs, legacy_job = content.split("  legacy-release-automation:\n", 1)
        native_job, publication_job = native_jobs.split("  native-publish:\n", 1)
        self.assertIn("inputs.mode == 'post' && inputs.create_release", publication_job)
        self.assertIn("contents: write", publication_job)
        self.assertIn("actions: write", publication_job)
        self.assertIn('native_release_publish.py --tag "$RELEASE_TAG"', publication_job)
        self.assertIn("contents: read", native_job)
        self.assertIn("persist-credentials: false", native_job)
        self.assertIn("dtolnay/rust-toolchain@1.97.0", native_job)
        self.assertNotIn("GH_TOKEN", native_job)
        self.assertIn("contents: write", legacy_job)
        self.assertIn("GH_TOKEN", legacy_job)
        self.assertIn('if [[ "$GITHUB_REF_TYPE" != "branch"', native_job)
        self.assertNotIn('tag="${{ inputs.tag }}"', content)
        self.assertNotIn('args+=(--from-tag "${{ inputs.from_tag }}")', content)
        self.assertIn('--materialize-out "$RUNNER_TEMP/qiongli-native-release-plan"', content)
        self.assertIn("Upload native release dry-run bundle", content)

    def test_testpypi_workflow_is_legacy_branch_only(self) -> None:
        content = PUBLISH_TESTPYPI_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("github.ref_type == 'branch'", content)
        self.assertIn("github.ref_name == 'main'", content)
        self.assertIn("github.ref_name == 'dev'", content)
        self.assertIn("github.ref_name == 'release/1.x-python'", content)
        self.assertIn('--print-field release_line', content)
        self.assertIn('if [[ "$release_line" != "legacy-1x" ]]; then', content)
        self.assertIn("id-token: write", content)

    def test_publish_pypi_workflow_verifies_tag_matches_repo_version(self) -> None:
        content = PUBLISH_PYPI_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn('bash scripts/verify_release_tag_version.sh --root "$RUNNER_TEMP/qiongli-dist" --tag "${RELEASE_TAG}"', content)
        self.assertIn('packages-dir: ${{ runner.temp }}/qiongli-dist/dist', content)
        self.assertNotIn('bash scripts/verify_release_tag_version.sh --tag "${GITHUB_REF_NAME}"', content)

    def test_tag_publish_workflows_limit_manual_publication_to_native_tags(self) -> None:
        for workflow in (PUBLISH_PYPI_WORKFLOW, PUBLISH_NPM_WORKFLOW):
            with self.subTest(workflow=workflow.name):
                content = workflow.read_text(encoding="utf-8")

                self.assertIn("workflow_dispatch:", content)
                self.assertIn("inputs.publish_release && github.ref_type == 'tag'", content)
                self.assertIn("startsWith(github.ref_name, 'v2.')", content)
                self.assertNotIn("inputs.tag", content)
                self.assertIn("push:", content)
                self.assertIn('tags:\n      - "v*"', content)
                self.assertIn("ref: ${{ github.ref }}", content)
                self.assertIn("RELEASE_TAG: ${{ github.ref_name }}", content)
                self.assertIn('bash scripts/verify_release_tag_version.sh --root "$RUNNER_TEMP/qiongli-dist" --tag "${RELEASE_TAG}"', content)
                self.assertIn("if: ${{ github.event_name == 'push' && !startsWith(github.ref_name, 'v2.') }}", content)
                self.assertIn("types: [published]", content)
                self.assertIn("--require-ci", content)
                self.assertIn('release_line="$(python3 scripts/release_version.py "${RELEASE_TAG}" --print-field release_line)"', content)
                self.assertIn('if [[ "$release_line" == "native-2x" ]]; then', content)

    def test_tag_publish_workflows_materialize_staging_before_version_verify(self) -> None:
        for workflow in (PUBLISH_PYPI_WORKFLOW, PUBLISH_NPM_WORKFLOW):
            with self.subTest(workflow=workflow.name):
                content = workflow.read_text(encoding="utf-8")

                verify = 'bash scripts/verify_release_tag_version.sh --root "$RUNNER_TEMP/qiongli-dist" --tag "${RELEASE_TAG}"'
                install = "python -m pip install -e ."
                materialize = 'python3 scripts/materialize_distribution_payloads.py --target all --out "$RUNNER_TEMP/qiongli-dist" --force'
                self.assertIn(materialize, content)
                self.assertIn(install, content)
                self.assertNotIn("python3 scripts/materialize_distribution_payloads.py --target all --in-place", content)
                self.assertLess(content.index(install), content.index(materialize))
                self.assertLess(content.index(materialize), content.index(verify))

    def test_release_workflow_materializes_staging_before_version_verify(self) -> None:
        content = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        verify = 'bash scripts/verify_release_tag_version.sh --root "$RUNNER_TEMP/qiongli-release-dist" --tag "$tag"'
        install = "python -m pip install -e . build twine"
        materialize = 'python3 scripts/materialize_distribution_payloads.py --target all --out "$RUNNER_TEMP/qiongli-release-dist" --force'
        self.assertIn(materialize, content)
        self.assertIn(install, content)
        self.assertNotIn("python3 scripts/materialize_distribution_payloads.py --target all --in-place", content)
        self.assertLess(content.index(install), content.index(materialize))
        self.assertLess(content.index(materialize), content.index(verify))

    def test_verify_release_tag_script_checks_expected_files(self) -> None:
        content = VERIFY_RELEASE_TAG.read_text(encoding="utf-8")

        self.assertIn("--root <dir>", content)
        self.assertIn('ROOT_DIR="$(cd "$2" && pwd)"', content)
        self.assertIn('cd "$ROOT_DIR"', content)
        self.assertIn('"${QIONGLI_PYTHON:-python3}" scripts/release_version.py "$TAG" --print-field "$field"', content)
        self.assertIn('expected_package_version="$(release_field package_version)"', content)
        self.assertIn('expected_release_line="$(release_field release_line)"', content)
        self.assertIn('expected_channel="$(release_field channel)"', content)
        self.assertIn('if [[ "$expected_release_line" == "native-2x" ]]; then', content)
        self.assertIn('packages/qiongli-native/Cargo.toml', content)
        self.assertIn('packages/qiongli-native/Cargo.lock', content)
        self.assertIn('packages/qiongli-lite-mcp/Cargo.lock', content)
        self.assertIn('native workspace channel mismatch', content)
        self.assertIn('pyproject.toml', content)
        self.assertIn('packages/python-qiongli/src/qiongli/__init__.py', content)
        self.assertIn('content/skills/registry.yaml', content)
        self.assertIn('content/workflow/VERSION', content)
        self.assertNotIn('Path("skills/registry.yaml")', content)
        self.assertNotIn('< qiongli-workflow/VERSION', content)
        self.assertNotIn('actual_workflow_registry_version', content)
        self.assertNotIn('Path("qiongli-workflow/skills/registry.yaml")', content)
        self.assertNotIn('echo "[verify-release-tag] qiongli-workflow/skills/registry.yaml mismatch', content)
        self.assertIn('packages/python-qiongli/src/qiongli/payload/qiongli-workflow/VERSION', content)
        self.assertIn('packages/python-qiongli/src/qiongli/payload/qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('packages/python-qiongli/src/qiongli/payload/skills/registry.yaml', content)
        self.assertIn('packages/npm-qiongli/package.json', content)
        self.assertIn('package-lock.json', content)
        self.assertIn('packages/npm-qiongli/payload/qiongli-workflow/VERSION', content)
        self.assertIn('packages/npm-qiongli/payload/qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('packages/npm-qiongli/python-runtime/qiongli/__init__.py', content)
        self.assertIn('packages/npm-qiongli/python-runtime/skills/registry.yaml', content)
        self.assertIn('plugins/qiongli/.codex-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli/skills/qiongli-workflow/VERSION', content)
        self.assertIn('plugins/qiongli/skills/qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('plugins/qiongli-next/.codex-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli-next/.claude-plugin/plugin.json', content)
        self.assertIn('plugins/qiongli-next/skills/qiongli-workflow/VERSION', content)
        self.assertIn('plugins/qiongli-next/skills/qiongli-workflow/skills/registry.yaml', content)
        self.assertIn('plugins/qiongli/.claude-plugin/plugin.json', content)
        self.assertNotIn('plugins/qiongli/gemini-extension.json', content)
        self.assertIn('"${QIONGLI_PYTHON:-python3}" scripts/audit_distribution_payloads.py --root "$ROOT_DIR"', content)

    def test_native_preflight_uses_external_plan_and_native_cargo_gates(self) -> None:
        content = RELEASE_PREFLIGHT.read_text(encoding="utf-8")
        ready = RELEASE_READY.read_text(encoding="utf-8")

        self.assertIn('native --materialize-out must be outside the source tree', content)
        self.assertIn('native --staging-dir must be outside the source tree', ready)
        self.assertIn('RELEASE_STAGING_DIR="$(canonical_external_path "$RELEASE_STAGING_DIR")"', ready)
        self.assertIn('native preflight forbids --in-place', content)
        self.assertIn('cd packages/qiongli-native', content)
        self.assertIn('cargo fmt --all -- --check', content)
        self.assertIn('cargo clippy --workspace --all-targets --all-features --locked -- -D warnings', content)
        self.assertIn('cargo test --workspace --all-targets --all-features --locked', content)
        self.assertIn('python3 scripts/native_release_dry_run.py \\', content)
        self.assertIn('--out-dir "$MATERIALIZE_OUT" \\', content)
        self.assertIn('--source-ref "$native_source_ref"', content)
        self.assertIn('--source-ref-type "$native_source_ref_type"', content)
        self.assertIn('--worktree-state "$native_worktree_state"', content)
        self.assertIn('--source-commit "$(git rev-parse HEAD)"', content)
        self.assertIn('dry-run evidence will not bind source_commit', content)
        self.assertIn('--json', content)

    def test_native_publish_and_postflight_fail_before_mutating_commands(self) -> None:
        automation = RELEASE_AUTOMATION.read_text(encoding="utf-8")
        postflight = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")

        automation_gate = 'if [[ "$release_line" == "native-2x" ]]; then'
        automation_gate_index = automation.index(automation_gate)
        self.assertLess(
            automation_gate_index,
            automation.index("\n    ensure_git_identity\n", automation_gate_index),
        )
        self.assertLess(
            automation_gate_index,
            automation.index('git push "$push_remote" "$push_branch"', automation_gate_index),
        )
        postflight_gate = 'if [[ "$RELEASE_LINE" == "native-2x" ]]; then'
        self.assertLess(postflight.index(postflight_gate), postflight.index('POSTFLIGHT_STAGING_DIR="$(mktemp'))
        self.assertLess(postflight.index(postflight_gate), postflight.index('publish_plugin_dist_refs "$TAG"'))

    @unittest.skipIf(os.name == "nt", "requires POSIX Bash release entrypoints")
    def test_native_publish_gate_is_functional_and_does_not_change_head(self) -> None:
        before = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            check=True,
        ).stdout.strip()
        refs_before = subprocess.run(
            ["git", "for-each-ref", "--format=%(refname):%(objectname)"],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            check=True,
        ).stdout
        result = subprocess.run(
            [
                "bash",
                "scripts/release_automation.sh",
                "publish",
                "--tag",
                "v2.0.0-alpha.1",
            ],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        postflight = subprocess.run(
            [
                "bash",
                "scripts/release_postflight.sh",
                "--tag",
                "v2.0.0-alpha.1",
                "--skip-remote",
                "--skip-ci-status",
            ],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        after = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            check=True,
        ).stdout.strip()
        refs_after = subprocess.run(
            ["git", "for-each-ref", "--format=%(refname):%(objectname)"],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            check=True,
        ).stdout

        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("RLS-201/PKG gate", result.stderr)
        self.assertIn("no commit, push, or tag was created", result.stderr)
        self.assertEqual(postflight.returncode, 1, postflight.stderr)
        self.assertIn("RLS-201/PKG gate", postflight.stderr)
        self.assertIn("no materialization", postflight.stderr)
        self.assertEqual(after, before)
        self.assertEqual(refs_after, refs_before)

    @unittest.skipIf(os.name == "nt", "requires POSIX Bash release entrypoints")
    def test_native_tag_verifier_binds_cargo_version_channel_and_lock(self) -> None:
        manifest = (REPO_ROOT / "packages/qiongli-native/Cargo.toml").read_text(encoding="utf-8")
        version = re.search(
            r'(?ms)^\[workspace\.package\]\s*$.*?^version\s*=\s*"([^"]+)"',
            manifest,
        )
        self.assertIsNotNone(version)
        aligned = subprocess.run(
            [
                "bash",
                "scripts/verify_release_tag_version.sh",
                "--root",
                str(REPO_ROOT),
                "--tag",
                f"v{version.group(1)}",
            ],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        mismatch = subprocess.run(
            [
                "bash",
                "scripts/verify_release_tag_version.sh",
                "--root",
                str(REPO_ROOT),
                "--tag",
                "v2.0.0-alpha.9999",
            ],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

        self.assertEqual(aligned.returncode, 0, aligned.stderr)
        self.assertIn("workspace version, channel, and Cargo.lock are aligned", aligned.stdout)
        self.assertEqual(mismatch.returncode, 1, mismatch.stderr)
        self.assertIn("native workspace version mismatch", mismatch.stderr)

    def test_active_native_release_paths_have_no_alpha1_literals(self) -> None:
        result = subprocess.run(
            [sys.executable, "tooling/scripts/check_native_release_literals.py"],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("active release paths are version-generic", result.stdout)
        self.assertIn("historical fixtures=1", result.stdout)

    @unittest.skipIf(os.name == "nt", "requires POSIX Bash release entrypoints")
    def test_native_preflight_rejects_source_tree_output_before_write(self) -> None:
        forbidden = REPO_ROOT / ".rel201-forbidden-output"
        self.assertFalse(forbidden.exists())
        result = subprocess.run(
            [
                "bash",
                "scripts/release_preflight.sh",
                "--tag",
                "v2.0.0-alpha.987654",
                "--materialize-out",
                str(forbidden),
            ],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn("must be outside the source tree", result.stderr)
        self.assertFalse(forbidden.exists())


if __name__ == "__main__":
    unittest.main()
