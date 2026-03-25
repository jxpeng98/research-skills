from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from bridges.providers.metadata_registry import collect_reference_records, merge_reference_records


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


if __name__ == "__main__":
    unittest.main()
