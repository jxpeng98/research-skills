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
        self.assertEqual(len(merged_rows), 1)
        row = merged_rows[0]
        self.assertEqual(row["retrieval_status"], "retrieved_oa")
        self.assertEqual(row["source_provider"], "zotero")
        self.assertEqual(row["fulltext_path"], "fulltext/platform-governance.pdf")
        self.assertEqual(row["license"], "CC-BY-4.0")
        self.assertIn("Candidate access URL identified locally.", row["notes"])
        self.assertIn("Resolved from Zotero attachment.", row["notes"])


if __name__ == "__main__":
    unittest.main()
