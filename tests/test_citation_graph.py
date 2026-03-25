from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from bridges.providers.citation_graph import extract_seed_candidates, run_citation_graph


class CitationGraphBaselineTests(unittest.TestCase):
    def test_extract_seed_candidates_reads_search_results_bibliography_and_notes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            project_root = root / "RESEARCH" / "demo-topic"
            notes_dir = project_root / "notes"
            notes_dir.mkdir(parents=True)
            (project_root / "search_results.csv").write_text(
                "record_id,source,query_id,retrieved_at,title,authors,year,venue,doi,url,abstract,paper_id\n"
                "s2:1,semantic_scholar,q1,2026-03-25T00:00:00+00:00,Existing Study,A Author,2024,AMJ,10.1000/existing,https://example.com,,seed-paper-1\n",
                encoding="utf-8",
            )
            (project_root / "bibliography.bib").write_text(
                "@article{demo,\n"
                "  title = {Bibliography Seed Title},\n"
                "  doi = {10.1000/bib-seed}\n"
                "}\n",
                encoding="utf-8",
            )
            (notes_dir / "seed-note.md").write_text(
                "# Seed Note\npaper_id: seed-note-7\ndoi: 10.1000/note-seed\n",
                encoding="utf-8",
            )

            candidates = extract_seed_candidates({"topic": "demo-topic"}, project_root)

        pairs = {(item["seed_type"], item["seed_value"]) for item in candidates}
        self.assertIn(("paper_id", "seed-paper-1"), pairs)
        self.assertIn(("doi", "10.1000/bib-seed"), pairs)
        self.assertIn(("paper_id", "seed-note-7"), pairs)

    def test_run_citation_graph_uses_local_seed_and_dedupes_existing_and_internal(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            project_root = root / "RESEARCH" / "demo-topic"
            project_root.mkdir(parents=True)
            (project_root / "search_results.csv").write_text(
                "record_id,source,query_id,retrieved_at,title,authors,year,venue,doi,url,abstract,paper_id\n"
                "s2:existing,semantic_scholar,q1,2026-03-25T00:00:00+00:00,Existing Paper,A Author,2023,AMJ,10.1000/existing,https://example.com/existing,,seed-paper-1\n",
                encoding="utf-8",
            )

            def fake_search(query: str, limit: int) -> dict[str, object]:
                del query, limit
                return {"data": []}

            def fake_citations(seed_id: str, limit: int) -> dict[str, object]:
                del seed_id, limit
                return {
                    "data": [
                        {
                            "paperId": "cand-existing",
                            "title": "Existing Paper",
                            "authors": [{"name": "A Author"}],
                            "year": 2023,
                            "venue": "AMJ",
                            "url": "https://example.com/existing-dup",
                            "citationCount": 15,
                            "externalIds": {"DOI": "10.1000/existing"},
                        },
                        {
                            "paperId": "cand-new-1",
                            "title": "Novel Candidate",
                            "authors": [{"name": "B Author"}],
                            "year": 2024,
                            "venue": "Org Science",
                            "url": "https://example.com/new-1",
                            "citationCount": 7,
                            "externalIds": {"DOI": "10.1000/new-candidate"},
                        },
                    ]
                }

            def fake_references(seed_id: str, limit: int) -> dict[str, object]:
                del seed_id, limit
                return {
                    "data": [
                        {
                            "paperId": "cand-new-2",
                            "title": "Novel Candidate",
                            "authors": [{"name": "B Author"}],
                            "year": 2024,
                            "venue": "Organization Studies",
                            "url": "https://example.com/new-2",
                            "citationCount": 6,
                            "externalIds": {"DOI": "https://doi.org/10.1000/new-candidate"},
                        }
                    ]
                }

            result = run_citation_graph(
                {"topic": "demo-topic", "seed_limit": 1, "graph_limit": 5},
                root,
                search_fn=fake_search,
                citations_fn=fake_citations,
                references_fn=fake_references,
                retrieved_at="2026-03-25T12:00:00+00:00",
            )

        self.assertEqual(result["status"], "ok")
        self.assertEqual(len(result["data"]["resolved_seeds"]), 1)
        self.assertEqual(len(result["data"]["search_results"]), 1)
        self.assertEqual(result["data"]["search_results"][0]["doi"], "10.1000/new-candidate")
        self.assertEqual(len(result["data"]["dedup_log"]), 2)
        decisions = {entry["decision"] for entry in result["data"]["dedup_log"]}
        self.assertIn("drop_duplicate", decisions)
        self.assertIn("merge_duplicate", decisions)
        snowball_decisions = {entry["decision"] for entry in result["data"]["snowball_log"]}
        self.assertIn("drop_duplicate", snowball_decisions)
        self.assertIn("new_candidate", snowball_decisions)


if __name__ == "__main__":
    unittest.main()
