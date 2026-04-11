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
    def test_same_version_install_reports_current_and_source_versions(self) -> None:
        if not SYSTEM_BASH.exists():
            self.skipTest("/bin/bash is not available")

        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            home_dir = temp_root / "home"
            home_dir.mkdir()
            codex_home = temp_root / "codex-home"
            existing_skill = codex_home / "skills" / "research-paper-workflow"
            existing_skill.mkdir(parents=True)
            source_version = (REPO_ROOT / "research-paper-workflow" / "VERSION").read_text(encoding="utf-8").strip()
            (existing_skill / "SKILL.md").write_text(
                "---\nname: research-paper-workflow\ndescription: current\n---\n",
                encoding="utf-8",
            )
            (existing_skill / "VERSION").write_text(f"{source_version}\n", encoding="utf-8")

            env = os.environ.copy()
            env["HOME"] = str(home_dir)
            env["CODEX_HOME"] = str(codex_home)
            env["NO_COLOR"] = "1"

            result = subprocess.run(
                [
                    str(SYSTEM_BASH),
                    str(INSTALL_SCRIPT),
                    "--target",
                    "codex",
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
            self.assertIn("Detected Versions", result.stdout)
            self.assertIn(f"source:      {source_version}", result.stdout)
            self.assertIn(f"codex:       {source_version}", result.stdout)
            self.assertIn(f"current {source_version}; source {source_version}; already installed", result.stdout)

    def test_doctor_command_exports_repo_root_on_pythonpath(self) -> None:
        content = INSTALL_SCRIPT.read_text(encoding="utf-8")

        self.assertIn('doctor_cmd() {', content)
        self.assertIn('PYTHONPATH="$pythonpath" python3 -m bridges.orchestrator "$@"', content)

    def test_claude_install_defaults_to_global_only_under_system_bash(self) -> None:
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
            self.assertFalse((project_dir / ".agent" / "workflows" / "proofread.md").exists())
            self.assertFalse((project_dir / ".agent" / "workflows" / "study-design.md").exists())
            self.assertFalse((project_dir / "CLAUDE.md").exists())
            self.assertFalse((project_dir / ".env").exists())
            self.assertNotIn("Env", result.stdout)
            self.assertTrue((claude_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())

    def test_antigravity_install_defaults_to_global_skill_only_when_cli_exists(self) -> None:
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
            self.assertFalse((project_dir / ".agents" / "skills" / "research-paper-workflow" / "SKILL.md").exists())
            self.assertFalse((project_dir / ".agent" / "skills" / "research-paper-workflow" / "SKILL.md").exists())
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
            # Project parts now only installs .env
            self.assertTrue((project_dir / ".env").exists())
            # Workflows and CLAUDE.md are no longer installed project-locally
            self.assertFalse((project_dir / ".agent" / "workflows" / "proofread.md").exists())
            self.assertFalse((project_dir / "CLAUDE.md").exists())
            self.assertFalse((claude_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())

    def test_existing_managed_install_auto_upgrades_without_overwrite(self) -> None:
        if not SYSTEM_BASH.exists():
            self.skipTest("/bin/bash is not available")

        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            home_dir = temp_root / "home"
            home_dir.mkdir()
            codex_home = temp_root / "codex-home"
            cli_dir = temp_root / "bin"
            cli_dir.mkdir()

            existing_skill = codex_home / "skills" / "research-paper-workflow"
            existing_skill.mkdir(parents=True)
            (existing_skill / "SKILL.md").write_text(
                "---\nname: research-paper-workflow\ndescription: legacy\n---\n",
                encoding="utf-8",
            )
            (existing_skill / "VERSION").write_text("v0.4.0-beta.14\n", encoding="utf-8")

            (cli_dir / "research-skills").write_text(
                "#!/usr/bin/env bash\nCLI_FLAVOR=\"shell-bootstrap\"\n# legacy\nresearch-skills <command>\n",
                encoding="utf-8",
            )
            (cli_dir / "research-skills-bootstrap").write_text(
                "#!/usr/bin/env bash\nDEFAULT_REPO=\"jxpeng98/research-skills\"\n# legacy\n--profile <partial|full>\n",
                encoding="utf-8",
            )
            (cli_dir / "rsk").symlink_to(cli_dir / "research-skills")
            (cli_dir / "rsw").symlink_to(cli_dir / "research-skills")

            env = os.environ.copy()
            env["HOME"] = str(home_dir)
            env["CODEX_HOME"] = str(codex_home)
            env["RESEARCH_SKILLS_BIN_DIR"] = str(cli_dir)
            env["PATH"] = f"{cli_dir}{os.pathsep}/usr/bin:/bin"
            env["NO_COLOR"] = "1"

            result = subprocess.run(
                [
                    str(SYSTEM_BASH),
                    str(INSTALL_SCRIPT),
                    "--target",
                    "codex",
                    "--mode",
                    "copy",
                    "--project-dir",
                    str(project_dir),
                    "--parts",
                    "globals,cli",
                ],
                cwd=str(REPO_ROOT),
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stdout + "\n" + result.stderr)
            self.assertIn("updated v0.4.0-beta.14", result.stdout)
            self.assertIn("already linked", result.stdout)
            self.assertEqual(
                (existing_skill / "VERSION").read_text(encoding="utf-8").strip(),
                (REPO_ROOT / "research-paper-workflow" / "VERSION").read_text(encoding="utf-8").strip(),
            )
            self.assertIn('CLI_FLAVOR="shell-bootstrap"', (cli_dir / "research-skills").read_text(encoding="utf-8"))
            self.assertIn('DEFAULT_REPO="jxpeng98/research-skills"', (cli_dir / "research-skills-bootstrap").read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
