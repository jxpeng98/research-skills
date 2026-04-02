from __future__ import annotations

import unittest

from bridges.codex_bridge import CodexBridge


class CodexBridgeTests(unittest.TestCase):
    def test_parse_output_returns_content_when_agent_messages_exist(self) -> None:
        bridge = CodexBridge()
        response = bridge.parse_output(
            [
                '{"type":"thread.started","thread_id":"abc"}',
                '{"type":"event","item":{"type":"agent_message","text":"Hello"}}',
                '{"type":"turn.completed"}',
            ]
        )

        self.assertTrue(response.success)
        self.assertEqual(response.session_id, "abc")
        self.assertEqual(response.content, "Hello")

    def test_parse_output_surfaces_last_error_when_no_agent_messages_arrive(self) -> None:
        bridge = CodexBridge()
        response = bridge.parse_output(
            [
                'WARNING: non-json startup line',
                '{"type":"thread.started","thread_id":"abc"}',
                '{"type":"error","message":"Reconnecting... dns lookup failed"}',
            ]
        )

        self.assertFalse(response.success)
        self.assertEqual(response.session_id, "abc")
        self.assertIn("Codex did not emit agent messages.", response.error or "")
        self.assertIn("dns lookup failed", response.error or "")


if __name__ == "__main__":
    unittest.main()
