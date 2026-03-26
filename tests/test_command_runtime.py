from __future__ import annotations

import unittest
from unittest import mock

from bridges import command_runtime


class CommandRuntimeTests(unittest.TestCase):
    def test_split_command_uses_posix_rules_on_unix(self) -> None:
        with mock.patch.object(command_runtime.os, "name", "posix"):
            parsed = command_runtime.split_command('python3 "scripts/mcp_demo.py" --flag')

        self.assertEqual(parsed, ["python3", "scripts/mcp_demo.py", "--flag"])

    def test_split_command_unwraps_windows_quotes(self) -> None:
        command = '"C:\\Python\\python.exe" "D:\\a\\repo\\script.py" "D:\\a\\repo\\fixture.json"'
        with mock.patch.object(command_runtime.os, "name", "nt"):
            parsed = command_runtime.split_command(command)

        self.assertEqual(
            parsed,
            [
                "C:\\Python\\python.exe",
                "D:\\a\\repo\\script.py",
                "D:\\a\\repo\\fixture.json",
            ],
        )

    def test_format_command_quotes_windows_paths_with_spaces(self) -> None:
        with mock.patch.object(command_runtime.os, "name", "nt"):
            command = command_runtime.format_command(
                "C:\\Program Files\\Python\\python.exe",
                "D:\\a\\repo\\script.py",
            )

        self.assertEqual(
            command,
            '"C:\\Program Files\\Python\\python.exe" D:\\a\\repo\\script.py',
        )


if __name__ == "__main__":
    unittest.main()
