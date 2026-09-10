from __future__ import annotations

import unittest
from pathlib import Path

import yaml

from qiongli.source_layout import RepoLayout

from qiongli.skill_docs import generate_skill_reference_docs


REPO_ROOT = Path(__file__).resolve().parents[1]


class SkillDocGenerationTests(unittest.TestCase):
    def test_generated_skill_docs_include_registry_driven_user_facing_fields(self) -> None:
        generated = generate_skill_reference_docs(REPO_ROOT)
        en_doc = generated["docs/reference/skills.md"]

        self.assertIn(
            "User-facing surfaces may read `display_name`, `when_to_use`, `summary_zh`, `display_name_zh`, and `when_to_use_zh` directly from that registry.",
            en_doc,
        )
        self.assertIn("| Skill | Display Name | When to use | Produces |", en_doc)
        self.assertIn("| `question-refiner` | Question Refiner |", en_doc)
        self.assertIn(
            "transform vague topics into structured rqs via pico/peo + finer evaluation",
            en_doc.lower(),
        )
        self.assertIn("| `model-collaborator` | Model Collaborator |", en_doc)
        self.assertIn(
            "literature screening, peer review simulation, rebuttal drafting, qualitative coding, or code/statistics validation",
            en_doc,
        )
        self.assertIn("| `academic-context-maintainer` | Academic Context Maintainer |", en_doc)
        self.assertIn(
            "stage-aware academic state summary that preserves research question scope, locked methodological choices, stable findings, unresolved disputes, and decision rationale",
            en_doc,
        )

    def test_generated_skill_docs_include_localized_registry_metadata(self) -> None:
        generated = generate_skill_reference_docs(REPO_ROOT)
        zh_doc = generated["docs/zh/reference/skills.md"]

        self.assertIn("论文架构师", zh_doc)
        self.assertIn("当你需要搭建论文整体结构、章节推进和核心论证主线时使用。", zh_doc)
        self.assertIn("| `F_writing` | 结构、结果解释、表格、图、摘要 | 8 |", zh_doc)

    def test_generated_skill_docs_include_current_domain_profiles(self) -> None:
        generated = generate_skill_reference_docs(REPO_ROOT)
        en_doc = generated["docs/reference/skills.md"]

        self.assertIn("- `business-management`", en_doc)
        self.assertIn("Auto-generated from `skills/registry.yaml`", en_doc)

    def test_generated_skill_docs_include_j_proofread_stage(self) -> None:
        generated = generate_skill_reference_docs(REPO_ROOT)
        en_doc = generated["docs/reference/skills.md"]
        zh_doc = generated["docs/zh/reference/skills.md"]

        self.assertIn("| `J_proofread` | AI detection, humanization, similarity, final polish | 4 |", en_doc)
        self.assertIn("| `J_proofread` | AI 痕迹检查、人声化改写、相似度、终稿校对 | 4 |", zh_doc)
        self.assertNotIn("`J`-level proofread and polishing entrypoints live at the workflow layer today", en_doc)

    def test_workflow_skill_overview_lists_current_stages_and_skills(self) -> None:
        workflow = RepoLayout(REPO_ROOT).workflow
        content = (workflow / "SKILL.md").read_text(encoding="utf-8")
        version = (workflow / "VERSION").read_text(encoding="utf-8").strip()
        frontmatter = content.split("---", 2)[1]
        metadata = yaml.safe_load(frontmatter)

        self.assertEqual(metadata["name"], "qiongli")
        self.assertIn(f"Qiongli version: {version}", metadata["description"])
        self.assertIn(f"Qiongli version: {version}", content)
        self.assertIn(f"Installed Qiongli workflow version: `{version}`", content)

        # Discovery must cover the actual registry, rather than a duplicated
        # hand-picked list or a stale stage count in the entrypoint.
        self.assertIn("skills-summary.md", content)
        summary = (REPO_ROOT / "content" / "skills-summary.md").read_text(encoding="utf-8")
        registry = yaml.safe_load((REPO_ROOT / "content" / "skills" / "registry.yaml").read_text())
        discovered = [
            line.split("|")[1].strip()
            for line in summary.splitlines()
            if line.startswith("| ") and not line.startswith("| Skill |")
        ]
        self.assertCountEqual([item["id"] for item in registry["skills"]], discovered)
        for item in registry["skills"]:
            self.assertTrue((REPO_ROOT / "content" / item["file"]).is_file(), item["id"])
