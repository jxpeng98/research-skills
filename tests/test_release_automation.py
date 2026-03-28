from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
RELEASE_AUTOMATION = REPO_ROOT / "scripts" / "release_automation.sh"
RELEASE_POSTFLIGHT = REPO_ROOT / "scripts" / "release_postflight.sh"
RELEASE_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "release-automation.yml"


class ReleaseAutomationTests(unittest.TestCase):
    def test_release_automation_script_supports_publish_mode(self) -> None:
        content = RELEASE_AUTOMATION.read_text(encoding="utf-8")

        self.assertIn("<pre|post|publish>", content)
        self.assertIn("publish --version 0.1.0", content)
        self.assertIn("./scripts/release_ready.sh --version", content)
        self.assertIn('git tag -a "$repo_tag"', content)
        self.assertIn('git push "$push_remote" "$push_branch" "$repo_tag"', content)
        self.assertIn('./scripts/release_postflight.sh --tag "$repo_tag"', content)

    def test_release_postflight_waits_for_required_workflows(self) -> None:
        content = RELEASE_POSTFLIGHT.read_text(encoding="utf-8")

        self.assertIn('REQUIRED_WORKFLOWS=("CI" "Install Check")', content)
        self.assertIn("--wait-ci", content)
        self.assertIn("query_ci_status", content)
        self.assertIn('ci_json_file="$(mktemp)"', content)
        self.assertNotIn("CI_JSON_PAYLOAD=", content)
        self.assertIn("gh release view", content)
        self.assertIn("--prerelease", content)

    def test_release_workflow_exposes_publish_mode(self) -> None:
        content = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("push:", content)
        self.assertIn('tags:\n      - "v*"', content)
        self.assertIn("- publish", content)
        self.assertIn("version:", content)
        self.assertIn("fetch-depth: 0", content)
        self.assertIn('git fetch --force --tags origin', content)
        self.assertIn('if [[ "${{ github.event_name }}" == "push" ]]; then', content)
        self.assertIn('mode="post"', content)
        self.assertIn('args+=(--create-release)', content)
        self.assertIn('git config user.name "github-actions[bot]"', content)


if __name__ == "__main__":
    unittest.main()
