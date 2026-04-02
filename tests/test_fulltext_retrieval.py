from __future__ import annotations

import json
import unittest
from pathlib import Path

from bridges.providers.fulltext_retrieval import merge_external_resolution_payload


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


if __name__ == "__main__":
    unittest.main()
