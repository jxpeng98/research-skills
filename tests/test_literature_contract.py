from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class LiteratureContractTests(unittest.TestCase):
    def test_research_workflow_contract_includes_shared_literature_bundle(self) -> None:
        content = (REPO_ROOT / "standards" / "research-workflow-contract.yaml").read_text(
            encoding="utf-8"
        )

        for token in (
            'literature_evidence_bundle:',
            '"dedup_log.csv"',
            '"retrieval_manifest.csv"',
            'scholarly-search:',
            'citation-graph:',
            'metadata-registry:',
            'fulltext-retrieval:',
        ):
            self.assertIn(token, content)

    def test_capability_map_contains_literature_provider_contract(self) -> None:
        content = (REPO_ROOT / "standards" / "mcp-agent-capability-map.yaml").read_text(
            encoding="utf-8"
        )

        for token in (
            "literature_provider_contract:",
            '"dedup_log.csv"',
            '"retrieval_manifest.csv"',
            "owns_artifacts:",
            "appends_to:",
        ):
            self.assertIn(token, content)

    def test_stage_b_reference_documents_new_bundle_artifacts(self) -> None:
        content = (
            REPO_ROOT / "research-paper-workflow" / "references" / "stage-B-literature.md"
        ).read_text(encoding="utf-8")

        for token in (
            "`dedup_log.csv`",
            "`retrieval_manifest.csv`",
            "candidate_record_id,canonical_record_id,decision,match_basis,resolver,notes",
            "record_id,citekey,doi,retrieval_status,version_label,source_provider",
            "MCP/provider adapters",
            "supplemental evidence",
        ):
            self.assertIn(token, content)

    def test_academic_searcher_uses_provider_layer_language(self) -> None:
        content = (REPO_ROOT / "skills" / "B_literature" / "academic-searcher.md").read_text(
            encoding="utf-8"
        )

        for token in (
            "`scholarly-search` as the primary discovery layer",
            "through the MCP/provider layer",
            "configured scholarly provider overlay",
            "canonical execution path in this repo is the MCP/provider stack",
        ):
            self.assertIn(token, content)

        self.assertNotIn("search_web tool", content)
        self.assertNotIn("read_url_content", content)

    def test_literature_templates_exist(self) -> None:
        self.assertTrue((REPO_ROOT / "templates" / "dedup-log.csv").exists())
        self.assertTrue((REPO_ROOT / "templates" / "retrieval-manifest.csv").exists())


if __name__ == "__main__":
    unittest.main()
