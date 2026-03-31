from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
BOOTSTRAP_SCRIPT = REPO_ROOT / "scripts" / "bootstrap_research_skill.sh"
POWERSHELL_BOOTSTRAP = REPO_ROOT / "scripts" / "bootstrap_research_skill.ps1"
SYSTEM_BASH = Path("/bin/bash")


class BootstrapResearchSkillTests(unittest.TestCase):
    def test_partial_profile_dry_run_skips_cli_and_doctor(self) -> None:
        if not SYSTEM_BASH.exists():
            self.skipTest("/bin/bash is not available")

        with tempfile.TemporaryDirectory() as tmp_dir:
            result = subprocess.run(
                [
                    str(SYSTEM_BASH),
                    str(BOOTSTRAP_SCRIPT),
                    "--profile",
                    "partial",
                    "--project-dir",
                    tmp_dir,
                    "--dry-run",
                ],
                cwd=str(REPO_ROOT),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

        self.assertEqual(result.returncode, 0, msg=result.stdout + "\n" + result.stderr)
        self.assertIn("profile: partial", result.stdout)
        self.assertIn("cli:     skip", result.stdout)
        self.assertNotIn("--install-cli", result.stdout)
        self.assertNotIn("--doctor", result.stdout)

    def test_full_profile_dry_run_enables_cli_and_doctor(self) -> None:
        if not SYSTEM_BASH.exists():
            self.skipTest("/bin/bash is not available")

        with tempfile.TemporaryDirectory() as tmp_dir:
            result = subprocess.run(
                [
                    str(SYSTEM_BASH),
                    str(BOOTSTRAP_SCRIPT),
                    "--profile",
                    "full",
                    "--project-dir",
                    tmp_dir,
                    "--dry-run",
                ],
                cwd=str(REPO_ROOT),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

        self.assertEqual(result.returncode, 0, msg=result.stdout + "\n" + result.stderr)
        self.assertIn("profile: full", result.stdout)
        self.assertIn("cli:     install ->", result.stdout)
        self.assertIn("--install-cli", result.stdout)
        self.assertIn("--doctor", result.stdout)

    def test_missing_profile_in_noninteractive_mode_fails_fast(self) -> None:
        if not SYSTEM_BASH.exists():
            self.skipTest("/bin/bash is not available")

        with tempfile.TemporaryDirectory() as tmp_dir:
            result = subprocess.run(
                [
                    str(SYSTEM_BASH),
                    str(BOOTSTRAP_SCRIPT),
                    "--project-dir",
                    tmp_dir,
                ],
                cwd=str(REPO_ROOT),
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=False,
            )

        self.assertEqual(result.returncode, 2, msg=result.stdout + "\n" + result.stderr)
        self.assertIn("Missing --profile and no interactive terminal is available", result.stderr)

    def test_powershell_bootstrap_is_manifest_driven(self) -> None:
        content = POWERSHELL_BOOTSTRAP.read_text(encoding="utf-8")

        self.assertIn("install\\install_manifest.tsv", content)
        self.assertIn("Expand-Archive", content)
        self.assertIn("Install-FromRepo", content)
        self.assertIn("[switch]$Beta", content)
        self.assertIn("Invoke-NativeChecked", content)
        self.assertIn("Ensure-PathEntry", content)
        self.assertIn('SetEnvironmentVariable("Path"', content)
        self.assertIn("Out-Host", content)
        self.assertIn("$PSVersionTable.PSVersion.Major -lt 7", content)
        self.assertIn("Microsoft.PowerShell", content)
        self.assertIn('[string]$SourceRepo = ""', content)
        self.assertIn("$sourceRepoRoot", content)
        self.assertIn("/releases?per_page=20", content)
        self.assertNotIn('Install-FromRepo "C:\\dry-run\\research-skills"', content)
        self.assertIn("[dry-run] Install workflow assets into client directories", content)
        self.assertNotIn('bootstrapUrl = "https://raw.githubusercontent.com', content)
        self.assertNotIn('$content = @"', content)
        self.assertIn('$env:PYTHONPATH = $RepoRoot', content)

    def test_shell_bootstrap_documents_beta_channel(self) -> None:
        content = BOOTSTRAP_SCRIPT.read_text(encoding="utf-8")

        self.assertIn("--beta", content)
        self.assertIn("--source-repo", content)
        self.assertIn("source:   local checkout", content)
        self.assertIn("latest beta/prerelease tag", content)
        self.assertIn("/releases?per_page=20", content)
        self.assertIn("persist_shell_path_entries", content)
        self.assertIn(".zshrc", content)
        self.assertIn(".bashrc", content)
        self.assertIn(".profile", content)


if __name__ == "__main__":
    unittest.main()
