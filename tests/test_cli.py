from __future__ import annotations

import argparse
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from research_skills import cli as cli_module


REPO_ROOT = Path(__file__).resolve().parents[1]


class InstallerCliTests(unittest.TestCase):
    def test_init_defaults_to_project_part(self) -> None:
        args = argparse.Namespace(
            project_dir=".",
            target="all",
            mode="copy",
            overwrite=False,
            doctor=False,
            dry_run=False,
            parts=None,
        )

        with mock.patch.object(cli_module, "install", return_value=0) as install_mock:
            exit_code = cli_module.cmd_init(args)

        self.assertEqual(exit_code, 0)
        options = install_mock.call_args.args[0]
        self.assertEqual(options.parts, ("project",))
        self.assertEqual(options.target, "all")

    def test_align_describes_global_first_upgrade_and_project_init(self) -> None:
        args = argparse.Namespace(repo="owner/repo")

        with mock.patch("builtins.print") as print_mock:
            exit_code = cli_module.cmd_align(args)

        self.assertEqual(exit_code, 0)
        lines = [" ".join(str(part) for part in call.args) for call in print_mock.call_args_list]
        joined = "\n".join(lines)
        self.assertIn("What `", joined)
        self.assertIn("upgrade` modifies by default", joined)
        self.assertIn("Use `rsk init --project-dir .` for project bootstrap", joined)
        self.assertIn("--parts project", joined)

    def test_upgrade_passes_parts_to_installer(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            extracted_root = temp_root / "archive-root"
            scripts_dir = extracted_root / "scripts"
            scripts_dir.mkdir(parents=True)
            (scripts_dir / "bootstrap_research_skill.py").write_text("# stub\n", encoding="utf-8")

            args = argparse.Namespace(
                repo="owner/repo",
                ref="v0.4.0",
                ref_type="tag",
                target="all",
                beta=False,
                mode="copy",
                project_dir=str(temp_root / "project"),
                overwrite=True,
                doctor=False,
                dry_run=False,
                parts="project,cli",
            )

            with mock.patch.object(cli_module, "_download") as download_mock, mock.patch.object(
                cli_module, "_extract_tarball", return_value=extracted_root
            ), mock.patch.object(cli_module, "install", return_value=0) as install_mock:
                exit_code = cli_module.cmd_upgrade(args)

        self.assertEqual(exit_code, 0)
        download_mock.assert_called_once()
        options = install_mock.call_args.args[0]
        self.assertEqual(options.parts, ("project", "cli"))

    def test_doctor_runs_orchestrator_subprocess(self) -> None:
        args = argparse.Namespace(cwd=".")
        completed = mock.Mock(returncode=0, stdout="doctor ok\n")

        with mock.patch.object(cli_module.subprocess, "run", return_value=completed) as run_mock:
            exit_code = cli_module.cmd_doctor(args)

        self.assertEqual(exit_code, 0)
        run_mock.assert_called_once()
        command = run_mock.call_args.args[0]
        self.assertEqual(command[:3], [cli_module.sys.executable, "-m", "bridges.orchestrator"])
        self.assertEqual(command[3:], ["doctor", "--cwd", str(Path(".").resolve())])


if __name__ == "__main__":
    unittest.main()
