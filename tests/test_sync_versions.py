from __future__ import annotations

import contextlib
import importlib.util
import io
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SYNC_VERSIONS_PATH = REPO_ROOT / "scripts" / "sync_versions.py"
SPEC = importlib.util.spec_from_file_location("sync_versions_module", SYNC_VERSIONS_PATH)
assert SPEC is not None and SPEC.loader is not None
sync_versions_module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(sync_versions_module)


class SyncVersionsTests(unittest.TestCase):
    def test_parse_version_normalizes_beta_layers(self) -> None:
        package_version, skill_version, repo_version = sync_versions_module.parse_version(
            "0.2.0b3"
        )

        self.assertEqual(package_version, "0.2.0b3")
        self.assertEqual(skill_version, "0.2.0-beta.3")
        self.assertEqual(repo_version, "v0.2.0-beta.3")

    def test_main_print_field_outputs_repo_version_without_syncing(self) -> None:
        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            exit_code = sync_versions_module.main(
                ["0.2.0b1", "--print-field", "repo_version"]
            )

        self.assertEqual(exit_code, 0)
        self.assertEqual(stdout.getvalue().strip(), "v0.2.0-beta.1")

    def test_sync_versions_updates_expected_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / "research_skills").mkdir()
            (root / "skills" / "F_writing").mkdir(parents=True)
            (root / "research-paper-workflow").mkdir()

            (root / "pyproject.toml").write_text(
                'name = "research-skills-installer"\nversion = "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "research_skills" / "__init__.py").write_text(
                '__version__ = "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "skills" / "registry.yaml").write_text(
                'skills:\n  - id: "demo"\n    version: "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "research-paper-workflow" / "VERSION").write_text(
                "v0.1.0\n",
                encoding="utf-8",
            )
            (root / "skills" / "F_writing" / "demo.md").write_text(
                '---\nid: "demo"\nstage: "F_writing"\n---\n',
                encoding="utf-8",
            )

            changed = sync_versions_module.sync_versions(root, "0.2.0b2")

            self.assertIn(root / "pyproject.toml", changed)
            self.assertIn(root / "research_skills" / "__init__.py", changed)
            self.assertIn(root / "skills" / "registry.yaml", changed)
            self.assertIn(root / "research-paper-workflow" / "VERSION", changed)
            self.assertIn('version = "0.2.0b2"', (root / "pyproject.toml").read_text())
            self.assertIn(
                '__version__ = "0.2.0b2"',
                (root / "research_skills" / "__init__.py").read_text(),
            )
            self.assertIn(
                'version: "0.2.0-beta.2"',
                (root / "skills" / "registry.yaml").read_text(),
            )
            self.assertEqual(
                (root / "research-paper-workflow" / "VERSION").read_text().strip(),
                "v0.2.0-beta.2",
            )
            self.assertNotIn(root / "skills" / "F_writing" / "demo.md", changed)
