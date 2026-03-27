from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = REPO_ROOT / "install" / "install_manifest.tsv"


def _read_manifest() -> list[dict[str, str]]:
    entries: list[dict[str, str]] = []
    for raw_line in MANIFEST_PATH.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        target, op, label, source, destination = raw_line.split("\t")
        entries.append(
            {
                "target": target,
                "op": op,
                "label": label,
                "source": source,
                "destination": destination,
            }
        )
    return entries


class InstallManifestTests(unittest.TestCase):
    def test_manifest_declares_expected_entries(self) -> None:
        entries = _read_manifest()
        expected = {
            ("codex", "Skill"),
            ("claude", "Skill"),
            ("claude", "Workflows"),
            ("claude", "CLAUDE.md"),
            ("gemini", "Skill"),
            ("gemini", "Quickstart"),
            ("gemini", "Profiles"),
            ("antigravity", "Workspace Skill"),
            ("antigravity", "Legacy Skill"),
            ("antigravity", "Global Skill"),
            ("project", "Env"),
        }

        actual = {(entry["target"], entry["label"]) for entry in entries}
        self.assertEqual(actual, expected)

    def test_manifest_operations_and_sources_are_valid(self) -> None:
        entries = _read_manifest()
        allowed_ops = {
            "dir-copy",
            "file-copy",
            "glob-copy",
            "claude-template",
            "quickstart-file",
            "conditional-dir-copy",
        }
        allowed_vars = {
            "${PROJECT_DIR}",
            "${CODEX_HOME}",
            "${CLAUDE_CODE_HOME}",
            "${GEMINI_HOME}",
            "${ANTIGRAVITY_HOME}",
        }

        for entry in entries:
            self.assertIn(entry["op"], allowed_ops, msg=entry)
            source = REPO_ROOT / entry["source"]
            if "*" in entry["source"]:
                matches = list(REPO_ROOT.glob(entry["source"]))
                self.assertTrue(matches, msg=f"glob did not match: {entry['source']}")
            else:
                self.assertTrue(source.exists(), msg=f"missing source: {entry['source']}")

            destination = entry["destination"]
            self.assertTrue(destination.startswith("${"), msg=f"destination should be templated: {destination}")
            self.assertTrue(any(var in destination for var in allowed_vars), msg=f"unknown template var: {destination}")

    def test_manifest_has_no_duplicate_target_destinations(self) -> None:
        entries = _read_manifest()
        seen: set[tuple[str, str]] = set()
        for entry in entries:
            key = (entry["target"], entry["destination"])
            self.assertNotIn(key, seen, msg=f"duplicate manifest destination: {key}")
            seen.add(key)


if __name__ == "__main__":
    unittest.main()
