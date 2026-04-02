from __future__ import annotations

import tempfile
import textwrap
import unittest
from pathlib import Path

from scripts.validate_research_standard import ValidationReport, validate_skill_structure


class SkillStructureLintTests(unittest.TestCase):
    def _skill_doc(self, body: str) -> str:
        return textwrap.dedent(body).strip() + "\n"

    def _write_registry(self, root: Path, items: list[dict[str, object]]) -> None:
        registry_lines = ["skills:"]
        for item in items:
            registry_lines.extend(
                [
                    f"  - id: {item['id']}",
                    f"    stage: {item['stage']}",
                    f"    file: {item['file']}",
                    f"    canonical: {str(item.get('canonical', True)).lower()}",
                    f"    deprecated: {str(item.get('deprecated', False)).lower()}",
                    f"    alias_of: \"{item.get('alias_of', '')}\"",
                    f"    summary: \"{item.get('summary', '')}\"",
                    f"    display_name: \"{item.get('display_name', item['id'])}\"",
                    f"    when_to_use: \"{item.get('when_to_use', 'Use when needed.')}\"",
                    f"    summary_zh: \"{item.get('summary_zh', '摘要')}\"",
                    f"    display_name_zh: \"{item.get('display_name_zh', '显示名')}\"",
                    f"    when_to_use_zh: \"{item.get('when_to_use_zh', '需要时使用。')}\"",
                    "    inputs: []",
                    "    outputs: []",
                ]
            )
        (root / "skills").mkdir(parents=True, exist_ok=True)
        (root / "skills" / "registry.yaml").write_text("\n".join(registry_lines) + "\n", encoding="utf-8")

    def test_canonical_skill_lint_warns_for_size_section_sprawl_and_summary_duplication(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_registry(
                root,
                [
                    {
                        "id": "demo-skill",
                        "stage": "Z_cross_cutting",
                        "file": "skills/Z_cross_cutting/demo-skill.md",
                        "summary": "Canonical summary sentence.",
                    }
                ],
            )
            skill_path = root / "skills" / "Z_cross_cutting" / "demo-skill.md"
            skill_path.parent.mkdir(parents=True, exist_ok=True)
            sections = "\n".join(
                f"## Section {index}\n\nCanonical summary sentence.\n" for index in range(1, 35)
            )
            filler = "\n".join(f"Line {index}" for index in range(600))
            skill_path.write_text(
                self._skill_doc(
                    f"""\
                    ---
                    id: demo-skill
                    stage: Z_cross_cutting
                    description: "Canonical summary sentence."
                    ---

                    # Demo Skill

                    ## Purpose

                    Canonical summary sentence.

                    ## Process

                    {filler}

                    {sections}

                    ## When to Use

                    Use when needed.

                    ## Quality Bar

                    - [ ] Clear

                    ## Common Pitfalls

                    - Avoid drift
                    """
                ),
                encoding="utf-8",
            )

            report = ValidationReport()
            validate_skill_structure(root, report)

        warning_blob = "\n".join(report.warnings)
        self.assertIn("budget: 520", warning_blob)
        self.assertIn("budget: 32", warning_blob)
        self.assertIn("repeats the registry summary", warning_blob)

    def test_alias_skill_lint_rejects_fat_alias_without_canonical_pointer(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_registry(
                root,
                [
                    {
                        "id": "alias-skill",
                        "stage": "Z_cross_cutting",
                        "file": "skills/Z_cross_cutting/alias-skill.md",
                        "canonical": False,
                        "deprecated": True,
                        "alias_of": "canonical-skill",
                        "summary": "Alias summary.",
                    }
                ],
            )
            skill_path = root / "skills" / "Z_cross_cutting" / "alias-skill.md"
            skill_path.parent.mkdir(parents=True, exist_ok=True)
            body = "\n".join(f"## Extra {index}\n\nDetails.\n" for index in range(1, 8))
            skill_path.write_text(
                self._skill_doc(
                    f"""\
                    ---
                    id: alias-skill
                    stage: Z_cross_cutting
                    description: "Alias summary."
                    ---

                    # Alias Skill

                    ## Purpose

                    Short purpose.

                    ## Process

                    Some process.

                    {body}
                    """
                )
                + ("\nextra-line" * 120),
                encoding="utf-8",
            )

            report = ValidationReport()
            validate_skill_structure(root, report)

        error_blob = "\n".join(report.errors)
        self.assertIn("thin stubs", error_blob)
        self.assertIn("does not mention the canonical skill id", error_blob)

    def test_alias_skill_lint_accepts_thin_stub_with_canonical_pointer(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self._write_registry(
                root,
                [
                    {
                        "id": "alias-skill",
                        "stage": "Z_cross_cutting",
                        "file": "skills/Z_cross_cutting/alias-skill.md",
                        "canonical": False,
                        "deprecated": True,
                        "alias_of": "canonical-skill",
                        "summary": "Alias summary.",
                    }
                ],
            )
            skill_path = root / "skills" / "Z_cross_cutting" / "alias-skill.md"
            skill_path.parent.mkdir(parents=True, exist_ok=True)
            skill_path.write_text(
                self._skill_doc(
                    """\
                    ---
                    id: alias-skill
                    stage: Z_cross_cutting
                    description: "Alias summary."
                    ---

                    # Alias Skill

                    ## Purpose

                    This is a thin alias stub. Use `canonical-skill` for the canonical implementation.

                    ## Process

                    Redirect to `canonical-skill` and avoid maintaining duplicate logic here.

                    ## When to Use

                    Only when following older references.

                    ## Common Pitfalls

                    - Do not expand this alias
                    """
                ),
                encoding="utf-8",
            )

            report = ValidationReport()
            validate_skill_structure(root, report)

        self.assertEqual(report.errors, [])
        self.assertEqual(report.warnings, [])


if __name__ == "__main__":
    unittest.main()
