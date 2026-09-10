from __future__ import annotations

import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]


class AcademicContextContinuityTests(unittest.TestCase):
    def test_workflow_contract_defines_context_artifacts_and_refresh_points(self) -> None:
        content = (RepoLayout(REPO_ROOT).standards / "research-workflow-contract.yaml").read_text(
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

    def test_boundary_review_is_academic_context_artifact(self) -> None:
        content = (RepoLayout(REPO_ROOT).standards / "research-workflow-contract.yaml").read_text(
            encoding="utf-8"
        )
        self.assertIn('"context/boundary_review.md"', content)
        self.assertIn("boundary_review_required_sections:", content)
        self.assertIn("claim_strength_boundary", content)
        self.assertIn("revisit_trigger", content)

    def test_context_maintainer_consumes_boundary_review(self) -> None:
        content = (
            RepoLayout(REPO_ROOT).skills / "Z_cross_cutting" / "academic-context-maintainer.md"
        ).read_text(encoding="utf-8")
        self.assertIn("context/boundary_review.md", content)
        self.assertIn("Do not broaden locked boundaries", content)
        self.assertIn("research_state.md", content)
        self.assertIn("decision_log.md", content)
        self.assertIn("stage_handoff.md", content)

    def test_workflow_and_context_skill_preserve_graph_readable_continuity(self) -> None:
        workflow = (RepoLayout(REPO_ROOT).workflow / "SKILL.md").read_text(encoding="utf-8")
        maintainer = (
            RepoLayout(REPO_ROOT).skills / "Z_cross_cutting" / "academic-context-maintainer.md"
        ).read_text(encoding="utf-8")

        for content in (workflow, maintainer):
            for token in (
                "contribution_claim",
                "Decision ID",
                "Cluster ID",
                "Claim ID",
                "project refresh",
                "qiongli_project_graph_snapshot",
            ):
                self.assertIn(token, content)

    def test_graph_continuity_reference_defines_safe_semantic_repair(self) -> None:
        workflow = (RepoLayout(REPO_ROOT).workflow / "SKILL.md").read_text(encoding="utf-8")
        maintainer = (
            RepoLayout(REPO_ROOT).skills / "Z_cross_cutting" / "academic-context-maintainer.md"
        ).read_text(encoding="utf-8")
        reference = (
            RepoLayout(REPO_ROOT).workflow / "references" / "academic-graph-continuity.md"
        ).read_text(encoding="utf-8")

        for content in (workflow, maintainer):
            self.assertIn("references/academic-graph-continuity.md", content)

        for token in (
            "Preserve existing narrative prose",
            "Preview the exact files, records, stable IDs, and source anchors",
            "Never invent",
            "`semanticNodeCount`",
            "non-`contains`",
            "qiongli project graph doctor --project-id <prj_id>",
            "`qiongli_project_graph_snapshot`",
            "`graph/semantic_links.jsonl`",
        ):
            self.assertIn(token, reference)

    def test_capability_map_wires_continuity_skill_into_stage_close_tasks(self) -> None:
        content = (RepoLayout(REPO_ROOT).standards / "mcp-agent-capability-map.yaml").read_text(
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
            RepoLayout(REPO_ROOT).skills / "Z_cross_cutting" / "academic-context-maintainer.md"
        ).read_text(encoding="utf-8")
        template = (RepoLayout(REPO_ROOT).templates / "research-state.md").read_text(encoding="utf-8")
        log_template = (RepoLayout(REPO_ROOT).templates / "decision-log.md").read_text(encoding="utf-8")

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
            RepoLayout(REPO_ROOT).workflow / "references" / "workflow-contract.md"
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
