from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from research_skills.universal_installer import InstallOptions, clean, install


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
            # Project parts now only installs .env
            self.assertTrue((project_dir / ".env").exists())
            # Workflows and CLAUDE.md are no longer installed project-locally
            self.assertFalse((project_dir / ".agent" / "workflows" / "proofread.md").exists())
            self.assertFalse((project_dir / "CLAUDE.md").exists())
            self.assertFalse((project_dir / ".gemini" / "research-skills.md").exists())

    def test_partial_profile_installs_global_skills_with_bundled_workflows(self) -> None:
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
            # Global skills installed for all clients
            self.assertTrue((codex_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())
            self.assertTrue((claude_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())
            self.assertTrue((gemini_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())
            self.assertTrue((antigravity_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())
            # Workflows bundled inside each global skill directory
            self.assertTrue((claude_home / "skills" / "research-paper-workflow" / "workflows" / "paper.md").exists())
            self.assertTrue((gemini_home / "skills" / "research-paper-workflow" / "workflows" / "lit-review.md").exists())
            self.assertTrue((antigravity_home / "skills" / "research-paper-workflow" / "workflows" / "paper.md").exists())
            # Synced bundled assets present in global skill directories
            self.assertTrue((claude_home / "skills" / "research-paper-workflow" / "skills-core.md").exists())
            self.assertTrue((claude_home / "skills" / "research-paper-workflow" / "skills" / "A_framing").is_dir())
            self.assertTrue((claude_home / "skills" / "research-paper-workflow" / "templates" / "manuscript-outline.md").exists())
            self.assertTrue((claude_home / "skills" / "research-paper-workflow" / "standards" / "research-workflow-contract.yaml").exists())
            self.assertTrue((claude_home / "skills" / "research-paper-workflow" / "roles" / "pi.yaml").exists())
            # No project-local files
            self.assertFalse((project_dir / ".agent" / "workflows" / "proofread.md").exists())
            self.assertFalse((project_dir / ".gemini" / "research-skills.md").exists())
            self.assertFalse((project_dir / ".agents" / "skills" / "research-paper-workflow" / "SKILL.md").exists())
            self.assertFalse((project_dir / ".env").exists())
            self.assertFalse((temp_root / ".local" / "bin" / "research-skills").exists())

    def test_full_profile_allows_explicit_no_cli(self) -> None:
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
            self.assertFalse((cli_dir / "research-skills").exists())


class CleanTests(unittest.TestCase):
    def test_clean_removes_stale_project_assets(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_dir = Path(tmp_dir) / "project"
            project_dir.mkdir(parents=True)
            # Create stale assets
            workflows_dir = project_dir / ".agent" / "workflows"
            workflows_dir.mkdir(parents=True)
            (workflows_dir / "paper.md").write_text("stale workflow")
            (workflows_dir / "lit-review.md").write_text("stale workflow")
            skill_dir = project_dir / ".agents" / "skills" / "research-paper-workflow"
            skill_dir.mkdir(parents=True)
            (skill_dir / "SKILL.md").write_text("stale skill")
            legacy_skill = project_dir / ".agent" / "skills" / "research-paper-workflow"
            legacy_skill.mkdir(parents=True)
            (legacy_skill / "SKILL.md").write_text("stale skill")
            gemini_dir = project_dir / ".gemini"
            gemini_dir.mkdir(parents=True)
            (gemini_dir / "research-skills.md").write_text("stale quickstart")
            (gemini_dir / "agent-profiles.example.json").write_text("{}")
            (project_dir / "CLAUDE.research-skills.md").write_text("stale")
            # Write a template-looking CLAUDE.md
            (project_dir / "CLAUDE.md").write_text("# Academic Deep Research Skills\nresearch-paper-workflow")

            result = clean(project_dir)
            self.assertEqual(result, 0)
            # All stale files removed
            self.assertFalse((workflows_dir / "paper.md").exists())
            self.assertFalse((workflows_dir / "lit-review.md").exists())
            self.assertFalse(skill_dir.exists())
            self.assertFalse(legacy_skill.exists())
            self.assertFalse((gemini_dir / "research-skills.md").exists())
            self.assertFalse((gemini_dir / "agent-profiles.example.json").exists())
            self.assertFalse((project_dir / "CLAUDE.research-skills.md").exists())
            self.assertFalse((project_dir / "CLAUDE.md").exists())

    def test_clean_keeps_user_customized_claude_md(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_dir = Path(tmp_dir) / "project"
            project_dir.mkdir(parents=True)
            (project_dir / "CLAUDE.md").write_text("# My Custom Project Instructions\nCustom content")

            result = clean(project_dir)
            self.assertEqual(result, 0)
            # User-customized CLAUDE.md should be preserved
            self.assertTrue((project_dir / "CLAUDE.md").exists())

    def test_clean_dry_run_does_not_delete(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_dir = Path(tmp_dir) / "project"
            workflows_dir = project_dir / ".agent" / "workflows"
            workflows_dir.mkdir(parents=True)
            (workflows_dir / "paper.md").write_text("stale")

            result = clean(project_dir, dry_run=True)
            self.assertEqual(result, 0)
            # File should still exist after dry-run
            self.assertTrue((workflows_dir / "paper.md").exists())


if __name__ == "__main__":
    unittest.main()
