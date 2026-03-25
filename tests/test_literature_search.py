from __future__ import annotations

import unittest

from bridges.providers.literature_search import (
    build_query_variants,
    dedupe_search_results,
    run_scholarly_search,
)


class LiteratureSearchBaselineTests(unittest.TestCase):
    def test_build_query_variants_uses_topic_question_and_keywords(self) -> None:
        task_packet = {
            "topic": "qualitative process research in management",
            "research_question": "How do founders use narratives to mobilize stakeholder support?",
            "keywords": ["qualitative research", "process theory", "stakeholder support"],
        }

        variants = build_query_variants(task_packet)

        self.assertGreaterEqual(len(variants), 3)
        self.assertEqual(variants[0]["query"], "qualitative process research in management")
        self.assertIn("How do founders use narratives", variants[1]["query"])
        self.assertTrue(any('"qualitative research"' in item["query"] for item in variants))

    def test_dedupe_search_results_merges_records_by_doi(self) -> None:
        records = [
            {
                "record_id": "s2:1",
                "query_id": "q1",
                "paper_id": "1",
                "title": "A Study of Platforms",
                "year": 2024,
                "doi": "10.1000/example",
                "authors": "A. Author",
                "citation_count": 4,
            },
            {
                "record_id": "s2:2",
                "query_id": "q2",
                "paper_id": "2",
                "title": "A Study of Platforms",
                "year": 2024,
                "doi": "10.1000/example",
                "authors": "",
                "citation_count": 8,
            },
        ]

        unique_records, dedup_log = dedupe_search_results(records)

        self.assertEqual(len(unique_records), 1)
        self.assertEqual(len(dedup_log), 1)
        self.assertEqual(dedup_log[0]["match_basis"], "doi")
        self.assertEqual(unique_records[0]["citation_count"], 8)
        self.assertEqual(unique_records[0]["query_ids"], "q1;q2")

    def test_run_scholarly_search_returns_bundle_aware_output(self) -> None:
        responses = {
            "qualitative governance": {
                "data": [
                    {
                        "paperId": "abc",
                        "title": "Qualitative Governance Research",
                        "authors": [{"name": "Alex Smith"}],
                        "year": 2023,
                        "abstract": "Study of governance in firms.",
                        "url": "https://example.com/paper-1",
                        "citationCount": 12,
                        "venue": "Academy of Management Journal",
                        "externalIds": {"DOI": "10.1000/xyz"},
                        "openAccessPdf": {"url": "https://example.com/paper-1.pdf"},
                    }
                ]
            },
            "governance firms": {
                "data": [
                    {
                        "paperId": "dup",
                        "title": "Qualitative Governance Research",
                        "authors": [{"name": "Alex Smith"}],
                        "year": 2023,
                        "abstract": "",
                        "url": "https://example.com/paper-1b",
                        "citationCount": 13,
                        "venue": "AMJ",
                        "externalIds": {"DOI": "https://doi.org/10.1000/xyz"},
                    }
                ]
            },
        }

        def fake_search(query: str, limit: int) -> dict[str, object]:
            del limit
            return responses.get(query, {"error": f"unexpected query: {query}", "data": []})

        task_packet = {
            "topic": "qualitative governance",
            "keywords": ["governance", "firms"],
            "per_query_limit": 5,
        }

        result = run_scholarly_search(
            task_packet,
            fake_search,
            retrieved_at="2026-03-25T12:00:00+00:00",
        )

        self.assertEqual(result["status"], "ok")
        self.assertEqual(result["data"]["raw_result_count"], 2)
        self.assertEqual(result["data"]["unique_result_count"], 1)
        self.assertEqual(result["data"]["duplicate_count"], 1)
        self.assertEqual(result["data"]["artifact_bundle"]["dedup_log"], "dedup_log.csv")
        self.assertEqual(result["data"]["search_results"][0]["doi"], "10.1000/xyz")
        self.assertEqual(result["data"]["search_results"][0]["query_ids"], "q1;q2")
        self.assertEqual(result["data"]["search_log"][0]["query_id"], "q1")


if __name__ == "__main__":
    unittest.main()
