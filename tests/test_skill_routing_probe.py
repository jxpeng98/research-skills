from contextlib import redirect_stderr, redirect_stdout
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
from evals.skill_routing import resource_reader


def trace(response):
    return '\n'.join(map(json.dumps, [
        {"type": "thread.started", "thread_id": "synthetic"}, {"type": "turn.started"},
        {"type": "item.completed", "item": {"id": "1", "type": "agent_message", "text": json.dumps(response)}},
        {"type": "turn.completed"},
    ]))


def read_trace(response, resources, path):
    events = list(map(json.loads, trace(response).splitlines()))
    item = {"id": "read-1", "type": "mcp_tool_call", "server": resource_reader.SERVER,
            "tool": resource_reader.TOOL, "arguments": {"path": path}}
    events[2:2] = [{"type": "item.started", "item": {**item, "status": "in_progress"}},
                   {"type": "item.completed", "item": {**item, "status": "completed",
                    "result": resource_reader.read_result(resources, item["arguments"]), "error": None}}]
    return "\n".join(map(json.dumps, events))


def store_trace(output, case_id, response, sources):
    directory = output / case_id
    directory.mkdir(exist_ok=True)
    (directory / "events.jsonl").write_text(trace(response), encoding="utf-8")
    cases = probe.load_cases(source=sources["corpus"], legacy="resource_route" not in response)
    probe.write_json(directory / "capture.json", {
        "exit_code": 0, "events_sha256": probe.digest(directory / "events.jsonl"),
        "prompt_sha256": probe.sha(probe.prompt(cases[case_id], sources["entry"], sources["instruction"])),
    })


class SkillRoutingProbeTests(unittest.TestCase):
    def test_invalid_cli_report_never_starts_paid_capture(self):
        with tempfile.TemporaryDirectory() as temporary, redirect_stderr(io.StringIO()):
            root = Path(temporary)
            with patch.object(probe, "capture") as capture:
                for report in (root, root / "capture"):
                    self.assertEqual(1, probe.main(["capture", str(root / "capture"), "--report", str(report)]))
                capture.assert_not_called()

    def capture(self, root, *, raw=None, no_skill=False, read_resources=False):
        root.mkdir(parents=True, exist_ok=True)
        cases = probe.load_cases()
        selected = list(cases)[:2]
        response = {key: values[0] for key, values in cases[selected[0]]["expected"].items()}
        response["answer"] = "Synthetic grader check, not a real Host answer."
        if no_skill:
            response.update(route="none", resource_route="none")
        calls = []

        def codex(cmd, **kwargs):
            if cmd == ["codex", "--version"]:
                return subprocess.CompletedProcess(cmd, 0, "codex-cli synthetic", "")
            calls.append((cmd, kwargs))
            text = trace(response) if raw is None else raw
            if read_resources and raw is None:
                resources = json.loads((output / "resources.json").read_text())
                text = read_trace(response, resources, response["route"])
            return subprocess.CompletedProcess(cmd, 0, text, "")

        (root / "config.toml").write_text('model="configured-test-model"\nmodel_reasoning_effort="high"\n')
        output = root / "capture"
        with patch.dict(os.environ, {"CODEX_HOME": str(root)}), patch.object(
            probe.subprocess, "run", side_effect=codex
        ), redirect_stdout(io.StringIO()):
            probe.capture(output, selected, no_skill=no_skill, read_resources=read_resources)
            with self.assertRaises(FileExistsError):
                probe.capture(output, selected, no_skill=no_skill, read_resources=read_resources)
        return output, selected, response, calls

    def test_resource_capture_requires_actual_matching_reads(self):
        with tempfile.TemporaryDirectory() as temporary, redirect_stdout(io.StringIO()):
            root = Path(temporary)
            output, selected, response, calls = self.capture(root, read_resources=True)
            self.assertTrue(probe.score(output, root / "scores"))
            manifest, sources, cases = probe.captured_inputs(output, None)
            self.assertEqual("codex-resource-reading-v1", manifest["kind"])
            self.assertIn(f'mcp_servers.{resource_reader.SERVER}.enabled_tools=["read_resource"]', calls[0][0])
            summary = json.loads((root / "scores/summary.json").read_text())
            self.assertEqual([response["route"]], summary["resource_reads"][selected[0]]["paths"])
            # A claimed read with a successful text-only trace is insufficient.
            store_trace(output, selected[0], {**response, "answer": "I read the whole card."}, sources)
            self.assertFalse(probe.score(output, root / "claimed-only"))
            changed = json.loads(sources["resources"])
            changed[response["route"]] = "tampered guidance"
            probe.write_json(output / "resources.json", changed)
            with self.assertRaisesRegex(ValueError, "snapshot changed"):
                probe.score(output, root / "tampered-snapshot")

    def test_resource_mcp_fails_closed_for_bad_results_calls_and_paths(self):
        resources = {"workflows/paper-read.md": "public guidance"}
        response = {"route": "workflows/paper-read.md", "resource_route": "none", "scope": "direct",
                    "next_action": "answer", "answer": "Synthetic answer"}
        raw = read_trace(response, resources, response["route"])
        filtered, reads = resource_reader.observed_reads(raw, resources)
        self.assertEqual(response, probe.trace_observation(filtered, 0))
        self.assertEqual([response["route"]], reads)
        with self.assertRaises(ValueError):
            probe.trace_observation(raw, 0)  # Tool-free intent lane remains strict.
        events = list(map(json.loads, raw.splitlines()))
        variants = [events[:2] + events[3:], events[:3] + events[4:], [events[2], *events],
                    events[:4] + [events[3]] + events[4:]]
        for field, value in (("server", "unexpected"), ("tool", "project_write"),
                             ("status", "failed"), ("arguments", {"path": "../../private"}),
                             ("result", {"content": [{"type": "text", "text": "forged"}]})):
            changed = json.loads(json.dumps(events))
            changed[3]["item"][field] = value
            variants.append(changed)
        for field, value in (("status", "failed"), ("status", None), ("error", {"message": "denied"}), ("result", {})):
            changed = json.loads(json.dumps(events))
            changed[2]["item"][field] = value
            variants.append(changed)
        for value in (True, "true", 0, None):
            changed = json.loads(json.dumps(events))
            changed[3]["item"]["result"]["isError"] = value
            variants.append(changed)
        for invalid in variants:
            with self.assertRaises(ValueError):
                resource_reader.observed_reads("\n".join(map(json.dumps, invalid)), resources)
        for arguments in ({"path": "/etc/passwd"}, {"path": "../private"}, {"path": "missing"}, {"path": 3}, {}):
            result = resource_reader.dispatch({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                       "params": {"name": "read_resource", "arguments": arguments}}, resources)
            self.assertTrue(result["result"]["isError"])

    def test_resource_reader_stdio_only_serves_snapshotted_content(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "resources.json"
            probe.write_json(path, {"card.md": "Synthetic public guidance"})
            requests = [
                {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-11-25"}},
                {"jsonrpc": "2.0", "method": "notifications/initialized"},
                {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
                {"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "read_resource", "arguments": {"path": "card.md"}}},
            ]
            result = subprocess.run([probe.sys.executable, resource_reader.__file__, str(path)],
                                    input="\n".join(map(json.dumps, requests)) + "\n",
                                    capture_output=True, text=True, check=True)
            replies = list(map(json.loads, result.stdout.splitlines()))
            self.assertEqual([1, 2, 3], [reply["id"] for reply in replies])
            self.assertEqual(["read_resource"], [tool["name"] for tool in replies[1]["result"]["tools"]])
            self.assertEqual("Synthetic public guidance", replies[2]["result"]["structuredContent"]["text"])

    def test_capture_preserves_model_and_stops_on_invalid_trace(self):
        with tempfile.TemporaryDirectory() as temporary, redirect_stdout(io.StringIO()):
            root = Path(temporary)
            output, selected, _, calls = self.capture(root, raw='{"type":"error"}\n')
            self.assertFalse(probe.score(output, root / "scores"))
            self.assertEqual(1, len(calls))  # No second model call after invalid evidence.
            cmd, kwargs = calls[0]
            for setting in ('model="configured-test-model"', 'model_reasoning_effort="high"',
                            'approval_policy="never"', 'web_search="disabled"', '--ignore-user-config'):
                self.assertIn(setting, cmd)
            self.assertEqual("read-only", cmd[cmd.index("--sandbox") + 1])
            self.assertNotIn(selected[0], kwargs["input"])
            self.assertFalse((output / selected[1]).exists())
            summary = json.loads((root / "scores/summary.json").read_text())
            self.assertEqual(2, summary["case_count"])
            self.assertEqual(0, summary["labels"]["scope"]["passed"])

    def test_snapshot_scoring_and_regrade_keep_original_evidence(self):
        with tempfile.TemporaryDirectory() as temporary, redirect_stdout(io.StringIO()):
            root = Path(temporary)
            output, selected, response, _ = self.capture(root)
            manifest, sources, cases = probe.captured_inputs(output, None)
            before = {str(p.relative_to(output)): p.read_bytes() for p in output.rglob("*") if p.is_file()}
            # Changed current product/producer files do not invalidate captured inputs.
            with patch.object(probe, "ENTRY", root / "no-current-entry"):
                self.assertTrue(probe.score(output, root / "original"))
            with self.assertRaises(FileExistsError):
                probe.score(output, root / "original")
            groups = yaml.safe_load(sources["corpus"])
            groups[0]["expected"]["scope"] = ["formal"]
            grading = root / "grading.yaml"
            grading.write_text(yaml.safe_dump(groups), encoding="utf-8")
            self.assertFalse(probe.score(output, root / "regrade", corpus=grading))
            original = json.loads((root / "original/summary.json").read_text())
            revised = json.loads((root / "regrade/summary.json").read_text())
            self.assertEqual(2, original["passed_cases"])
            self.assertEqual(0, revised["passed_cases"])
            self.assertEqual({"passed": 2, "total": 2}, revised["labels"]["route"])
            self.assertEqual(before, {str(p.relative_to(output)): p.read_bytes() for p in output.rglob("*") if p.is_file()})
            groups[0]["request"]["en"] += " changed"
            grading.write_text(yaml.safe_dump(groups), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "expectations only"):
                probe.score(output, root / "changed-request", corpus=grading)
            for selection in ([], [selected[0], selected[0]], ["../escape"], [None]):
                probe.write_json(output / "manifest.json", {**manifest, "cases": selection})
                with self.subTest(selection=selection), self.assertRaises(ValueError):
                    probe.score(output, root / "invalid-selection")
            probe.write_json(output / "manifest.json", manifest)
            receipt = output / selected[0] / "capture.json"
            saved = receipt.read_text()
            metadata = json.loads(saved)
            metadata["prompt_sha256"] = "changed"
            probe.write_json(receipt, metadata)
            self.assertFalse(probe.score(output, root / "changed-prompt"))
            receipt.write_text(saved)
            raw = output / selected[0] / "events.jsonl"
            raw.write_text(raw.read_text() + '\n{}')
            self.assertFalse(probe.score(output, root / "changed-trace"))
            store_trace(output, selected[0], {**response, "route": "unregistered-route"}, sources)
            self.assertFalse(probe.score(output, root / "wrong-label"))
            (output / "entry.md").write_text("changed")
            with self.assertRaisesRegex(ValueError, "snapshot changed"):
                probe.score(output, root / "changed-entry")

    def test_no_skill_has_only_common_label_metrics(self):
        with tempfile.TemporaryDirectory() as temporary, redirect_stdout(io.StringIO()):
            root = Path(temporary)
            output, _, _, _ = self.capture(root, no_skill=True)
            self.assertTrue(probe.score(output, root / "scores"))
            summary = json.loads((root / "scores/summary.json").read_text())
            self.assertEqual({"scope", "next_action"}, set(summary["labels"]))
            self.assertEqual(["route", "resource_route"], summary["unassessed_labels"])

    def test_legacy_hash_binding_and_partial_adjudication_never_execute_producer(self):
        with tempfile.TemporaryDirectory() as temporary, redirect_stdout(io.StringIO()):
            root = Path(temporary)
            groups = yaml.safe_load(probe.CORPUS.read_text())[:1]
            del groups[0]["expected"]["resource_route"]
            sources = {"entry": "historical entry", "corpus": yaml.safe_dump(groups),
                       "probe": 'INSTRUCTION = "historical instruction"\nraise RuntimeError("never execute")\n'}
            manifest = {"kind": "codex-supplied-entry-intent-v1", "source": {k: probe.sha(v) for k, v in sources.items()},
                        "cases": [groups[0]["id"] + "-en"], "corpus_count": 2}
            probe.write_json(root / "manifest.json", manifest)
            expected = groups[0]["expected"]
            response = {key: values[0] for key, values in expected.items()}
            response.update(answer="Synthetic legacy answer.", scope="formal")
            store_trace(root, manifest["cases"][0], response, {**sources, "instruction": "historical instruction"})
            rules = {"version": "legacy-1.1", "base_corpus_sha256": [manifest["source"]["corpus"]],
                     "cases": {groups[0]["id"]: {"reason": "Synthetic test adjudication", "expected": {"scope": ["formal"]}}}}
            adjudications = root / "adjudications.yaml"
            adjudications.write_text(yaml.safe_dump(rules))
            with self.assertRaisesRegex(ValueError, "legacy-ref"):
                probe.score(root, root / "missing-ref")
            with patch.object(probe, "git_sources", return_value=("pinned-test-commit", sources.copy())):
                self.assertFalse(probe.score(root, root / "original", legacy_ref="test"))
                self.assertTrue(probe.score(root, root / "adjudicated", legacy_ref="test", adjudications=adjudications))
                summary = json.loads((root / "adjudicated/summary.json").read_text())
                self.assertEqual("legacy-post-hoc-adjudication", summary["grading"]["kind"])
                self.assertNotIn("resource_route", summary["labels"])
                self.assertEqual(["direct"], summary["original_expectations"][manifest["cases"][0]]["scope"])
                with self.assertRaisesRegex(ValueError, "cannot be upgraded"):
                    probe.score(root, root / "upgrade", legacy_ref="test", corpus=probe.CORPUS)
                rules["cases"][groups[0]["id"]]["expected"]["resource_route"] = ["none"]
                adjudications.write_text(yaml.safe_dump(rules))
                with self.assertRaises(ValueError):
                    probe.score(root, root / "invalid-override", legacy_ref="test", adjudications=adjudications)
            with patch.object(probe, "git_sources", return_value=("wrong", {**sources, "entry": "wrong"})):
                with self.assertRaisesRegex(ValueError, "hashes"):
                    probe.score(root, root / "wrong-ref", legacy_ref="wrong")

    def test_corpus_and_traces_fail_closed_without_leaking_labels(self):
        cases = probe.load_cases()
        self.assertEqual(48, len(cases))
        self.assertEqual({"en", "zh"}, {case["language"] for case in cases.values()})
        case = next(iter(cases.values()))
        response = {key: values[0] for key, values in case["expected"].items()}
        response["answer"] = "Synthetic answer."
        raw = trace(response)
        self.assertEqual(response, probe.trace_observation(raw, 0))
        bad_traces = ["", "not json", json.dumps([]), raw + "\n{}", raw.rsplit("\n", 1)[0],
                      raw + '\n{"type":"turn.completed"}', '{"type":"error"}\n' + raw,
                      '{"type":"item.started","item":{"type":"command_execution"}}\n' + raw,
                      '{"type":"item.completed","item":{"type":"new_unknown_tool"}}\n' + raw]
        for invalid in bad_traces:
            with self.subTest(trace=invalid), self.assertRaises(ValueError):
                probe.trace_observation(invalid, 0)
        with self.assertRaises(ValueError):
            probe.trace_observation(raw, 1)
        groups = yaml.safe_load(probe.CORPUS.read_text(encoding="utf-8"))
        for invalid in ([], groups + groups, [{**groups[0], "id": "../escape"}]):
            with self.assertRaises(ValueError):
                probe.load_cases(source=yaml.safe_dump(invalid))
        groups[0]["expected"]["route"] = ["skills-summary.md"]
        with self.assertRaisesRegex(ValueError, "discovery indexes"):
            probe.load_cases(source=yaml.safe_dump(groups))
        del groups[0]["expected"]["resource_route"]
        self.assertEqual(2, len(probe.load_cases(source=yaml.safe_dump(groups[:1]), legacy=True)))
        sample = dict(case, expected={"secret-label": ["DO_NOT_LEAK"]}, category="DO_NOT_LEAK")
        self.assertNotIn("DO_NOT_LEAK", probe.prompt(sample))


if __name__ == "__main__":
    unittest.main()
