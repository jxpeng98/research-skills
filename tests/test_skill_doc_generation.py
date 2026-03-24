from __future__ import annotations

import unittest
from pathlib import Path

from research_skills.skill_docs import generate_skill_reference_docs


REPO_ROOT = Path(__file__).resolve().parents[1]


class SkillDocGenerationTests(unittest.TestCase):
    def test_generated_skill_docs_include_localized_registry_metadata(self) -> None:
        generated = generate_skill_reference_docs(REPO_ROOT)
        zh_doc = generated["docs/zh/reference/skills.md"]

        self.assertIn("论文架构师", zh_doc)
        self.assertIn("当你需要搭建论文整体结构、章节推进和核心论证主线时使用。", zh_doc)
        self.assertIn("| `F_writing` | 结构、结果解释、表格、图、摘要 | 6 |", zh_doc)

    def test_generated_skill_docs_include_current_domain_profiles(self) -> None:
        generated = generate_skill_reference_docs(REPO_ROOT)
        en_doc = generated["docs/reference/skills.md"]

        self.assertIn("- `business-management`", en_doc)
        self.assertIn("Auto-generated from `skills/registry.yaml`", en_doc)
