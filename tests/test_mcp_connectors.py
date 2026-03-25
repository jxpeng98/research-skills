from __future__ import annotations

import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from bridges.mcp_connectors import MCPConnector


REPO_ROOT = Path(__file__).resolve().parents[1]
FIXTURES_DIR = REPO_ROOT / "tests" / "fixtures"


class MCPConnectorTests(unittest.TestCase):
    def setUp(self) -> None:
        super().setUp()
        self.connector = MCPConnector()

    def test_resolve_provider_prefers_env_override(self) -> None:
        script_path = REPO_ROOT / "scripts" / "mcp_scholarly_search.py"
        with mock.patch.dict(
            os.environ,
            {"RESEARCH_MCP_SCHOLARLY_SEARCH_CMD": f"python3 {script_path}"},
            clear=False,
        ):
            resolution = self.connector.resolve_provider("scholarly-search")

        self.assertEqual(resolution.source, "env_override")
        self.assertIn("python3", resolution.command or "")
        self.assertEqual(resolution.native_script, str(script_path))

    def test_resolve_provider_detects_builtin(self) -> None:
        with mock.patch.dict(os.environ, {}, clear=False):
            resolution = self.connector.resolve_provider("metadata-registry")

        self.assertEqual(resolution.source, "builtin")
        self.assertTrue((resolution.native_script or "").endswith("mcp_metadata_registry.py"))

    def test_resolve_provider_detects_external_slot(self) -> None:
        with mock.patch.dict(os.environ, {}, clear=False):
            resolution = self.connector.resolve_provider("screening-tracker")

        self.assertEqual(resolution.source, "external_slot")
        self.assertEqual(resolution.command, None)

    def test_resolve_provider_detects_builtin_fulltext_stub(self) -> None:
        with mock.patch.dict(os.environ, {}, clear=False):
            resolution = self.connector.resolve_provider("fulltext-retrieval")

        self.assertEqual(resolution.source, "builtin")
        self.assertTrue((resolution.native_script or "").endswith("mcp_fulltext_retrieval.py"))

    def test_builtin_metadata_registry_normalizes_local_doi(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            project_root = root / "RESEARCH" / "demo-topic"
            project_root.mkdir(parents=True)
            bib = project_root / "bibliography.bib"
            bib.write_text(
                "@article{demo,\n  doi = {https://doi.org/10.1234/Example.5678}\n}\n",
                encoding="utf-8",
            )
            packet = {
                "topic": "demo-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "required_outputs": ["bibliography.bib"],
            }

            evidence = self.connector.collect("metadata-registry", packet, root)

        self.assertEqual(evidence.status, "ok")
        self.assertIn("normalized 1 identifiers", evidence.summary.lower())
        self.assertEqual(evidence.data["identifier_count"], 1)
        self.assertEqual(evidence.data["record_count"], 1)
        self.assertEqual(evidence.data["identifiers"][0]["type"], "doi")
        self.assertEqual(evidence.data["identifiers"][0]["normalized"], "10.1234/example.5678")

    def test_builtin_metadata_registry_supports_json_and_note_sources_without_bib(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            project_root = root / "RESEARCH" / "demo-topic"
            notes_dir = project_root / "notes"
            notes_dir.mkdir(parents=True)
            (project_root / "references.json").write_text(
                json.dumps(
                    [
                        {
                            "id": "existing-json-id",
                            "title": "Platform Governance in Practice",
                            "author": [{"family": "Smith", "given": "Alex"}],
                            "issued": {"date-parts": [[2024]]},
                            "container-title": "Organization Science",
                            "DOI": "10.1000/platform-governance",
                            "URL": "https://example.com/platform-governance",
                        }
                    ]
                ),
                encoding="utf-8",
            )
            (notes_dir / "governance-note.md").write_text(
                "# Platform Governance in Practice\n\n"
                "## Metadata\n\n"
                "| Field | Value |\n"
                "|-------|-------|\n"
                "| **Authors** | Smith, Alex |\n"
                "| **Year** | 2024 |\n"
                "| **Venue** | Organization Science |\n"
                "| **DOI** | 10.1000/platform-governance |\n",
                encoding="utf-8",
            )
            packet = {
                "topic": "demo-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "required_outputs": ["references.json", "notes/governance-note.md"],
            }

            evidence = self.connector.collect("metadata-registry", packet, root)

        self.assertEqual(evidence.status, "ok")
        self.assertEqual(evidence.data["record_count"], 1)
        self.assertTrue(evidence.data["reference_state"]["supports_non_bib_workflows"])
        self.assertEqual(evidence.data["reference_state"]["preferred_input_mode"], "references.json")
        self.assertEqual(evidence.data["records"][0]["citekey"], "smith2024platform")
        self.assertIn("csl_json", evidence.data["reference_state"]["source_formats"])

    def test_builtin_metadata_registry_applies_external_enrichment_overlay(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            project_root = root / "RESEARCH" / "demo-topic"
            project_root.mkdir(parents=True)
            (project_root / "search_results.csv").write_text(
                "record_id,source,query_id,retrieved_at,title,authors,year,venue,doi,url,abstract\n"
                "s2:1,semantic_scholar,q1,2026-03-25T00:00:00+00:00,Platform Governance in Practice,\"Smith, Alex\",2024,,10.1000/platform-governance,https://example.com,\n",
                encoding="utf-8",
            )
            enrich_script = root / "metadata_enrich_stub.py"
            enrich_script.write_text(
                "import json, sys\n"
                "payload = json.loads(sys.stdin.read())\n"
                "records = payload.get('records', [])\n"
                "for record in records:\n"
                "    if record.get('doi') == '10.1000/platform-governance':\n"
                "        record['venue'] = 'Administrative Science Quarterly'\n"
                "        record['openalex_id'] = 'W1234567890'\n"
                "print(json.dumps({'status': 'ok', 'summary': 'overlay enriched 1 record', 'data': {'records': records}}))\n",
                encoding="utf-8",
            )
            packet = {
                "topic": "demo-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "required_outputs": ["search_results.csv"],
            }

            with mock.patch.dict(
                os.environ,
                {
                    "RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD": f"python3 {enrich_script}",
                },
                clear=False,
            ):
                evidence = self.connector.collect("metadata-registry", packet, root)

        self.assertEqual(evidence.status, "ok")
        self.assertEqual(
            evidence.data["records"][0]["venue"],
            "Administrative Science Quarterly",
        )
        self.assertEqual(evidence.data["records"][0]["openalex_id"], "W1234567890")
        self.assertTrue(evidence.data["reference_state"]["external_enrichment"]["configured"])
        self.assertEqual(
            evidence.data["reference_state"]["external_enrichment"]["status"],
            "ok",
        )

    def test_builtin_metadata_registry_applies_recorded_openalex_fixture(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            project_root = root / "RESEARCH" / "demo-topic"
            project_root.mkdir(parents=True)
            (project_root / "search_results.csv").write_text(
                "record_id,source,query_id,retrieved_at,title,authors,year,venue,doi,url,abstract\n"
                "s2:1,semantic_scholar,q1,2026-03-25T00:00:00+00:00,Platform Governance Practice,\"Smith, Alex\",2024,Org Science,10.1000/platform-governance,https://example.com/platform-governance,\n",
                encoding="utf-8",
            )
            fixture_path = FIXTURES_DIR / "metadata_enrichment_openalex.json"
            enrich_script = root / "metadata_enrich_fixture.py"
            enrich_script.write_text(
                "from pathlib import Path\n"
                "import sys\n"
                "sys.stdout.write(Path(sys.argv[1]).read_text(encoding='utf-8'))\n",
                encoding="utf-8",
            )
            packet = {
                "topic": "demo-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "required_outputs": ["search_results.csv"],
            }

            with mock.patch.dict(
                os.environ,
                {
                    "RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD": f"python3 {enrich_script} {fixture_path}",
                },
                clear=False,
            ):
                evidence = self.connector.collect("metadata-registry", packet, root)

        self.assertEqual(evidence.status, "ok")
        record = evidence.data["records"][0]
        self.assertEqual(record["title"], "Platform Governance in Practice")
        self.assertEqual(record["venue"], "Organization Science")
        self.assertEqual(record["field_provenance"]["title"]["provider"], "openalex")
        self.assertEqual(record["field_provenance"]["venue"]["provider"], "openalex")
        self.assertEqual(record["field_provenance"]["doi"]["provider"], "search_results")

    def test_builtin_fulltext_retrieval_prepares_manifest_without_bib(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            project_root = root / "RESEARCH" / "demo-topic"
            project_root.mkdir(parents=True)
            (project_root / "search_results.csv").write_text(
                "record_id,source,query_id,retrieved_at,paper_id,title,authors,year,venue,doi,url,abstract,open_access_pdf_url\n"
                "s2:1,semantic_scholar,q1,2026-03-25T00:00:00+00:00,S2-1,Platform Governance in Practice,\"Smith, Alex\",2024,Organization Science,10.1000/platform-governance,https://example.com/landing,Abstract text,https://example.com/platform-governance.pdf\n",
                encoding="utf-8",
            )
            packet = {
                "topic": "demo-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "required_outputs": ["search_results.csv"],
            }

            evidence = self.connector.collect("fulltext-retrieval", packet, root)

        self.assertEqual(evidence.status, "ok")
        self.assertEqual(evidence.data["provider_mode"], "builtin_fulltext_manifest_stub")
        self.assertEqual(evidence.data["record_count"], 1)
        self.assertEqual(
            evidence.data["retrieval_manifest"][0]["retrieval_status"],
            "not_retrieved:oa_candidate",
        )
        self.assertEqual(
            evidence.data["retrieval_manifest"][0]["source_provider"],
            "semantic_scholar_oa_candidate",
        )
        self.assertEqual(
            evidence.data["screening_full_text_rows"][0]["fulltext_status"],
            "not_retrieved:oa_candidate",
        )
        self.assertEqual(evidence.data["summary_counts"]["oa_candidates"], 1)

    def test_builtin_fulltext_retrieval_applies_external_resolution_overlay(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            project_root = root / "RESEARCH" / "demo-topic"
            (project_root / "fulltext").mkdir(parents=True)
            (project_root / "fulltext" / "platform-governance.pdf").write_bytes(b"%PDF-1.4\n")
            (project_root / "search_results.csv").write_text(
                "record_id,source,query_id,retrieved_at,paper_id,title,authors,year,venue,doi,url,abstract,open_access_pdf_url\n"
                "s2:1,semantic_scholar,q1,2026-03-25T00:00:00+00:00,S2-1,Platform Governance in Practice,\"Smith, Alex\",2024,Organization Science,10.1000/platform-governance,https://example.com/landing,Abstract text,https://example.com/platform-governance.pdf\n",
                encoding="utf-8",
            )
            fixture_path = FIXTURES_DIR / "fulltext_resolution_zotero.json"
            resolve_script = root / "fulltext_resolve_fixture.py"
            resolve_script.write_text(
                "from pathlib import Path\n"
                "import sys\n"
                "sys.stdout.write(Path(sys.argv[1]).read_text(encoding='utf-8'))\n",
                encoding="utf-8",
            )
            packet = {
                "topic": "demo-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "required_outputs": ["search_results.csv"],
            }

            with mock.patch.dict(
                os.environ,
                {
                    "RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD": f"python3 {resolve_script} {fixture_path}",
                },
                clear=False,
            ):
                evidence = self.connector.collect("fulltext-retrieval", packet, root)

        self.assertEqual(evidence.status, "ok")
        self.assertEqual(evidence.data["provider_mode"], "builtin_fulltext_manifest_overlay")
        self.assertTrue(evidence.data["external_resolution"]["configured"])
        row = evidence.data["retrieval_manifest"][0]
        self.assertEqual(row["retrieval_status"], "retrieved_oa")
        self.assertEqual(row["source_provider"], "zotero")
        self.assertEqual(row["fulltext_path"], "fulltext/platform-governance.pdf")
        self.assertEqual(row["license"], "CC-BY-4.0")


if __name__ == "__main__":
    unittest.main()
