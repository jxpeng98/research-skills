from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


class CrossPlatformRoutingGrillContractTests(unittest.TestCase):
    def test_canonical_workflow_declares_cross_platform_trigger_contract(self) -> None:
        skill_text = read(LAYOUT.workflow / "SKILL.md")
        routing_text = read(LAYOUT.workflow / "references" / "platform-routing.md")
        combined = skill_text + "\n" + routing_text

        for phrase in (
            "Cross-Platform Trigger Contract",
            "Ambiguity Trigger",
            "academic research lifecycle",
            "does not require explicit",
            "Codex",
            "Claude",
            "CLI",
            "qiongli_orchestrator_route",
            "不知道怎么做",
            "not sure",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, combined)

    def test_stage_grill_contract_is_all_stage_and_cross_stage(self) -> None:
        boundary_text = read(LAYOUT.skills / "Z_cross_cutting" / "boundary-interviewer.md")
        critique_text = read(LAYOUT.skills / "Z_cross_cutting" / "self-critique.md")
        handoff_text = read(LAYOUT.workflow / "references" / "stage-handoff-contract.md")
        combined = boundary_text + "\n" + critique_text + "\n" + handoff_text

        for phrase in (
            "Stage-Aware Grill Contract",
            "Cross-Stage Grill Memory",
            "light automatic grill",
            "deep grill",
            "Open Grill Issues",
            "Resolved Grill Decisions",
            "Revisit Triggers",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, combined)

        for stage in ("Stage A", "Stage B", "Stage C", "Stage D", "Stage E", "Stage F", "Stage G", "Stage J", "Stage H", "Stage I", "Stage K"):
            with self.subTest(stage=stage):
                self.assertIn(stage, combined)

        self.assertNotIn("Future trigger stages", boundary_text)

    def test_direct_workflow_skill_and_agent_usage_declares_writing_harness(self) -> None:
        paths = (
            REPO_ROOT / "content" / "skills-core.md",
            LAYOUT.workflow / "SKILL.md",
            LAYOUT.workflow / "workflows" / "paper-write.md",
            LAYOUT.workflow / "workflows" / "academic-write.md",
            LAYOUT.workflow / "references" / "stage-F-writing.md",
            LAYOUT.skills / "F_writing" / "manuscript-architect.md",
            LAYOUT.roles / "science-writer.yaml",
        )
        combined = "\n".join(read(path) for path in paths)

        for phrase in (
            "Writing Harness Contract",
            "Story Spine",
            "write -> review -> confirm",
            "do not draft the whole artifact in one uninterrupted pass",
            "mainline drift",
            "generic or vague claims",
            "next blocking boundary/grill question",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, combined)

        skills_core = read(REPO_ROOT / "content" / "skills-core.md")
        for phrase in (
            "Writing Harness Contract",
            "Story Spine",
            "write -> review -> confirm",
            "mainline drift",
        ):
            with self.subTest(skills_core_phrase=phrase):
                self.assertIn(phrase, skills_core)

        for role_name in ("science-writer.yaml", "research-orchestrator.yaml", "pi.yaml"):
            role_text = read(LAYOUT.roles / role_name)
            with self.subTest(role=role_name):
                self.assertIn("Writing Harness Contract", role_text)
                self.assertIn("write -> review -> confirm", role_text)

    def test_writing_role_uses_academic_writer_name_with_legacy_alias(self) -> None:
        role_text = read(LAYOUT.roles / "science-writer.yaml")
        capability_text = read(LAYOUT.standards / "mcp-agent-capability-map.yaml")

        for phrase in (
            "id: academic-writer",
            'display_name: "Academic Writer"',
            "legacy_ids:",
            "science-writer",
            "aliases:",
            "scholarly-writer",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, role_text)

        self.assertIn('mapped_role: "academic-writer"', capability_text)

    def test_stage_i_declares_academic_analysis_code_constraints(self) -> None:
        paths = (
            LAYOUT.workflow / "references" / "stage-I-code.md",
            LAYOUT.workflow / "workflows" / "code-build.md",
            LAYOUT.skills / "I_code" / "code-builder.md",
            LAYOUT.skills / "I_code" / "code-specification.md",
            LAYOUT.skills / "I_code" / "code-planning.md",
        )
        combined = "\n".join(read(path) for path in paths)

        for phrase in (
            "Academic Analysis Code",
            "estimand",
            "dataset lineage",
            "model diagnostics",
            "manuscript-facing",
            "service layers",
            "controllers",
            "analysis_plan_source",
            "manuscript_outputs",
            "robustness_checks",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, combined)

    def test_materialized_plugin_contains_routing_and_grill_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "dist-source"
            result = subprocess.run(
                [
                    sys.executable,
                    "scripts/materialize_distribution_payloads.py",
                    "--target",
                    "plugin",
                    "--out",
                    str(out),
                    "--force",
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(result.returncode, 0, msg=result.stderr + result.stdout)

            plugin_skill = out / "plugins" / "qiongli" / "skills" / "qiongli-workflow"
            skill_text = read(plugin_skill / "SKILL.md")
            self.assertIn("references/platform-routing.md", skill_text)
            routing_text = read(plugin_skill / "references" / "platform-routing.md")
            boundary_text = read(plugin_skill / "skills" / "Z_cross_cutting" / "boundary-interviewer.md")
            paper_write_text = read(plugin_skill / "workflows" / "paper-write.md")
            manuscript_text = read(plugin_skill / "skills" / "F_writing" / "manuscript-architect.md")
            science_writer_text = read(plugin_skill / "roles" / "science-writer.yaml")
            orchestrator_role_text = read(plugin_skill / "roles" / "research-orchestrator.yaml")
            pi_role_text = read(plugin_skill / "roles" / "pi.yaml")

        for phrase in (
            "Cross-Platform Trigger Contract",
            "Ambiguity Trigger",
            "Stage-Aware Grill Contract",
            "Cross-Stage Grill Memory",
            "Writing Harness Contract",
            "write -> review -> confirm",
            "mainline drift",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(
                    phrase,
                    "\n".join(
                        [
                            skill_text,
                            routing_text,
                            boundary_text,
                            paper_write_text,
                            manuscript_text,
                            science_writer_text,
                            orchestrator_role_text,
                            pi_role_text,
                        ]
                    ),
                )


if __name__ == "__main__":
    unittest.main()
