from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from bridges.mcp_connectors import MCPConnector


REPO_ROOT = Path(__file__).resolve().parents[1]


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
            resolution = self.connector.resolve_provider("fulltext-retrieval")

        self.assertEqual(resolution.source, "external_slot")
        self.assertEqual(resolution.command, None)

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
        self.assertIn("normalized 1 unique identifiers", evidence.summary.lower())
        self.assertEqual(evidence.data["identifier_count"], 1)
        self.assertEqual(evidence.data["identifiers"][0]["type"], "doi")
        self.assertEqual(evidence.data["identifiers"][0]["normalized"], "10.1234/example.5678")


if __name__ == "__main__":
    unittest.main()
