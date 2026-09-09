from __future__ import annotations

import contextlib
import io
import json
import shutil
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path

from scripts.release_version import main, parse_release_version


REPO_ROOT = Path(__file__).resolve().parents[1]


class ReleaseVersionContractTests(unittest.TestCase):
    def test_registry_prerelease_channels_are_standard_and_share_one_version(self):
        for raw, semver, pypi in [('2.0.0A1', '2.0.0-alpha.1', '2.0.0a1'),
                                  ('2.0.0B1', '2.0.0-beta.1', '2.0.0b1'),
                                  ('v2.0.0-alpha.7', '2.0.0-alpha.7', '2.0.0a7')]:
            identity = parse_release_version(raw)
            self.assertEqual(identity.npm_version, semver)
            self.assertEqual(identity.package_version, pypi)
            self.assertEqual(identity.repo_tag, 'v' + semver)
            self.assertEqual(identity.npm_dist_tag, 'next')
        self.assertEqual(parse_release_version('2.0.0').npm_dist_tag, 'latest')

    def test_supported_versions_have_complete_canonical_identities(self) -> None:
        cases = (
            (
                "0.2.0",
                {
                    "release_line": "legacy-1x",
                    "version": "0.2.0",
                    "repo_tag": "v0.2.0",
                    "channel": "stable",
                    "prerelease_number": None,
                    "package_version": "0.2.0",
                    "source_branch": "primary",
                    "version_source": "pyproject.toml",
                    "is_prerelease": False,
                },
            ),
            (
                "v1.19.1-beta.2",
                {
                    "release_line": "legacy-1x",
                    "version": "1.19.1-beta.2",
                    "repo_tag": "v1.19.1-beta.2",
                    "channel": "beta",
                    "prerelease_number": 2,
                    "package_version": "1.19.1b2",
                    "source_branch": "dev",
                    "version_source": "pyproject.toml",
                    "is_prerelease": True,
                },
            ),
            (
                "2.0.0a1",
                {
                    "release_line": "native-2x",
                    "version": "2.0.0-alpha.1",
                    "repo_tag": "v2.0.0-alpha.1",
                    "channel": "alpha",
                    "prerelease_number": 1,
                    "package_version": "2.0.0a1",
                    "source_branch": "2.x",
                    "version_source": "packages/qiongli-native/Cargo.toml",
                    "is_prerelease": True,
                },
            ),
            (
                "v2.3.4-beta.5",
                {
                    "release_line": "native-2x",
                    "version": "2.3.4-beta.5",
                    "repo_tag": "v2.3.4-beta.5",
                    "channel": "beta",
                    "prerelease_number": 5,
                    "package_version": "2.3.4b5",
                    "source_branch": "2.x",
                    "version_source": "packages/qiongli-native/Cargo.toml",
                    "is_prerelease": True,
                },
            ),
            (
                "2.4.0",
                {
                    "release_line": "native-2x",
                    "version": "2.4.0",
                    "repo_tag": "v2.4.0",
                    "channel": "stable",
                    "prerelease_number": None,
                    "package_version": "2.4.0",
                    "source_branch": "2.x",
                    "version_source": "packages/qiongli-native/Cargo.toml",
                    "is_prerelease": False,
                },
            ),
        )

        for raw, expected in cases:
            with self.subTest(raw=raw):
                identity = parse_release_version(raw)
                self.assertEqual(identity.product, "qiongli")
                self.assertEqual(identity.skill_version, expected["version"])
                self.assertEqual(identity.npm_version, expected["version"])
                for field, value in expected.items():
                    self.assertEqual(getattr(identity, field), value)

    def test_compact_and_semver_prerelease_aliases_are_identical(self) -> None:
        aliases = (
            ("1.19.1b3", "v1.19.1-beta.3"),
            ("2.0.0a4", "v2.0.0-alpha.4"),
            ("2.0.0b5", "v2.0.0-beta.5"),
        )
        for compact, semantic in aliases:
            with self.subTest(compact=compact):
                self.assertEqual(
                    parse_release_version(compact),
                    parse_release_version(semantic),
                )

    def test_expected_channel_must_agree_with_version(self) -> None:
        identity = parse_release_version("v2.0.0-alpha.1", expected_channel="alpha")
        self.assertEqual(identity.channel, "alpha")

        with self.assertRaisesRegex(ValueError, "channel mismatch"):
            parse_release_version("v2.0.0-alpha.1", expected_channel="stable")
        with self.assertRaisesRegex(ValueError, "unsupported expected channel"):
            parse_release_version("2.0.0", expected_channel="next")

    def test_unsupported_or_ambiguous_versions_fail_closed(self) -> None:
        invalid = (
            "v1.19.1-alpha.1",
            "1.19.1a1",
            "2.0.0-alpha",
            "2.0.0-beta",
            "2.0.0a",
            "2.0.0b",
            "2.0.0-alpha.01",
            "2.0.0-beta.01",
            "2.0.0a01",
            "2.0.0b01",
            "2.0.0-alpha.0",
            "2.0.0-beta.0",
            "2.0.0a0",
            "2.0.0b0",
            "02.0.0-alpha.1",
            "2.00.0-alpha.1",
            "2.0.00-alpha.1",
            "2.0.0-rc.1",
            "2.0.0-dev.1",
            "2.0.0-alpha.1+build.7",
            "3.0.0-alpha.1",
            "V2.0.0-alpha.1",
            "next",
            "",
        )
        for raw in invalid:
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                parse_release_version(raw)

    def test_cli_emits_json_or_one_normalized_field(self) -> None:
        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            self.assertEqual(main(["2.0.0a2"]), 0)
        payload = json.loads(stdout.getvalue())
        self.assertEqual(payload["repo_tag"], "v2.0.0-alpha.2")
        self.assertEqual(payload["release_line"], "native-2x")

        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            self.assertEqual(main(["2.0.0a2", "--print-field", "source_branch"]), 0)
        self.assertEqual(stdout.getvalue().strip(), "2.x")

        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            self.assertEqual(main(["2.0.0a2", "--print-field", "repo_version"]), 0)
        self.assertEqual(stdout.getvalue().strip(), "v2.0.0-alpha.2")

    def test_native_tag_contract_rejects_one_drifted_version_owner(self) -> None:
        native_manifest_path = REPO_ROOT / "packages/qiongli-native/Cargo.toml"
        with native_manifest_path.open("rb") as handle:
            native_manifest = tomllib.load(handle)
        workspace = native_manifest["workspace"]
        version = workspace["package"]["version"]
        tag = f"v{version}"

        relative_files = [
            Path("scripts/release_version.py"),
            Path("tooling/scripts/release_version.py"),
            Path("packages/qiongli-native/Cargo.toml"),
            Path("packages/qiongli-native/Cargo.lock"),
            Path("packages/qiongli-lite-mcp/Cargo.lock"),
            Path("content/.codex-plugin/plugin.json"),
            Path("content/.claude-plugin/plugin.json"),
            Path("content/skills/registry.yaml"),
            Path("content/workflow/VERSION"),
            Path("content/workflow/SKILL.md"),
            Path(
                "packages/qiongli-native/crates/qiongli-content/resources/"
                "qiongli-core.lock.json"
            ),
            Path(f"tooling/release/{tag}.md"),
            Path(
                "packages/qiongli-native/apps/qiongli/examples/"
                "native_candidate_acceptance.rs"
            ),
            Path(
                "packages/qiongli-native/apps/qiongli/examples/"
                "native_community_alpha_promotion.rs"
            ),
            Path(
                "packages/qiongli-native/apps/qiongli/examples/"
                "native_community_alpha_release.rs"
            ),
            Path(
                "packages/qiongli-native/apps/qiongli/examples/"
                "native_update_metadata.rs"
            ),
        ]
        relative_files.extend(
            Path("packages/qiongli-native") / member / "Cargo.toml"
            for member in workspace["members"]
        )

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            for relative in relative_files:
                target = root / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(REPO_ROOT / relative, target)

            command = [
                "bash",
                str(REPO_ROOT / "scripts/verify_release_tag_version.sh"),
                "--root",
                str(root),
                "--tag",
                tag,
            ]
            aligned = subprocess.run(
                command,
                cwd=REPO_ROOT,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            self.assertEqual(aligned.returncode, 0, aligned.stderr)

            plugin_path = root / "content/.codex-plugin/plugin.json"
            plugin = json.loads(plugin_path.read_text(encoding="utf-8"))
            plugin["version"] = "2.0.0-alpha.9999"
            plugin_path.write_text(json.dumps(plugin, indent=2) + "\n", encoding="utf-8")
            drifted = subprocess.run(
                command,
                cwd=REPO_ROOT,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            self.assertEqual(drifted.returncode, 1, drifted.stderr)
            self.assertIn("native plugin version mismatch", drifted.stderr)


if __name__ == "__main__":
    unittest.main()
