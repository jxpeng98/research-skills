from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from bridges.providers.metadata_registry import (
    collect_reference_records,
    merge_external_enrichment_payload,
    merge_reference_records,
)


FIXTURES_DIR = Path(__file__).resolve().parent / "fixtures"


class MetadataRegistryTests(unittest.TestCase):
    def test_merge_reference_records_works_without_bibtex_input(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / "references.ris").write_text(
                "TY  - JOUR\n"
                "ID  - ris-seed\n"
                "AU  - Smith, Alex\n"
                "TI  - Platform Governance in Practice\n"
                "JO  - Organization Science\n"
                "PY  - 2024\n"
                "DO  - 10.1000/platform-governance\n"
                "ER  - \n",
                encoding="utf-8",
            )
            (root / "search_results.csv").write_text(
                "record_id,source,query_id,retrieved_at,title,authors,year,venue,doi,url,abstract\n"
                "s2:1,semantic_scholar,q1,2026-03-25T00:00:00+00:00,Platform Governance in Practice,\"Smith, Alex\",2024,Organization Science,https://doi.org/10.1000/platform-governance,https://example.com,\n",
                encoding="utf-8",
            )

            records = collect_reference_records(root, ["references.ris", "search_results.csv"])
            merged_records, dedup_log = merge_reference_records(records)

        self.assertEqual(len(records), 2)
        self.assertEqual(len(merged_records), 1)
        self.assertEqual(merged_records[0]["citekey"], "ris-seed")
        self.assertEqual(merged_records[0]["doi"], "10.1000/platform-governance")
        self.assertEqual(len(dedup_log), 1)

    def test_external_enrichment_prefers_openalex_for_core_bibliographic_fields(self) -> None:
        local_records = [
            {
                "record_id": "s2:1",
                "source_format": "search_results.csv",
                "source_provider": "search_results",
                "source_paths": ["search_results.csv"],
                "title": "Platform Governance Practice",
                "authors": ["Smith, Alex"],
                "year": 2024,
                "venue": "Org Science",
                "doi": "10.1000/platform-governance",
                "url": "https://example.com/platform-governance",
                "abstract": "",
                "citekey": "",
                "field_provenance": {
                    "title": {"provider": "search_results", "source_format": "search_results.csv", "source_path": "search_results.csv"},
                    "authors": {"provider": "search_results", "source_format": "search_results.csv", "source_path": "search_results.csv"},
                    "year": {"provider": "search_results", "source_format": "search_results.csv", "source_path": "search_results.csv"},
                    "venue": {"provider": "search_results", "source_format": "search_results.csv", "source_path": "search_results.csv"},
                    "doi": {"provider": "search_results", "source_format": "search_results.csv", "source_path": "search_results.csv"},
                    "url": {"provider": "search_results", "source_format": "search_results.csv", "source_path": "search_results.csv"},
                },
            }
        ]
        crossref_payload = json.loads((FIXTURES_DIR / "metadata_enrichment_crossref.json").read_text(encoding="utf-8"))
        openalex_payload = json.loads((FIXTURES_DIR / "metadata_enrichment_openalex.json").read_text(encoding="utf-8"))

        merged_records, _ = merge_external_enrichment_payload(local_records, crossref_payload)
        merged_records, enrichment_info = merge_external_enrichment_payload(merged_records, openalex_payload)

        self.assertEqual(enrichment_info["status"], "ok")
        self.assertEqual(len(merged_records), 1)
        record = merged_records[0]
        self.assertEqual(record["title"], "Platform Governance in Practice")
        self.assertEqual(record["venue"], "Organization Science")
        self.assertEqual(record["authors"], ["Smith, Alex", "Lee, Jordan"])
        self.assertEqual(record["publisher"], "INFORMS")
        self.assertEqual(record["openalex_id"], "W1234567890")
        self.assertEqual(record["volume"], "35")
        self.assertEqual(record["issue"], "2")
        self.assertEqual(record["pages"], "123-145")
        self.assertEqual(record["oa_url"], "https://oa.example.org/platform-governance.pdf")
        self.assertEqual(record["field_provenance"]["title"]["provider"], "openalex")
        self.assertEqual(record["field_provenance"]["venue"]["provider"], "openalex")
        self.assertEqual(record["field_provenance"]["authors"]["provider"], "openalex")
        self.assertEqual(record["field_provenance"]["volume"]["provider"], "crossref")
        self.assertEqual(record["field_provenance"]["pages"]["provider"], "crossref")
        self.assertEqual(record["field_provenance"]["doi"]["provider"], "search_results")

    def test_external_enrichment_exposes_field_policy_and_merge_trace(self) -> None:
        local_records = [
            {
                "record_id": "s2:1",
                "source_format": "search_results.csv",
                "source_provider": "search_results",
                "source_paths": ["search_results.csv"],
                "title": "Platform Governance Practice",
                "authors": ["Smith, Alex"],
                "year": 2024,
                "venue": "Org Science",
                "doi": "10.1000/platform-governance",
                "url": "https://example.com/platform-governance",
                "abstract": "",
                "citekey": "",
                "field_provenance": {
                    "title": {"provider": "search_results", "source_format": "search_results.csv", "source_path": "search_results.csv"},
                    "authors": {"provider": "search_results", "source_format": "search_results.csv", "source_path": "search_results.csv"},
                    "year": {"provider": "search_results", "source_format": "search_results.csv", "source_path": "search_results.csv"},
                    "venue": {"provider": "search_results", "source_format": "search_results.csv", "source_path": "search_results.csv"},
                    "doi": {"provider": "search_results", "source_format": "search_results.csv", "source_path": "search_results.csv"},
                    "url": {"provider": "search_results", "source_format": "search_results.csv", "source_path": "search_results.csv"},
                },
            }
        ]
        openalex_payload = json.loads((FIXTURES_DIR / "metadata_enrichment_openalex.json").read_text(encoding="utf-8"))
        crossref_payload = json.loads((FIXTURES_DIR / "metadata_enrichment_crossref.json").read_text(encoding="utf-8"))

        merged_records, _ = merge_external_enrichment_payload(local_records, openalex_payload)
        merged_records, enrichment_info = merge_external_enrichment_payload(merged_records, crossref_payload)

        self.assertEqual(enrichment_info["field_policy_version"], "openalex_core_crossref_structural_v1")
        self.assertTrue(enrichment_info["merge_trace"])
        record = merged_records[0]
        self.assertEqual(record["title"], "Platform Governance in Practice")
        self.assertEqual(record["venue"], "Organization Science")
        self.assertEqual(record["url"], "https://doi.org/10.1000/platform-governance")
        self.assertEqual(record["publisher"], "INFORMS")
        self.assertEqual(record["field_provenance"]["title"]["provider"], "openalex")
        self.assertEqual(record["field_provenance"]["url"]["provider"], "crossref")
        self.assertEqual(record["field_provenance"]["publisher"]["provider"], "crossref")

        self.assertTrue(
            any(
                item["field"] == "title"
                and item["decision"] == "keep_existing"
                and item["existing_provider"] == "openalex"
                and item["candidate_provider"] == "crossref"
                for item in enrichment_info["merge_trace"]
            )
        )
        self.assertTrue(
            any(
                item["field"] == "url"
                and item["decision"] == "prefer_candidate"
                and item["existing_provider"] == "openalex"
                and item["candidate_provider"] == "crossref"
                for item in enrichment_info["merge_trace"]
            )
        )

    def test_external_enrichment_keeps_existing_doi_and_exposes_fixture_provenance(self) -> None:
        local_records = [
            {
                "record_id": "s2:1",
                "source_format": "search_results.csv",
                "source_provider": "search_results",
                "source_paths": ["search_results.csv"],
                "title": "Platform Governance in Practice",
                "authors": ["Smith, Alex"],
                "year": 2024,
                "venue": "Organization Science",
                "doi": "10.1000/platform-governance",
                "url": "https://example.com/platform-governance",
                "abstract": "",
                "citekey": "smith2024platform",
                "field_provenance": {
                    "doi": {"provider": "search_results", "source_format": "search_results.csv", "source_path": "search_results.csv"},
                    "citekey": {"provider": "search_results", "source_format": "search_results.csv", "source_path": "search_results.csv"},
                },
            }
        ]
        openalex_payload = json.loads((FIXTURES_DIR / "metadata_enrichment_openalex.json").read_text(encoding="utf-8"))

        merged_records, enrichment_info = merge_external_enrichment_payload(local_records, openalex_payload)

        self.assertEqual(enrichment_info["provenance"], ["https://api.openalex.org/works/W1234567890"])
        record = merged_records[0]
        self.assertEqual(record["doi"], "10.1000/platform-governance")
        self.assertEqual(record["citekey"], "smith2024platform")
        self.assertEqual(record["field_provenance"]["doi"]["provider"], "search_results")
        self.assertEqual(record["field_provenance"]["citekey"]["provider"], "search_results")


if __name__ == "__main__":
    unittest.main()
