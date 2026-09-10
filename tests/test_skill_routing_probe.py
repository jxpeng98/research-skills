from contextlib import redirect_stdout
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import yaml

from evals.skill_routing import probe


class SkillRoutingProbeTests(unittest.TestCase):
    def test_capture_preserves_model_and_stops_on_invalid_trace(self):
        cases = probe.load_cases()
        selected = list(cases)[:2]
        calls = []

        def codex(cmd, **kwargs):
            if cmd == ["codex", "--version"]:
                return subprocess.CompletedProcess(cmd, 0, "codex-cli synthetic", "")
            calls.append((cmd, kwargs))
            return subprocess.CompletedProcess(cmd, 0, '{"type":"error"}\n', "")

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "config.toml").write_text('model="configured-test-model"\nmodel_reasoning_effort="high"\n')
            output = root / "capture"
            with patch.dict(os.environ, {"CODEX_HOME": str(root)}), patch.object(
                probe.subprocess, "run", side_effect=codex
            ), redirect_stdout(io.StringIO()):
                probe.capture(output, selected, cases)
                self.assertFalse(probe.score(output, cases))
                with self.assertRaises(FileExistsError):
                    probe.capture(output, selected, cases)
            self.assertEqual(1, len(calls))  # No second model call after invalid evidence.
            cmd, kwargs = calls[0]
            for setting in ('model="configured-test-model"', 'model_reasoning_effort="high"',
                            'approval_policy="never"', 'web_search="disabled"', '--ignore-user-config'):
                self.assertIn(setting, cmd)
            self.assertEqual("read-only", cmd[cmd.index("--sandbox") + 1])
            self.assertNotIn(selected[0], kwargs["input"])
            manifest = json.loads((output / "manifest.json").read_text())
            self.assertEqual(selected, manifest["cases"])
            self.assertFalse((output / selected[1]).exists())

    def test_corpus_and_capture_fail_closed(self):
        cases = probe.load_cases()
        self.assertEqual(48, len(cases))
        self.assertEqual({"en", "zh"}, {case["language"] for case in cases.values()})
        case_id, case = next(iter(cases.items()))
        response = {key: values[0] for key, values in case["expected"].items()}
        response["answer"] = "Synthetic grader check, not a real Host answer."
        events = [{"type": "thread.started", "thread_id": "synthetic"},
                  {"type": "turn.started"},
                  {"type": "item.completed", "item": {"id": "1", "type": "agent_message",
                                                        "text": json.dumps(response)}},
                  {"type": "turn.completed"}]
        raw = "\n".join(map(json.dumps, events))
        self.assertEqual(response, probe.trace_observation(raw, 0))
        bad_traces = ["", "not json", json.dumps([]), raw + "\n{}",
                      "\n".join(map(json.dumps, events[:-1])),
                      raw + "\n" + json.dumps({"type": "turn.completed"}),
                      json.dumps({"type": "error"}) + "\n" + raw,
                      json.dumps({"type": "item.started", "item": {
                          "id": "tool", "type": "command_execution", "command": "true"}}) + "\n" + raw,
                      json.dumps({"type": "item.completed", "item": {
                          "id": "tool", "type": "new_unknown_tool"}}) + "\n" + raw]
        for invalid in bad_traces:
            with self.subTest(trace=invalid), self.assertRaises(ValueError):
                probe.trace_observation(invalid, 0)
        with self.assertRaises(ValueError):
            probe.trace_observation(raw, 1)

        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            manifest = {"kind": "codex-supplied-entry-intent-v1", "source": probe.bindings(),
                        "cases": [case_id]}
            probe.write_json(output / "manifest.json", manifest)
            with redirect_stdout(io.StringIO()):
                self.assertFalse(probe.score(output, cases))  # Missing evidence.
            directory = output / case_id
            directory.mkdir()

            def store_trace(text):
                (directory / "events.jsonl").write_text(text, encoding="utf-8")
                probe.write_json(directory / "capture.json", {
                    "exit_code": 0, "events_sha256": probe.digest(directory / "events.jsonl"),
                    "prompt_sha256": hashlib.sha256(probe.prompt(case).encode()).hexdigest(),
                })

            store_trace(raw)
            with redirect_stdout(io.StringIO()):
                self.assertTrue(probe.score(output, cases))
                wrong_route = raw.replace(response["route"], "unregistered-route")
                store_trace(wrong_route)
                self.assertFalse(probe.score(output, cases))
                store_trace(raw)
                (directory / "events.jsonl").write_text(wrong_route, encoding="utf-8")
                self.assertFalse(probe.score(output, cases))  # Altered trace digest.
            for selection in ([], [case_id, case_id], ["../escape"], [None]):
                probe.write_json(output / "manifest.json", {**manifest, "cases": selection})
                with self.subTest(selection=selection), self.assertRaises(ValueError):
                    probe.score(output, cases)
            manifest["source"]["entry"] = "stale"
            probe.write_json(output / "manifest.json", manifest)
            with self.assertRaises(ValueError):
                probe.score(output, cases)

            groups = yaml.safe_load(probe.CORPUS.read_text(encoding="utf-8"))
            corpus = output / "cases.yaml"
            for invalid in ([], groups + groups, [{**groups[0], "id": "../escape"}]):
                corpus.write_text(yaml.safe_dump(invalid), encoding="utf-8")
                with self.assertRaises(ValueError):
                    probe.load_cases(corpus)
            sample = dict(case, expected={"secret-label": ["DO_NOT_LEAK"]}, category="DO_NOT_LEAK")
            self.assertNotIn("DO_NOT_LEAK", probe.prompt(sample))


if __name__ == "__main__":
    unittest.main()
