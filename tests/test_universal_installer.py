from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from research_skills.universal_installer import InstallOptions, install


REPO_ROOT = Path(__file__).resolve().parents[1]


class UniversalInstallerTests(unittest.TestCase):
    def test_project_only_parts_skip_global_skill_install(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            claude_home = temp_root / "claude-home"
            gemini_home = temp_root / "gemini-home"
            antigravity_home = temp_root / "antigravity-home"

            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["CLAUDE_CODE_HOME"] = str(claude_home)
            env["GEMINI_HOME"] = str(gemini_home)
            env["ANTIGRAVITY_HOME"] = str(antigravity_home)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        target="all",
                        parts=("project",),
                    )
                )

            self.assertEqual(result, 0)
            self.assertFalse((codex_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())
            self.assertFalse((claude_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())
            self.assertFalse((gemini_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())
            self.assertTrue((project_dir / ".agent" / "workflows" / "proofread.md").exists())
            self.assertTrue((project_dir / "CLAUDE.md").exists())
            self.assertTrue((project_dir / ".gemini" / "research-skills.md").exists())
            self.assertTrue((project_dir / ".env").exists())

    def test_partial_profile_installs_assets_without_shell_cli(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            claude_home = temp_root / "claude-home"
            gemini_home = temp_root / "gemini-home"
            antigravity_home = temp_root / "antigravity-home"

            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["CLAUDE_CODE_HOME"] = str(claude_home)
            env["GEMINI_HOME"] = str(gemini_home)
            env["ANTIGRAVITY_HOME"] = str(antigravity_home)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        profile="partial",
                    )
                )

            self.assertEqual(result, 0)
            self.assertTrue((codex_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())
            self.assertTrue((claude_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())
            self.assertTrue((gemini_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())
            self.assertTrue((project_dir / ".agent" / "workflows" / "proofread.md").exists())
            self.assertTrue((project_dir / ".gemini" / "research-skills.md").exists())
            self.assertTrue((project_dir / ".agents" / "skills" / "research-paper-workflow" / "SKILL.md").exists())
            self.assertTrue((project_dir / ".env").exists())
            self.assertFalse((temp_root / ".local" / "bin" / "research-skills").exists())

    def test_full_profile_allows_explicit_no_cli_and_preserves_existing_claude_md(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            (project_dir / "CLAUDE.md").write_text("existing", encoding="utf-8")
            codex_home = temp_root / "codex-home"
            claude_home = temp_root / "claude-home"
            gemini_home = temp_root / "gemini-home"
            antigravity_home = temp_root / "antigravity-home"
            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["CLAUDE_CODE_HOME"] = str(claude_home)
            env["GEMINI_HOME"] = str(gemini_home)
            env["ANTIGRAVITY_HOME"] = str(antigravity_home)
            env["PATH"] = ""

            cli_dir = temp_root / "bin"
            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        profile="full",
                        install_cli=False,
                        doctor=False,
                        cli_dir=cli_dir,
                    )
                )

            self.assertEqual(result, 0)
            self.assertEqual((project_dir / "CLAUDE.md").read_text(encoding="utf-8"), "existing")
            self.assertTrue((project_dir / "CLAUDE.research-skills.md").exists())
            self.assertFalse((cli_dir / "research-skills").exists())


if __name__ == "__main__":
    unittest.main()
