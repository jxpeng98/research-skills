from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class AcademicContextContinuityTests(unittest.TestCase):
    def test_workflow_contract_defines_context_artifacts_and_refresh_points(self) -> None:
        content = (REPO_ROOT / "standards" / "research-workflow-contract.yaml").read_text(
            encoding="utf-8"
        )

        for token in (
            '"context/research_state.md"',
            '"context/decision_log.md"',
            "academic_context_continuity:",
            "research_state_required_sections:",
            "decision_log_required_fields:",
            'A: "Lock the working research question',
            'E: "Distinguish stable findings from tentative signals',
            'H: "Summarize reviewer-sensitive weaknesses',
        ):
            self.assertIn(token, content)

    def test_capability_map_wires_continuity_skill_into_stage_close_tasks(self) -> None:
        content = (REPO_ROOT / "standards" / "mcp-agent-capability-map.yaml").read_text(
            encoding="utf-8"
        )

        for token in (
            "academic-context-maintainer:",
            '"context/research_state.md"',
            '"context/decision_log.md"',
        ):
            self.assertIn(token, content)

        for task_id in ("A5", "B6", "C5", "D3", "E5", "F6", "H4"):
            self.assertRegex(
                content,
                rf"{task_id}:\n(?:\s+.+\n)+?\s+- \"academic-context-maintainer\"",
            )

    def test_continuity_skill_and_templates_define_academic_state_not_runtime_state(self) -> None:
        skill = (
            REPO_ROOT / "skills" / "Z_cross_cutting" / "academic-context-maintainer.md"
        ).read_text(encoding="utf-8")
        template = (REPO_ROOT / "templates" / "research-state.md").read_text(encoding="utf-8")
        log_template = (REPO_ROOT / "templates" / "decision-log.md").read_text(encoding="utf-8")

        for token in (
            "This skill is not a runtime memory compactor.",
            "`compact`",
            "`handoff trace`",
            "`resume_state`",
            "Stage Refresh Matrix",
            "`context/research_state.md`",
            "`context/decision_log.md`",
            "locked decisions",
            "unresolved disputes",
        ):
            self.assertIn(token, skill)

        self.assertIn("## Current Evidence Position", template)
        self.assertIn("## Active Risks and Fragility Points", template)
        self.assertIn("## Source Artifact Anchors", template)
        self.assertIn("| Decision ID | Stage | Status | Decision |", log_template)
        self.assertIn("Revisit Trigger", log_template)

    def test_generated_workflow_contract_reference_includes_academic_context_section(self) -> None:
        content = (
            REPO_ROOT / "research-paper-workflow" / "references" / "workflow-contract.md"
        ).read_text(encoding="utf-8")

        for token in (
            "## Academic Context Continuity",
            "artifact: `context/research_state.md`",
            "artifact: `context/decision_log.md`",
            "### `context/research_state.md` must preserve",
            "### `context/decision_log.md` must preserve",
        ):
            self.assertIn(token, content)


if __name__ == "__main__":
    unittest.main()
