from __future__ import annotations

import csv
import tempfile
import unittest
from pathlib import Path
from typing import Any

from bridges.mcp_connectors import MCPConnector
from bridges.providers.citation_graph import run_citation_graph
from bridges.providers.literature_search import run_scholarly_search


class LiteraturePipelineIntegrationTests(unittest.TestCase):
    def setUp(self) -> None:
        super().setUp()
        self.connector = MCPConnector()

    def test_builtin_literature_stack_forms_a_closed_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            project_root = root / "RESEARCH" / "demo-topic"
            project_root.mkdir(parents=True)
            (project_root / "notes").mkdir()
            (project_root / "notes" / "seed-note.md").write_text(
                "# Platform Governance in Practice\n\n"
                "| Field | Value |\n"
                "|---|---|\n"
                "| DOI | 10.1000/platform-governance |\n"
                "| Year | 2024 |\n"
                "| Venue | Organization Science |\n",
                encoding="utf-8",
            )

            task_packet = {
                "topic": "demo-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "research_question": "How do platform governance routines shape capability building?",
                "keywords": ["platform governance", "capability building"],
                "required_outputs": ["search_results.csv", "notes/seed-note.md"],
            }

            search_output = run_scholarly_search(
                task_packet,
                search_fn=self._search_stub,
                retrieved_at="2026-03-25T00:00:00+00:00",
            )
            self.assertEqual(search_output["status"], "ok")
            initial_results = search_output["data"]["search_results"]
            self.assertEqual(len(initial_results), 1)
            self._write_search_results(project_root / "search_results.csv", initial_results)

            citation_output = run_citation_graph(
                task_packet,
                root,
                search_fn=self._search_stub,
                citations_fn=self._citations_stub,
                references_fn=self._references_stub,
                retrieved_at="2026-03-25T00:05:00+00:00",
            )
            self.assertEqual(citation_output["status"], "ok")
            self.assertGreaterEqual(len(citation_output["data"]["resolved_seeds"]), 1)
            self.assertEqual(len(citation_output["data"]["search_results"]), 1)

            combined_results = initial_results + citation_output["data"]["search_results"]
            self._write_search_results(project_root / "search_results.csv", combined_results)

            metadata_evidence = self.connector.collect("metadata-registry", task_packet, root)
            self.assertEqual(metadata_evidence.status, "ok")
            self.assertEqual(metadata_evidence.data["record_count"], 2)
            self.assertIn(
                "search_results",
                metadata_evidence.data["reference_state"]["preferred_input_mode"],
            )

            fulltext_evidence = self.connector.collect("fulltext-retrieval", task_packet, root)
            self.assertEqual(fulltext_evidence.status, "ok")
            self.assertEqual(len(fulltext_evidence.data["retrieval_manifest"]), 2)
            statuses = {
                row["doi"]: row["retrieval_status"]
                for row in fulltext_evidence.data["retrieval_manifest"]
            }
            self.assertEqual(statuses["10.1000/platform-governance"], "not_retrieved:oa_candidate")
            self.assertEqual(statuses["10.1000/capability-routines"], "not_retrieved:needs_provider")
            self.assertEqual(fulltext_evidence.data["summary_counts"]["total"], 2)
            self.assertEqual(fulltext_evidence.data["summary_counts"]["oa_candidates"], 1)
            self.assertEqual(fulltext_evidence.data["summary_counts"]["unresolved"], 1)

    @staticmethod
    def _write_search_results(path: Path, rows: list[dict[str, Any]]) -> None:
        fieldnames = [
            "record_id",
            "source",
            "query_id",
            "retrieved_at",
            "paper_id",
            "title",
            "authors",
            "year",
            "venue",
            "doi",
            "url",
            "abstract",
            "citation_count",
            "open_access_pdf_url",
        ]
        with path.open("w", encoding="utf-8", newline="") as handle:
            writer = csv.DictWriter(handle, fieldnames=fieldnames)
            writer.writeheader()
            for row in rows:
                writer.writerow({field: row.get(field, "") for field in fieldnames})

    @staticmethod
    def _search_stub(query: str, limit: int) -> dict[str, Any]:
        _ = (query, limit)
        return {
            "data": [
                {
                    "paperId": "seed-paper",
                    "externalIds": {"DOI": "10.1000/platform-governance"},
                    "title": "Platform Governance in Practice",
                    "authors": [{"name": "Alex Smith"}],
                    "year": 2024,
                    "venue": "Organization Science",
                    "abstract": "Routines shape capability development.",
                    "url": "https://example.com/platform-governance",
                    "citationCount": 42,
                    "openAccessPdf": {
                        "url": "https://example.com/platform-governance.pdf",
                    },
                }
            ]
        }

    @staticmethod
    def _citations_stub(seed_id: str, limit: int) -> dict[str, Any]:
        _ = limit
        if seed_id != "seed-paper":
            return {"data": []}
        return {
            "data": [
                {
                    "citingPaper": {
                        "paperId": "citing-paper",
                        "externalIds": {"DOI": "10.1000/capability-routines"},
                        "title": "Capability Routines in Digital Platforms",
                        "authors": [{"name": "Taylor Chen"}],
                        "year": 2025,
                        "venue": "Academy of Management Journal",
                        "abstract": "Capability routines evolve through governance changes.",
                        "url": "https://example.com/capability-routines",
                        "citationCount": 7,
                    }
                }
            ]
        }

    @staticmethod
    def _references_stub(seed_id: str, limit: int) -> dict[str, Any]:
        _ = (seed_id, limit)
        return {"data": []}


if __name__ == "__main__":
    unittest.main()
