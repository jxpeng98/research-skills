from __future__ import annotations

import contextlib
import importlib.util
import io
import tempfile
import unittest
from pathlib import Path

import yaml

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
SYNC_VERSIONS_PATH = RepoLayout(REPO_ROOT).scripts / "sync_versions.py"
SPEC = importlib.util.spec_from_file_location("sync_versions_module", SYNC_VERSIONS_PATH)
assert SPEC is not None and SPEC.loader is not None
sync_versions_module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(sync_versions_module)


class SyncVersionsTests(unittest.TestCase):
    def test_entrypoint_version_collapses_repeated_release_prefixes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "SKILL.md"
            path.write_text('---\nname: qiongli\ndescription: "Qiongli version: v2.0.0-alpha.7. Qiongli version: v2.0.0-alpha.2. Research workflow."\n---\n')
            self.assertTrue(sync_versions_module.replace_skill_entrypoint_version(path, "v2.0.0-alpha.8"))
            result = path.read_text()
            self.assertIn('description: "Qiongli version: v2.0.0-alpha.8. Research workflow."', result)
            self.assertFalse(sync_versions_module.replace_skill_entrypoint_version(path, "v2.0.0-alpha.8"))

    def test_parse_version_normalizes_beta_layers(self) -> None:
        package_version, skill_version, repo_version, npm_version = sync_versions_module.parse_version(
            "0.2.0b3"
        )

        self.assertEqual(package_version, "0.2.0b3")
        self.assertEqual(skill_version, "0.2.0-beta.3")
        self.assertEqual(repo_version, "v0.2.0-beta.3")
        self.assertEqual(npm_version, "0.2.0-beta.3")

    def test_parse_version_preserves_four_tuple_api_for_native_alpha(self) -> None:
        self.assertEqual(
            sync_versions_module.parse_version("v2.0.0-alpha.2"),
            (
                "2.0.0a2",
                "2.0.0-alpha.2",
                "v2.0.0-alpha.2",
                "2.0.0-alpha.2",
            ),
        )

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
            root = Path(tmp_dir).resolve()
            layout = RepoLayout(root)
            layout.python_package.mkdir(parents=True)
            (root / "skills" / "F_writing").mkdir(parents=True)
            (root / "qiongli-workflow" / "skills").mkdir(parents=True)
            (root / "packages" / "npm-qiongli").mkdir(parents=True)

            (root / "pyproject.toml").write_text(
                'name = "qiongli"\nversion = "0.1.0"\n',
                encoding="utf-8",
            )
            (layout.python_package / "__init__.py").write_text(
                '__version__ = "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "skills" / "registry.yaml").write_text(
                'skills:\n  - id: "demo"\n    version: "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "qiongli-workflow" / "VERSION").write_text("v0.1.0\n", encoding="utf-8")
            (root / "qiongli-workflow" / "SKILL.md").write_text(
                "---\n"
                "name: qiongli\n"
                'description: "Qiongli version: v0.1.0. Demo workflow."\n'
                "---\n"
                "\n"
                "# Qiongli Academic Workflow\n"
                "\n"
                "Installed Qiongli workflow version: `v0.1.0`\n",
                encoding="utf-8",
            )
            (root / "qiongli-workflow" / "skills" / "registry.yaml").write_text(
                'skills:\n  - id: "demo"\n    version: "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "packages" / "npm-qiongli" / "package.json").write_text(
                '{\n  "name": "qiongli",\n  "version": "0.1.0"\n}\n',
                encoding="utf-8",
            )
            (root / "uv.lock").write_text(
                'version = 1\n'
                '\n'
                '[[package]]\n'
                'name = "qiongli"\n'
                'version = "0.1.0"\n'
                'source = { editable = "." }\n',
                encoding="utf-8",
            )
            (root / "package-lock.json").write_text(
                '{\n'
                '  "name": "qiongli-docs",\n'
                '  "lockfileVersion": 3,\n'
                '  "packages": {\n'
                '    "": {"name": "qiongli-docs"},\n'
                '    "node_modules/qiongli": {"resolved": "packages/npm-qiongli", "link": true},\n'
                '    "packages/npm-qiongli": {"name": "qiongli", "version": "0.1.0"}\n'
                '  }\n'
                '}\n',
                encoding="utf-8",
            )

            npm_payload = root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow"
            (npm_payload / "skills").mkdir(parents=True)
            (npm_payload / "VERSION").write_text("v0.1.0\n", encoding="utf-8")
            (npm_payload / "skills" / "registry.yaml").write_text(
                'skills:\n  - id: "demo"\n    version: "0.1.0"\n',
                encoding="utf-8",
            )
            npm_runtime = root / "packages" / "npm-qiongli" / "python-runtime"
            (npm_runtime / "qiongli").mkdir(parents=True)
            (npm_runtime / "qiongli" / "__init__.py").write_text(
                '__version__ = "0.1.0"\n',
                encoding="utf-8",
            )
            (npm_runtime / "skills").mkdir(parents=True)
            (npm_runtime / "skills" / "registry.yaml").write_text(
                'skills:\n  - id: "demo"\n    version: "0.1.0"\n',
                encoding="utf-8",
            )

            generated_manifest = root / "plugins" / "qiongli" / ".codex-plugin" / "plugin.json"
            generated_manifest.parent.mkdir(parents=True)
            generated_manifest.write_text(
                '{\n  "name": "qiongli",\n  "version": "0.1.0"\n}\n',
                encoding="utf-8",
            )
            generated_next_version = root / "plugins" / "qiongli-next" / "skills" / "qiongli-workflow" / "VERSION"
            generated_next_version.parent.mkdir(parents=True)
            generated_next_version.write_text("v0.1.0\n", encoding="utf-8")
            (root / "skills" / "F_writing" / "demo.md").write_text(
                '---\nid: "demo"\nstage: "F_writing"\n---\n',
                encoding="utf-8",
            )

            changed = sync_versions_module.sync_versions(root, "0.2.0b2")

            self.assertIn(root / "pyproject.toml", changed)
            self.assertIn(layout.python_package / "__init__.py", changed)
            self.assertIn(root / "skills" / "registry.yaml", changed)
            self.assertIn(root / "qiongli-workflow" / "VERSION", changed)
            self.assertIn(root / "qiongli-workflow" / "SKILL.md", changed)
            self.assertIn(root / "qiongli-workflow" / "skills" / "registry.yaml", changed)
            self.assertIn(root / "packages" / "npm-qiongli" / "package.json", changed)
            self.assertIn(root / "package-lock.json", changed)
            self.assertIn(root / "uv.lock", changed)
            self.assertIn(npm_payload / "VERSION", changed)
            self.assertIn(npm_payload / "skills" / "registry.yaml", changed)
            self.assertIn(npm_runtime / "qiongli" / "__init__.py", changed)
            self.assertIn(npm_runtime / "skills" / "registry.yaml", changed)

            self.assertIn('version = "0.2.0b2"', (root / "pyproject.toml").read_text())
            self.assertIn('__version__ = "0.2.0b2"', (layout.python_package / "__init__.py").read_text())
            self.assertIn('version: "0.2.0-beta.2"', (root / "skills" / "registry.yaml").read_text())
            self.assertEqual((root / "qiongli-workflow" / "VERSION").read_text().strip(), "v0.2.0-beta.2")
            updated_skill = (root / "qiongli-workflow" / "SKILL.md").read_text()
            self.assertIn("Qiongli version: v0.2.0-beta.2", updated_skill)
            metadata = yaml.safe_load(updated_skill.split("---", 2)[1])
            self.assertEqual(metadata["description"], "Qiongli version: v0.2.0-beta.2. Demo workflow.")
            self.assertIn(
                "Installed Qiongli workflow version: `v0.2.0-beta.2`",
                updated_skill,
            )
            self.assertIn(
                'version: "0.2.0-beta.2"',
                (root / "qiongli-workflow" / "skills" / "registry.yaml").read_text(),
            )
            self.assertIn(
                '"version": "0.2.0-beta.2"',
                (root / "packages" / "npm-qiongli" / "package.json").read_text(),
            )
            self.assertIn('version = "0.2.0b2"', (root / "uv.lock").read_text())
            self.assertIn('"version": "0.2.0-beta.2"', (root / "package-lock.json").read_text())
            self.assertEqual((npm_payload / "VERSION").read_text().strip(), "v0.2.0-beta.2")
            self.assertIn('version: "0.2.0-beta.2"', (npm_payload / "skills" / "registry.yaml").read_text())
            self.assertIn('__version__ = "0.2.0b2"', (npm_runtime / "qiongli" / "__init__.py").read_text())
            self.assertIn('version: "0.2.0-beta.2"', (npm_runtime / "skills" / "registry.yaml").read_text())

            self.assertNotIn(generated_manifest, changed)
            self.assertNotIn(generated_next_version, changed)
            self.assertIn('"version": "0.1.0"', generated_manifest.read_text())
            self.assertEqual(generated_next_version.read_text().strip(), "v0.1.0")
            self.assertNotIn(root / "skills" / "F_writing" / "demo.md", changed)

    def test_native_sync_updates_workspace_identity_lock_and_native_content(self) -> None:
        targets = (
            ("v2.0.0-alpha.2", "2.0.0-alpha.2", "alpha"),
            ("2.1.0b3", "2.1.0-beta.3", "beta"),
            ("v2.1.0", "2.1.0", "stable"),
        )
        for raw, expected_version, expected_channel in targets:
            with self.subTest(raw=raw), tempfile.TemporaryDirectory() as tmp_dir:
                root = Path(tmp_dir).resolve()
                native_root = root / "packages" / "qiongli-native"
                native_root.mkdir(parents=True)
                manifest = native_root / "Cargo.toml"
                manifest.write_text(
                    """[workspace]
resolver = "3"
members = ["apps/qiongli", "crates/qiongli-platform"]

[workspace.package]
version = "2.0.0-alpha.1"
edition = "2024"

[workspace.metadata.qiongli]
product = "qiongli"
channel = "alpha"

[workspace.lints.rust]
unsafe_code = "forbid"
""",
                    encoding="utf-8",
                )
                for member, package_name in (
                    ("apps/qiongli", "qiongli"),
                    ("crates/qiongli-platform", "qiongli-platform"),
                ):
                    member_manifest = native_root / member / "Cargo.toml"
                    member_manifest.parent.mkdir(parents=True)
                    member_manifest.write_text(
                        f'[package]\nname = "{package_name}"\nversion.workspace = true\n',
                        encoding="utf-8",
                    )
                lockfile = native_root / "Cargo.lock"
                lockfile.write_text(
                    """version = 4

[[package]]
name = "qiongli"
version = "2.0.0-alpha.1"

[[package]]
name = "qiongli-platform"
version = "2.0.0-alpha.1"

[[package]]
name = "unchanged-dependency"
version = "9.8.7"
""",
                    encoding="utf-8",
                )
                lite_lockfile = root / "packages" / "qiongli-lite-mcp" / "Cargo.lock"
                lite_lockfile.parent.mkdir(parents=True)
                lite_lockfile.write_text(
                    """version = 4

[[package]]
name = "qiongli-lite-mcp"
version = "0.2.0-beta.3"

[[package]]
name = "qiongli-platform"
version = "2.0.0-alpha.1"
""",
                    encoding="utf-8",
                )
                content_root = root / "content"
                for plugin_kind in (".codex-plugin", ".claude-plugin"):
                    plugin = content_root / plugin_kind / "plugin.json"
                    plugin.parent.mkdir(parents=True)
                    plugin.write_text(
                        '{"name":"qiongli","version":"2.0.0-alpha.1"}\n',
                        encoding="utf-8",
                    )
                full_mcpb_manifest = root / "packages" / "qiongli-full-mcpb" / "manifest.json"
                full_mcpb_manifest.parent.mkdir(parents=True)
                full_mcpb_manifest.write_text(
                    '{"name":"qiongli-full-runtime","version":"2.0.0-alpha.1"}\n',
                    encoding="utf-8",
                )
                registry = content_root / "skills" / "registry.yaml"
                registry.parent.mkdir(parents=True)
                registry.write_text(
                    'skills:\n  - id: demo\n    version: "2.0.0-alpha.1"\n',
                    encoding="utf-8",
                )
                workflow_version = content_root / "workflow" / "VERSION"
                workflow_version.parent.mkdir(parents=True)
                workflow_version.write_text("v2.0.0-alpha.1\n", encoding="utf-8")
                workflow_skill = content_root / "workflow" / "SKILL.md"
                workflow_skill.write_text(
                    "---\n"
                    "name: qiongli\n"
                    'description: "Qiongli version: v2.0.0-alpha.1. Native workflow."\n'
                    "---\n\n"
                    "Installed Qiongli workflow version: `v2.0.0-alpha.1`\n",
                    encoding="utf-8",
                )
                legacy_files = {
                    root / "pyproject.toml": '[project]\nversion = "1.19.0b1"\n',
                    root / "packages" / "npm-qiongli" / "package.json": (
                        '{"name":"qiongli","version":"1.19.0-beta.1"}\n'
                    ),
                    root / "qiongli-workflow" / "VERSION": "v1.19.0-beta.1\n",
                }
                for path, content in legacy_files.items():
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text(content, encoding="utf-8")

                changed = sync_versions_module.sync_versions(root, raw)

                self.assertEqual(
                    changed,
                    [
                        manifest,
                        lockfile,
                        lite_lockfile,
                        content_root / ".codex-plugin" / "plugin.json",
                        content_root / ".claude-plugin" / "plugin.json",
                        full_mcpb_manifest,
                        registry,
                        workflow_version,
                        workflow_skill,
                    ],
                )
                manifest_text = manifest.read_text(encoding="utf-8")
                self.assertIn(f'version = "{expected_version}"', manifest_text)
                self.assertIn(f'channel = "{expected_channel}"', manifest_text)
                self.assertIn('edition = "2024"', manifest_text)
                lock_text = lockfile.read_text(encoding="utf-8")
                self.assertEqual(lock_text.count(f'version = "{expected_version}"'), 2)
                self.assertIn('name = "unchanged-dependency"\nversion = "9.8.7"', lock_text)
                lite_lock_text = lite_lockfile.read_text(encoding="utf-8")
                self.assertIn(
                    f'name = "qiongli-platform"\nversion = "{expected_version}"',
                    lite_lock_text,
                )
                self.assertIn(
                    'name = "qiongli-lite-mcp"\nversion = "0.2.0-beta.3"',
                    lite_lock_text,
                )
                self.assertIn(
                    f'"version": "{expected_version}"',
                    (content_root / ".codex-plugin" / "plugin.json").read_text(),
                )
                self.assertIn(
                    f'"version": "{expected_version}"',
                    full_mcpb_manifest.read_text(),
                )
                self.assertIn(
                    f'version: "{expected_version}"',
                    registry.read_text(encoding="utf-8"),
                )
                self.assertEqual(workflow_version.read_text().strip(), f"v{expected_version}")
                self.assertIn(
                    f"Installed Qiongli workflow version: `v{expected_version}`",
                    workflow_skill.read_text(encoding="utf-8"),
                )
                for path, original in legacy_files.items():
                    self.assertEqual(path.read_text(encoding="utf-8"), original)

                self.assertEqual(sync_versions_module.sync_versions(root, raw), [])

    def test_native_sync_validates_manifest_and_lock_before_writing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir).resolve()
            native_root = root / "packages" / "qiongli-native"
            native_root.mkdir(parents=True)
            manifest = native_root / "Cargo.toml"
            original_manifest = """[workspace.package]
version = "2.0.0-alpha.1"

[workspace.metadata.qiongli]
channel = "alpha"
"""
            manifest.write_text(original_manifest, encoding="utf-8")
            (native_root / "Cargo.lock").write_text(
                "version = 4\n\n[[package]]\nname = \"not-qiongli\"\nversion = \"1.0.0\"\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "exactly one Cargo.lock package"):
                sync_versions_module.sync_versions(root, "v2.0.0-alpha.2")

            self.assertEqual(manifest.read_text(encoding="utf-8"), original_manifest)
