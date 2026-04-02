from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from bridges.providers.fulltext_retrieval import merge_external_resolution_payload, run_fulltext_retrieval


FIXTURES_DIR = Path(__file__).resolve().parent / "fixtures"


class FulltextRetrievalTests(unittest.TestCase):
    def test_external_resolution_prefers_resolver_for_retrieved_local_copy(self) -> None:
        local_manifest = [
            {
                "record_id": "s2:1",
                "citekey": "smith2024platform",
                "doi": "10.1000/platform-governance",
                "retrieval_status": "not_retrieved:oa_candidate",
                "version_label": "",
                "source_provider": "semantic_scholar_oa_candidate",
                "retrieved_at": "2026-03-25T00:00:00+00:00",
                "fulltext_path": "",
                "access_url": "https://example.com/platform-governance.pdf",
                "license": "",
                "notes": "Candidate access URL identified locally.",
            }
        ]
        zotero_payload = json.loads(
            (FIXTURES_DIR / "fulltext_resolution_zotero.json").read_text(encoding="utf-8")
        )

        merged_rows, resolution_info = merge_external_resolution_payload(
            local_manifest,
            zotero_payload,
        )

        self.assertEqual(resolution_info["status"], "ok")
        self.assertEqual(resolution_info["provenance"], ["zotero://select/library/items/ABC123"])
        self.assertEqual(resolution_info["contract_version"], "resolver_manifest_overlay_v1")
        self.assertTrue(resolution_info["merge_trace"])
        self.assertEqual(len(merged_rows), 1)
        row = merged_rows[0]
        self.assertEqual(row["retrieval_status"], "retrieved_oa")
        self.assertEqual(row["source_provider"], "zotero")
        self.assertEqual(row["fulltext_path"], "fulltext/platform-governance.pdf")
        self.assertEqual(row["license"], "CC-BY-4.0")
        self.assertIn("Candidate access URL identified locally.", row["notes"])
        self.assertIn("Resolved from Zotero attachment.", row["notes"])

    def test_external_resolution_accepts_wrapper_alias_fields(self) -> None:
        local_manifest = [
            {
                "record_id": "s2:1",
                "citekey": "smith2024platform",
                "doi": "10.1000/platform-governance",
                "retrieval_status": "not_retrieved:oa_candidate",
                "version_label": "",
                "source_provider": "semantic_scholar_oa_candidate",
                "retrieved_at": "2026-03-25T00:00:00+00:00",
                "fulltext_path": "",
                "access_url": "https://example.com/platform-governance.pdf",
                "license": "",
                "notes": "Candidate access URL identified locally.",
            }
        ]
        wrapper_payload = {
            "status": "ok",
            "summary": "Wrapper resolver fixture",
            "provenance": ["fixture://resolver-wrapper"],
            "data": {
                "manifest_rows": [
                    {
                        "reference_id": "s2:1",
                        "resolver": "zotero",
                        "status": "retrieved_oa",
                        "version": "accepted_manuscript",
                        "pdf_path": "fulltext/platform-governance.pdf",
                        "url": "https://resolver.example.org/platform-governance.pdf",
                        "rights": "CC-BY-4.0",
                        "notes": "Resolved through wrapper alias fields.",
                    }
                ]
            },
        }

        merged_rows, resolution_info = merge_external_resolution_payload(
            local_manifest,
            wrapper_payload,
        )

        self.assertEqual(resolution_info["contract_version"], "resolver_manifest_overlay_v1")
        self.assertTrue(resolution_info["merge_trace"])
        row = merged_rows[0]
        self.assertEqual(row["retrieval_status"], "retrieved_oa")
        self.assertEqual(row["source_provider"], "zotero")
        self.assertEqual(row["version_label"], "accepted_manuscript")
        self.assertEqual(row["fulltext_path"], "fulltext/platform-governance.pdf")
        self.assertEqual(row["access_url"], "https://resolver.example.org/platform-governance.pdf")
        self.assertEqual(row["license"], "CC-BY-4.0")
        self.assertIn("Candidate access URL identified locally.", row["notes"])
        self.assertIn("Resolved through wrapper alias fields.", row["notes"])
        self.assertTrue(
            any(
                item["field"] == "retrieval_status"
                and item["decision"] == "prefer_candidate"
                and item["candidate_provider"] == "zotero"
                for item in resolution_info["merge_trace"]
            )
        )
        self.assertTrue(
            any(
                item["field"] == "notes"
                and item["decision"] == "append_notes"
                and item["candidate_provider"] == "zotero"
                for item in resolution_info["merge_trace"]
            )
        )

    def test_run_fulltext_retrieval_exposes_resolution_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            project_root = root / "RESEARCH" / "demo-topic"
            project_root.mkdir(parents=True)
            (project_root / "search_results.csv").write_text(
                "record_id,source,query_id,retrieved_at,paper_id,title,authors,year,venue,doi,url,abstract,open_access_pdf_url\n"
                "s2:1,semantic_scholar,q1,2026-03-25T00:00:00+00:00,S2-1,Platform Governance in Practice,\"Smith, Alex\",2024,Organization Science,10.1000/platform-governance,https://example.com/landing,Abstract text,https://example.com/platform-governance.pdf\n"
                "s2:2,semantic_scholar,q1,2026-03-25T00:00:00+00:00,S2-2,Capability Routines in Digital Platforms,\"Taylor Chen\",2025,Academy of Management Journal,10.1000/capability-routines,https://example.com/capability-routines,Abstract text,\n",
                encoding="utf-8",
            )

            output = run_fulltext_retrieval(
                {
                    "topic": "demo-topic",
                    "artifact_root": "RESEARCH/[topic]/",
                    "required_outputs": ["search_results.csv"],
                },
                root,
                retrieved_at="2026-03-25T00:00:00+00:00",
            )

        bundle = output["data"]["resolution_bundle"]
        self.assertEqual(bundle["bundle_version"], "fulltext_resolution_bundle_v1")
        self.assertEqual(bundle["default_mode"], "planning_stub_with_optional_overlay")
        self.assertEqual(bundle["resolver_contract_version"], "resolver_manifest_overlay_v1")
        self.assertFalse(bundle["resolver_configured"])
        self.assertEqual(bundle["pending_count"], 2)
        self.assertEqual(bundle["resolved_count"], 0)
        self.assertTrue(
            any(item["step"] == "resolve_oa_candidate" for item in bundle["next_actions"])
        )
        self.assertTrue(
            any(item["step"] == "resolve_via_library_or_provider" for item in bundle["next_actions"])
        )


if __name__ == "__main__":
    unittest.main()
