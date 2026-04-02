from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from bridges.providers.screening_tracker import run_screening_tracker


class ScreeningTrackerTests(unittest.TestCase):
    def test_builtin_screening_tracker_derives_resume_state_from_local_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            project_root = root / "RESEARCH" / "demo-topic"
            screening_dir = project_root / "screening"
            screening_dir.mkdir(parents=True)
            (screening_dir / "title_abstract.md").write_text(
                "# Title/Abstract Screening\n\n"
                "| ID | Title | Year | Include | Exclude Reason |\n"
                "|---|---|---|---|---|\n"
                "| 1 | Platform Governance in Practice | 2024 | ✓ | |\n"
                "| 2 | Capability Routines in Digital Platforms | 2025 | ? | Need full-text |\n",
                encoding="utf-8",
            )
            (screening_dir / "full_text.md").write_text(
                "# Full-text Screening\n\n"
                "| record_id | decision (include/exclude) | exclusion_reason | fulltext_status | notes |\n"
                "|---|---|---|---|---|\n"
                "| s2:1 | include | | retrieved_oa | Include in synthesis |\n",
                encoding="utf-8",
            )
            (project_root / "retrieval_manifest.csv").write_text(
                "record_id,citekey,doi,retrieval_status,version_label,source_provider,retrieved_at,fulltext_path,access_url,license,notes\n"
                "s2:1,smith2024platform,10.1000/platform-governance,retrieved_oa,published,zotero,2026-03-25T00:00:00+00:00,fulltext/platform-governance.pdf,https://example.com/platform-governance.pdf,CC-BY-4.0,Resolved\n"
                "s2:2,chen2025capability,10.1000/capability-routines,not_retrieved:needs_provider,,builtin_fulltext_stub,2026-03-25T00:00:00+00:00,,https://example.com/capability-routines,,Needs resolver\n",
                encoding="utf-8",
            )

            output = run_screening_tracker(
                {
                    "topic": "demo-topic",
                    "artifact_root": "RESEARCH/[topic]/",
                },
                root,
            )

        self.assertEqual(output["status"], "warning")
        self.assertEqual(output["data"]["provider_mode"], "builtin_screening_checkpoint_stub")
        self.assertTrue(output["data"]["checkpoint_state"]["title_abstract"]["exists"])
        self.assertTrue(output["data"]["checkpoint_state"]["full_text"]["exists"])
        self.assertTrue(output["data"]["checkpoint_state"]["retrieval_manifest"]["exists"])
        self.assertEqual(output["data"]["resume_state"]["bundle_status"], "in_progress")
        self.assertEqual(output["data"]["resume_state"]["pending_retrieval_records"], 1)
        self.assertEqual(output["data"]["resume_state"]["pending_full_text_decisions"], 0)
        self.assertTrue(
            any(
                item["step"] == "fulltext_retrieval"
                and item["pending_records"] == 1
                for item in output["data"]["resume_state"]["next_actions"]
            )
        )

    def test_builtin_screening_tracker_flags_prisma_refresh_when_bundle_is_otherwise_complete(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            project_root = root / "RESEARCH" / "demo-topic"
            screening_dir = project_root / "screening"
            screening_dir.mkdir(parents=True)
            (screening_dir / "title_abstract.md").write_text(
                "# Title/Abstract Screening\n\n"
                "| ID | Title | Include |\n"
                "|---|---|---|\n"
                "| 1 | Platform Governance in Practice | ✓ |\n",
                encoding="utf-8",
            )
            (screening_dir / "full_text.md").write_text(
                "# Full-text Screening\n\n"
                "| record_id | decision | exclusion_reason | fulltext_status | notes |\n"
                "|---|---|---|---|---|\n"
                "| s2:1 | include | | retrieved_oa | Include in synthesis |\n",
                encoding="utf-8",
            )
            (project_root / "retrieval_manifest.csv").write_text(
                "record_id,citekey,doi,retrieval_status,version_label,source_provider,retrieved_at,fulltext_path,access_url,license,notes\n"
                "s2:1,smith2024platform,10.1000/platform-governance,retrieved_oa,published,zotero,2026-03-25T00:00:00+00:00,fulltext/platform-governance.pdf,https://example.com/platform-governance.pdf,CC-BY-4.0,Resolved\n",
                encoding="utf-8",
            )

            output = run_screening_tracker(
                {
                    "topic": "demo-topic",
                    "artifact_root": "RESEARCH/[topic]/",
                },
                root,
            )

        self.assertEqual(output["data"]["resume_state"]["bundle_status"], "in_progress")
        self.assertTrue(
            any(
                item["step"] == "prisma_flow_refresh"
                for item in output["data"]["resume_state"]["next_actions"]
            )
        )


if __name__ == "__main__":
    unittest.main()
