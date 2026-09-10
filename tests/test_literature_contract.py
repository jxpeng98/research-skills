from __future__ import annotations

import re
import tempfile
import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout
from qiongli.subject_materializer import MaterializeOptions, materialize_subject_package


REPO_ROOT = Path(__file__).resolve().parents[1]
LAYOUT = RepoLayout(REPO_ROOT)
COLLECT_EVIDENCE_BOUNDARY_BULLET = (
    "- Do not use `qiongli_collect_evidence` to judge built-in literature provider "
    "configuration. That tool is a filesystem/builtin/external-command evidence adapter; "
    "direct provider names such as `openalex` require a separate `RESEARCH_MCP_OPENALEX_CMD`. "
    "Use `qiongli_literature_status`, `qiongli_config_status`, `qiongli_test_provider`, "
    "and `qiongli_literature_search` to judge OpenAlex, Semantic Scholar, Crossref, "
    "PubMed, and arXiv provider availability."
)
DESKTOP_MCPB_PROVIDER_BULLET = (
    "- Desktop/Web users need the Qiongli Literature Provider `.mcpb` "
    "(`qiongli-literature-provider.mcpb`) or another configured provider MCP before "
    "claiming `provider_connected` literature search. The MCPB is the separate local "
    "Claude Desktop provider for OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv "
    "configuration/search. Its primary package uses the Rust Lite MCP executable, not a "
    "user-installed Node or Python runtime. arXiv is enabled without credentials. "
    "Platform-native search alone is `native_only`, not `provider_connected`; if no "
    "provider MCP/MCPB and no platform-native search is available, record the run as "
    "`strategy_only`."
)


class LiteratureContractTests(unittest.TestCase):
    def test_research_workflow_contract_includes_shared_literature_bundle(self) -> None:
        content = (LAYOUT.standards / "research-workflow-contract.yaml").read_text(
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
        content = (LAYOUT.standards / "mcp-agent-capability-map.yaml").read_text(
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
            LAYOUT.workflow / "references" / "stage-B-literature.md"
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

    def test_stage_b_distinguishes_discovery_coverage_from_fulltext_access(self) -> None:
        content = (
            LAYOUT.workflow / "references" / "stage-B-literature.md"
        ).read_text(encoding="utf-8")

        self.assertIn("discovery coverage", content)
        self.assertIn("full-text access coverage", content)
        self.assertIn("retrieval_manifest.csv", content)
        self.assertIn("native_fulltext_queries", content)
        self.assertIn("Zotero attachment", content)
        self.assertIn("metadata_only", content)

    def test_academic_searcher_uses_provider_layer_language(self) -> None:
        content = (LAYOUT.skills / "B_literature" / "academic-searcher.md").read_text(
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
        self.assertTrue((LAYOUT.templates / "dedup-log.csv").exists())
        self.assertTrue((LAYOUT.templates / "retrieval-manifest.csv").exists())

    def test_search_diagnostics_template_locks_coverage_semantics_fields(self) -> None:
        content = (LAYOUT.templates / "search-diagnostics.md").read_text(encoding="utf-8")

        for token in (
            "## Coverage Semantics",
            "discovery_coverage_basis",
            "native_fulltext_candidate_coverage",
            "zotero_attachment_coverage",
            "known_item_recall",
            "duplicate_saturation",
            "full_text_access_coverage",
            "evidence_limit_distribution",
            "cannot_claim_absolute_completeness: true",
        ):
            self.assertIn(token, content)

    def test_literature_workflows_route_through_provider_adapters(self) -> None:
        workflow_paths = (
            LAYOUT.workflow / "workflows" / "lit-review.md",
            LAYOUT.workflow / "workflows" / "paper-read.md",
        )

        required_tokens = (
            "MCP/provider",
            "`scholarly-search`",
            "`metadata-registry`",
            "`fulltext-retrieval`",
            "provider_connected",
            "strategy_only",
        )
        forbidden_tokens = (
            "Execute Semantic Scholar API search",
            "Execute arXiv API search",
            "Supplement with Google Scholar web search",
            "Attempt OA retrieval via Unpaywall/CORE/Semantic Scholar",
            "DOI lookup via Semantic Scholar API",
            "arXiv API (for arXiv links)",
            "Title search via Semantic Scholar",
        )

        for path in workflow_paths:
            with self.subTest(workflow=str(path.relative_to(REPO_ROOT))):
                content = path.read_text(encoding="utf-8")
                for token in required_tokens:
                    self.assertIn(token, content)
                for token in forbidden_tokens:
                    self.assertNotIn(token, content)

    def test_literature_workflows_define_search_plan_execution_modes_and_provenance(self) -> None:
        workflow_paths = (
            LAYOUT.workflow / "references" / "literature-provider-routing.md",
            LAYOUT.workflow / "workflows" / "lit-review.md",
            LAYOUT.workflow / "workflows" / "paper-read.md",
        )

        required_tokens = (
            "qiongli_search_plan",
            "hybrid_search",
            "provider_connected",
            "native_only",
            "strategy_only",
            "provider_capability_mode",
            "MCP servers must not call Codex or Claude native search directly",
            "native_search_queries",
            "mcp:openalex",
            "mcp:semantic_scholar",
            "mcp:crossref",
            "mcp:pubmed",
            "mcp:arxiv",
            "native:codex_web_search",
            "native:claude_web_search",
            "user_corpus",
        )

        for path in workflow_paths:
            with self.subTest(workflow=str(path.relative_to(REPO_ROOT))):
                content = path.read_text(encoding="utf-8")
                for token in required_tokens:
                    self.assertIn(token, content)

    def test_academic_searcher_documents_hybrid_search_layer_ownership(self) -> None:
        content = (LAYOUT.skills / "B_literature" / "academic-searcher.md").read_text(
            encoding="utf-8"
        )

        for token in (
            "Hybrid search coordination belongs to the workflow/router layer",
            "Provider layer owns provider calls",
            "active agent owns platform-native search",
            "skill owns logging, normalization, dedupe, and diagnostics",
            "MCP servers must not call Codex or Claude native search directly",
            "qiongli_search_plan",
            "hybrid_search",
            "native_only",
            "provider_connected",
            "strategy_only",
            "native:codex_web_search",
            "native:claude_web_search",
            "user_corpus",
        ):
            self.assertIn(token, content)

    def test_workflow_search_modes_do_not_downgrade_native_search_to_strategy_only(self) -> None:
        content = (LAYOUT.workflow / "references" / "literature-provider-routing.md").read_text(encoding="utf-8")

        self.assertIn(
            "`strategy_only` only when neither provider MCP nor platform-native search is available",
            content,
        )
        self.assertIn(
            "If provider preflight is unavailable or non-provider-connected but platform-native search is usable, write `qiongli_search_plan` with `search_execution_mode: native_only`.",
            content,
        )
        self.assertNotIn(
            "Treat `strategy_only` as a constrained mode: use platform search",
            content,
        )
        self.assertNotIn(
            "or platform-native search capability before claiming `provider_connected`",
            content,
        )
        self.assertNotRegex(
            content,
            re.compile(
                r"provider_connected[^.\n]*platform-native search capability|"
                r"platform-native search capability[^.\n]*provider_connected"
            ),
        )

    def test_platform_routing_keeps_provider_connected_separate_from_native_search(self) -> None:
        content = (LAYOUT.workflow / "references" / "platform-routing.md").read_text(
            encoding="utf-8"
        )

        self.assertIn("Platform-native search alone is `native_only`", content)
        self.assertNotIn("provider MCPB or platform-native search capability", content)
        self.assertNotRegex(
            content,
            re.compile(
                r"provider_connected[^.\n]*platform-native search capability|"
                r"platform-native search capability[^.\n]*provider_connected"
            ),
        )

    def test_workflow_strategy_only_is_not_bound_to_evidence_limits(self) -> None:
        workflow_paths = (
            LAYOUT.workflow / "workflows" / "lit-review.md",
            LAYOUT.workflow / "workflows" / "paper-read.md",
        )
        evidence_limit_terms = (
            r"abstract[-_ ]only",
            r"metadata[-_ ]only",
            r"manually supplied",
        )
        forbidden_strategy_only_binding = re.compile(
            rf"`?strategy_only`?[^.\n]*(?:{'|'.join(evidence_limit_terms)})|"
            rf"(?:{'|'.join(evidence_limit_terms)})[^.\n]*`?strategy_only`?",
            re.IGNORECASE,
        )

        for path in workflow_paths:
            with self.subTest(workflow=str(path.relative_to(REPO_ROOT))):
                content = path.read_text(encoding="utf-8")
                self.assertNotRegex(content, forbidden_strategy_only_binding)

    def test_academic_searcher_does_not_own_search_plan_creation(self) -> None:
        content = (LAYOUT.skills / "B_literature" / "academic-searcher.md").read_text(
            encoding="utf-8"
        )

        self.assertNotRegex(content, re.compile(r"This\s+skill owns the search plan"))
        self.assertIn(
            "workflow/router owns `qiongli_search_plan` creation and `search_execution_mode` selection",
            content,
        )

    def test_workflow_guidance_rejects_collect_evidence_as_provider_status_source(self) -> None:
        content = (LAYOUT.workflow / "references" / "literature-provider-routing.md").read_text(encoding="utf-8")

        self.assertIn(COLLECT_EVIDENCE_BOUNDARY_BULLET.replace(
            "Use `qiongli_literature_status`, `qiongli_config_status`, `qiongli_test_provider`, "
            "and `qiongli_literature_search`", "Use visible `qiongli_literature_status`, "
            "`qiongli_config_status`, and `qiongli_literature_search` tools"), content)
        self.assertIn("`qiongli_literature_status`", content)
        self.assertIn("`qiongli_literature_search`", content)

    def test_workflow_guidance_uses_exact_desktop_mcpb_provider_bullet(self) -> None:
        content = (LAYOUT.workflow / "references" / "literature-provider-routing.md").read_text(encoding="utf-8")

        self.assertIn(DESKTOP_MCPB_PROVIDER_BULLET, content)

    def test_materialized_workflow_guidance_matches_provider_boundary_text(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            out = Path(tmp_dir) / "core-desktop"
            materialize_subject_package(
                MaterializeOptions(source=REPO_ROOT, out=out, subject="core", flavor="desktop")
            )
            text = (out / "SKILL.md").read_text(encoding="utf-8")

        self.assertIn(COLLECT_EVIDENCE_BOUNDARY_BULLET, text)
        self.assertIn(DESKTOP_MCPB_PROVIDER_BULLET, text)
        self.assertNotIn("use platform search or user-supplied corpus", text)
        self.assertIn("do not claim review-grade external provider or native-search coverage", text)


if __name__ == "__main__":
    unittest.main()
