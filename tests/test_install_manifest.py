from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = REPO_ROOT / "install" / "install_manifest.tsv"
PACKAGED_MANIFEST_PATH = REPO_ROOT / "research_skills" / "install_manifest.tsv"


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
    def test_packaged_manifest_matches_repo_manifest(self) -> None:
        self.assertTrue(PACKAGED_MANIFEST_PATH.exists(), msg="missing packaged install manifest")
        self.assertEqual(
            PACKAGED_MANIFEST_PATH.read_text(encoding="utf-8"),
            MANIFEST_PATH.read_text(encoding="utf-8"),
        )

    def test_manifest_declares_expected_entries(self) -> None:
        entries = _read_manifest()
        expected = {
            ("codex", "Skill"),
            ("claude", "Skill"),
            ("gemini", "Skill"),
            ("antigravity", "Skill"),
            ("project", "Env"),
        }

        actual = {(entry["target"], entry["label"]) for entry in entries}
        self.assertEqual(actual, expected)

    def test_manifest_operations_and_sources_are_valid(self) -> None:
        entries = _read_manifest()
        allowed_ops = {
            "dir-copy",
            "file-copy",
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

    def test_all_global_skill_entries_use_dir_copy(self) -> None:
        """All global skill entries should use dir-copy with the research-paper-workflow source."""
        entries = _read_manifest()
        for entry in entries:
            if entry["target"] in {"codex", "claude", "gemini", "antigravity"}:
                self.assertEqual(entry["op"], "dir-copy", msg=f"expected dir-copy for {entry}")
                self.assertEqual(entry["source"], "research-paper-workflow", msg=f"unexpected source for {entry}")

    def test_no_project_dir_in_global_entries(self) -> None:
        """Global install entries must not reference PROJECT_DIR."""
        entries = _read_manifest()
        for entry in entries:
            if entry["target"] in {"codex", "claude", "gemini", "antigravity"}:
                self.assertNotIn("${PROJECT_DIR}", entry["destination"], msg=f"global entry references PROJECT_DIR: {entry}")

    def test_skill_source_contains_bundled_workflows(self) -> None:
        """The skill source directory must contain bundled workflows."""
        skill_src = REPO_ROOT / "research-paper-workflow"
        workflows_dir = skill_src / "workflows"
        self.assertTrue(workflows_dir.is_dir(), msg="missing workflows directory in skill source")
        workflow_files = list(workflows_dir.glob("*.md"))
        self.assertGreaterEqual(len(workflow_files), 10, msg=f"expected at least 10 workflow files, found {len(workflow_files)}")
        # Verify key workflows are present
        expected_workflows = {"paper.md", "lit-review.md", "paper-read.md", "find-gap.md", "academic-write.md"}
        actual_names = {f.name for f in workflow_files}
        self.assertTrue(expected_workflows.issubset(actual_names), msg=f"missing key workflows: {expected_workflows - actual_names}")

    def test_skill_source_is_self_contained(self) -> None:
        """After sync, the skill package must be self-contained with all required assets."""
        pkg = REPO_ROOT / "research-paper-workflow"
        # skills-core.md
        self.assertTrue((pkg / "skills-core.md").is_file(), msg="missing skills-core.md in skill package")
        # skills/ directory with stage subdirectories
        skills_dir = pkg / "skills"
        self.assertTrue(skills_dir.is_dir(), msg="missing skills/ directory in skill package")
        for stage in ("A_framing", "B_literature", "F_writing", "I_code"):
            self.assertTrue((skills_dir / stage).is_dir(), msg=f"missing skills/{stage}/ in skill package")
        # templates/ directory
        templates_dir = pkg / "templates"
        self.assertTrue(templates_dir.is_dir(), msg="missing templates/ directory in skill package")
        for tpl in ("manuscript-outline.md", "cover-letter.md", "ethics-irb-pack.md"):
            self.assertTrue((templates_dir / tpl).is_file(), msg=f"missing templates/{tpl}")
        # CLAUDE.project.md should NOT be in templates (excluded by sync)
        self.assertFalse(
            (templates_dir / "CLAUDE.project.md").exists(),
            msg="CLAUDE.project.md should be excluded from bundled templates",
        )
        # standards/ directory
        standards_dir = pkg / "standards"
        self.assertTrue(standards_dir.is_dir(), msg="missing standards/ directory in skill package")
        self.assertTrue((standards_dir / "research-workflow-contract.yaml").is_file())
        self.assertTrue((standards_dir / "mcp-agent-capability-map.yaml").is_file())
        # roles/ directory
        roles_dir = pkg / "roles"
        self.assertTrue(roles_dir.is_dir(), msg="missing roles/ directory in skill package")
        self.assertGreaterEqual(
            len(list(roles_dir.glob("*.yaml"))), 5,
            msg="expected at least 5 role files",
        )


if __name__ == "__main__":
    unittest.main()
