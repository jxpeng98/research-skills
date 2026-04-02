from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from bridges.command_runtime import current_python_command
from bridges.providers.overlay_runtime import invoke_overlay_json


class OverlayRuntimeTests(unittest.TestCase):
    def test_invoke_overlay_json_returns_not_configured_without_env(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            payload = {"provider": "demo"}
            with mock.patch.dict(os.environ, {}, clear=False):
                parsed, info = invoke_overlay_json(
                    env_name="RESEARCH_MCP_FAKE_OVERLAY_CMD",
                    payload=payload,
                    cwd=Path(tmp_dir),
                    timeout_seconds=1,
                    label="Fake overlay",
                )

        self.assertIsNone(parsed)
        self.assertEqual(info, {"configured": False})

    def test_invoke_overlay_json_reports_non_json_output(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            script = root / "emit_text.py"
            script.write_text("print('not-json')\n", encoding="utf-8")
            with mock.patch.dict(
                os.environ,
                {"RESEARCH_MCP_FAKE_OVERLAY_CMD": current_python_command(str(script))},
                clear=False,
            ):
                parsed, info = invoke_overlay_json(
                    env_name="RESEARCH_MCP_FAKE_OVERLAY_CMD",
                    payload={"provider": "demo"},
                    cwd=root,
                    timeout_seconds=5,
                    label="Fake overlay",
                )

        self.assertIsNone(parsed)
        self.assertTrue(info["configured"])
        self.assertEqual(info["status"], "warning")
        self.assertIn("non-JSON", info["summary"])

    def test_invoke_overlay_json_executes_quoted_command_with_space_paths(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir) / "workspace with spaces"
            root.mkdir(parents=True)
            script = root / "emit fixture.py"
            fixture = root / "overlay fixture.json"
            fixture.write_text(
                '{"status":"ok","summary":"fixture overlay","data":{"echo":"ok"}}',
                encoding="utf-8",
            )
            script.write_text(
                "from pathlib import Path\n"
                "import json\n"
                "import sys\n"
                "payload = json.loads(sys.stdin.read())\n"
                "data = json.loads(Path(sys.argv[1]).read_text(encoding='utf-8'))\n"
                "data['data']['provider'] = payload.get('provider')\n"
                "print(json.dumps(data))\n",
                encoding="utf-8",
            )
            with mock.patch.dict(
                os.environ,
                {
                    "RESEARCH_MCP_FAKE_OVERLAY_CMD": current_python_command(
                        str(script),
                        str(fixture),
                    )
                },
                clear=False,
            ):
                parsed, info = invoke_overlay_json(
                    env_name="RESEARCH_MCP_FAKE_OVERLAY_CMD",
                    payload={"provider": "demo-overlay"},
                    cwd=root,
                    timeout_seconds=5,
                    label="Fake overlay",
                )

        self.assertEqual(info, {"configured": True})
        self.assertIsNotNone(parsed)
        self.assertEqual(parsed["status"], "ok")
        self.assertEqual(parsed["data"]["echo"], "ok")
        self.assertEqual(parsed["data"]["provider"], "demo-overlay")


if __name__ == "__main__":
    unittest.main()
