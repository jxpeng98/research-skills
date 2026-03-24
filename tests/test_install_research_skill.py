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
            self.assertTrue((claude_home / "skills" / "research-paper-workflow" / "SKILL.md").exists())


if __name__ == "__main__":
    unittest.main()
