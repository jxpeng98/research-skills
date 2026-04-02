from __future__ import annotations

import unittest
from datetime import datetime, timezone
from pathlib import Path

from research_skills.team_run_acceptance import (
    render_team_run_receipt,
    summarize_team_run_result,
)

REPO_ROOT = Path(__file__).resolve().parents[1]


class TeamRunAcceptanceTests(unittest.TestCase):
    def test_summarize_team_run_result_extracts_metrics_and_observations(self) -> None:
        raw_result = {
            "task_description": "H3 empirical acceptance-probe",
            "confidence": 0.0,
            "merged_analysis": (
                "## Team-Run: H3 (run_id=test)\n\n"
                "- Partition strategy: by_reviewer_persona\n"
                "- Work units dispatched: 3\n"
                "- Successful shards: 0/3\n"
                "- Barrier status: blocked\n"
                "- Consensus policy: union_with_priority\n"
                "- Profile: default\n\n"
                "## Routing Notes\n"
                "- Worker domain_expert failed (claude): claude CLI not found in PATH. Please install it first.\n"
                "- Worker reviewer_2 failed (gemini): gemini CLI not found in PATH. Please install it first.\n"
                "- Barrier policy=block: halting because not all workers succeeded.\n"
            ),
        }

        summary = summarize_team_run_result(raw_result, ["python3", "-m", "bridges.orchestrator"], 0)

        self.assertEqual(summary.task_id, "H3")
        self.assertEqual(summary.paper_type, "empirical")
        self.assertEqual(summary.topic, "acceptance-probe")
        self.assertEqual(summary.barrier_status, "blocked")
        self.assertEqual(summary.work_units, "3")
        self.assertEqual(summary.successful_shards, "0/3")
        self.assertEqual(len(summary.blocking_observations), 2)

    def test_render_team_run_receipt_includes_summary_and_json(self) -> None:
        raw_result = {
            "task_description": "B1 systematic-review acceptance-probe",
            "confidence": 0.0,
            "merged_analysis": (
                "## Team-Run: B1 (run_id=test)\n\n"
                "- Work units dispatched: 1\n"
                "- Successful shards: 0/1\n"
                "- Barrier status: blocked\n"
                "- Profile: default\n\n"
                "## Routing Notes\n"
                "- Worker batch_1 failed (codex): network error\n"
            ),
        }
        summary = summarize_team_run_result(raw_result, ["python3", "cmd"], 0)
        receipt = render_team_run_receipt(
            summary,
            generated_at=datetime(2026, 4, 2, 10, 53, tzinfo=timezone.utc),
            git_commit="abc123",
        )

        self.assertIn("# Team-Run Acceptance Receipt — B1", receipt)
        self.assertIn("Barrier Status: blocked", receipt)
        self.assertIn("Worker batch_1 failed (codex): network error", receipt)
        self.assertIn('"task_description": "B1 systematic-review acceptance-probe"', receipt)

    def test_collaboration_docs_and_receipts_are_present(self) -> None:
        guide_en = (REPO_ROOT / "guides" / "advanced" / "agent-skill-collaboration.md").read_text(encoding="utf-8")
        doc_en = (REPO_ROOT / "docs" / "advanced" / "agent-skill-collaboration.md").read_text(encoding="utf-8")
        guide_zh = (REPO_ROOT / "guides" / "advanced" / "agent-skill-collaboration_CN.md").read_text(encoding="utf-8")
        doc_zh = (REPO_ROOT / "docs" / "zh" / "advanced" / "agent-skill-collaboration.md").read_text(encoding="utf-8")

        self.assertEqual(guide_en, doc_en)
        self.assertEqual(guide_zh, doc_zh)
        self.assertIn("scripts/capture_team_run_acceptance.py", guide_en)
        self.assertIn("scripts/capture_team_run_acceptance.py", guide_zh)

        self.assertTrue((REPO_ROOT / "release" / "acceptance" / "team-run-b1-local-receipt.md").exists())
        self.assertTrue((REPO_ROOT / "release" / "acceptance" / "team-run-h3-local-receipt.md").exists())


if __name__ == "__main__":
    unittest.main()
