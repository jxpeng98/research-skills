from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class MCPProviderDocsTests(unittest.TestCase):
    def test_english_provider_setup_includes_capability_matrix_and_modes(self) -> None:
        content = (REPO_ROOT / "docs" / "advanced" / "mcp-providers-setup.md").read_text(
            encoding="utf-8"
        )

        for token in (
            "## MCP Capability Matrix",
            "## Quick Decision Rules",
            "Full external override",
            "Builtin baseline + external overlay",
            "`RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD`",
            "`RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD`",
            "| `reporting-guidelines` | No builtin MCP, but strong skill-level fallback via `reporting-checker`. |",
            "| `submission-kit` | No builtin MCP, but strong skill-level fallback via `submission-packager`. |",
        ):
            self.assertIn(token, content)

    def test_chinese_provider_setup_includes_capability_matrix_and_modes(self) -> None:
        content = (
            REPO_ROOT / "docs" / "zh" / "advanced" / "mcp-providers-setup.md"
        ).read_text(encoding="utf-8")

        for token in (
            "## MCP 能力矩阵",
            "## 快速决策规则",
            "完整外部替换",
            "builtin baseline + 外部 overlay",
            "`RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD`",
            "`RESEARCH_MCP_FULLTEXT_RETRIEVAL_RESOLVE_CMD`",
            "| `reporting-guidelines` | 没有 builtin MCP，但 `reporting-checker` skill 已经提供强 fallback。 |",
            "| `submission-kit` | 没有 builtin MCP，但 `submission-packager` skill 已经提供强 fallback。 |",
        ):
            self.assertIn(token, content)


if __name__ == "__main__":
    unittest.main()
