from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
INSTALL_SCRIPT = REPO_ROOT / "scripts" / "install_research_skill.sh"
SYSTEM_BASH = Path("/bin/bash")


class InstallResearchSkillTests(unittest.TestCase):
    def test_doctor_command_exports_repo_root_on_pythonpath(self) -> None:
        content = INSTALL_SCRIPT.read_text(encoding="utf-8")

        self.assertIn('doctor_cmd() {', content)
        self.assertIn('PYTHONPATH="$pythonpath" python3 -m bridges.orchestrator "$@"', content)

    def test_claude_install_succeeds_under_system_bash(self) -> None:
        if not SYSTEM_BASH.exists():
            self.skipTest("/bin/bash is not available")

        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project with spaces"
            project_dir.mkdir(parents=True)
            claude_home = temp_root / "claude-home"
            home_dir = temp_root / "home"
            home_dir.mkdir()

            env = os.environ.copy()
            env["HOME"] = str(home_dir)
            env["CLAUDE_CODE_HOME"] = str(claude_home)
            env["NO_COLOR"] = "1"

            result = subprocess.run(
                [
                    str(SYSTEM_BASH),
                    str(INSTALL_SCRIPT),
                    "--target",
                    "claude",
                    "--mode",
                    "copy",
                    "--project-dir",
                    str(project_dir),
                ],
                cwd=str(REPO_ROOT),
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stdout + "\n" + result.stderr)
            self.assertTrue((project_dir / ".agent" / "workflows" / "proofread.md").exists())
            self.assertTrue((project_dir / ".agent" / "workflows" / "study-design.md").exists())
            self.assertTrue((project_dir / "CLAUDE.md").exists())
            self.assertTrue((project_dir / ".env").exists())
            self.assertIn("Env", result.stdout)
            self.assertTrue((claude_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())

    def test_antigravity_install_writes_workspace_and_global_skill_when_cli_exists(self) -> None:
        if not SYSTEM_BASH.exists():
            self.skipTest("/bin/bash is not available")

        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            antigravity_home = temp_root / "antigravity-home"
            home_dir = temp_root / "home"
            bin_dir = temp_root / "bin"
            home_dir.mkdir()
            bin_dir.mkdir()
            antigravity_cli = bin_dir / "antigravity"
            antigravity_cli.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
            antigravity_cli.chmod(0o755)

            env = os.environ.copy()
            env["HOME"] = str(home_dir)
            env["ANTIGRAVITY_HOME"] = str(antigravity_home)
            env["PATH"] = f"{bin_dir}{os.pathsep}{env.get('PATH', '')}"
            env["NO_COLOR"] = "1"

            result = subprocess.run(
                [
                    str(SYSTEM_BASH),
                    str(INSTALL_SCRIPT),
                    "--target",
                    "antigravity",
                    "--mode",
                    "copy",
                    "--project-dir",
                    str(project_dir),
                ],
                cwd=str(REPO_ROOT),
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stdout + "\n" + result.stderr)
            self.assertIn("CLI", result.stdout)
            self.assertIn("antigravity", result.stdout)
            self.assertTrue(
                (project_dir / ".agents" / "skills" / "research-paper-workflow" / "SKILL.md").exists()
            )
            self.assertTrue(
                (project_dir / ".agent" / "skills" / "research-paper-workflow" / "SKILL.md").exists()
            )
            self.assertTrue(
                (antigravity_home / "skills" / "research-paper-workflow" / "SKILL.md").exists()
            )

    def test_all_target_reports_install_hints_for_missing_clis(self) -> None:
        if not SYSTEM_BASH.exists():
            self.skipTest("/bin/bash is not available")

        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            home_dir = temp_root / "home"
            home_dir.mkdir()

            env = os.environ.copy()
            env["HOME"] = str(home_dir)
            env["PATH"] = "/usr/bin:/bin"
            env["NO_COLOR"] = "1"

            result = subprocess.run(
                [
                    str(SYSTEM_BASH),
                    str(INSTALL_SCRIPT),
                    "--target",
                    "all",
                    "--mode",
                    "copy",
                    "--project-dir",
                    str(project_dir),
                ],
                cwd=str(REPO_ROOT),
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stdout + "\n" + result.stderr)
            self.assertIn("codex CLI not found in PATH", result.stdout)
            self.assertIn("Install the Codex CLI from the official OpenAI distribution", result.stdout)
            self.assertIn("claude CLI not found in PATH", result.stdout)
            self.assertIn("npm install -g @anthropic-ai/claude-code", result.stdout)
            self.assertIn("gemini CLI not found in PATH", result.stdout)
            self.assertIn("npm install -g @google/gemini-cli", result.stdout)
            self.assertIn("antigravity CLI not found in PATH", result.stdout)
            self.assertIn("Install Antigravity and ensure the `antigravity` binary is on PATH", result.stdout)

    def test_parts_project_skips_global_skill_copy(self) -> None:
        if not SYSTEM_BASH.exists():
            self.skipTest("/bin/bash is not available")

        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            claude_home = temp_root / "claude-home"
            home_dir = temp_root / "home"
            home_dir.mkdir()

            env = os.environ.copy()
            env["HOME"] = str(home_dir)
            env["CLAUDE_CODE_HOME"] = str(claude_home)
            env["NO_COLOR"] = "1"

            result = subprocess.run(
                [
                    str(SYSTEM_BASH),
                    str(INSTALL_SCRIPT),
                    "--target",
                    "claude",
                    "--mode",
                    "copy",
                    "--project-dir",
                    str(project_dir),
                    "--parts",
                    "project",
                ],
                cwd=str(REPO_ROOT),
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stdout + "\n" + result.stderr)
            self.assertIn("parts:   project", result.stdout)
            self.assertTrue((project_dir / ".agent" / "workflows" / "proofread.md").exists())
            self.assertTrue((project_dir / "CLAUDE.md").exists())
            self.assertTrue((project_dir / ".env").exists())
            self.assertFalse((claude_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())


if __name__ == "__main__":
    unittest.main()
