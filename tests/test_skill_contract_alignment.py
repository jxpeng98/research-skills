from __future__ import annotations

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
QUALITATIVE_CODING = REPO_ROOT / "skills" / "E_synthesis" / "qualitative-coding.md"
DISCUSSION_WRITER = REPO_ROOT / "skills" / "F_writing" / "discussion-writer.md"
LIMITATION_AUDITOR = REPO_ROOT / "skills" / "H_submission" / "limitation-auditor.md"
CAPABILITY_MAP = REPO_ROOT / "standards" / "mcp-agent-capability-map.yaml"
WORKFLOW_CONTRACT = REPO_ROOT / "standards" / "research-workflow-contract.yaml"
WORKFLOW_REFERENCE = REPO_ROOT / "research-paper-workflow" / "references" / "workflow-contract.md"
STAGE_E_REFERENCE = REPO_ROOT / "research-paper-workflow" / "references" / "stage-E-synthesis.md"


class SkillContractAlignmentTests(unittest.TestCase):
    def test_skill_files_use_canonical_artifact_paths(self) -> None:
        qualitative = QUALITATIVE_CODING.read_text(encoding="utf-8")
        discussion = DISCUSSION_WRITER.read_text(encoding="utf-8")
        limitation = LIMITATION_AUDITOR.read_text(encoding="utf-8")

        self.assertIn('artifact: "synthesis/qualitative_data_dictionary.md"', qualitative)
        self.assertIn('artifact: "synthesis/thematic_codebook.md"', qualitative)
        self.assertNotIn('artifact: "data_dictionary.md"', qualitative)
        self.assertNotIn('artifact: "thematic_codebook.md"', qualitative)

        self.assertIn('artifact: "manuscript/discussion.md"', discussion)
        self.assertIn('artifact: "manuscript/discussion_story_spine.md"', discussion)
        self.assertNotIn('artifact: "manuscript_fragments/06_discussion.md"', discussion)
        self.assertNotIn('artifact: "tools/discussion_story_spine.md"', discussion)

        self.assertIn('artifact: "revision/limitations_audit.md"', limitation)
        self.assertIn('artifact: "revision/limitation_mitigations.md"', limitation)
        self.assertNotIn('artifact: "tools/limitations_audit.md"', limitation)
        self.assertNotIn('artifact: "tools/limitation_mitigations.md"', limitation)

    def test_capability_map_matches_canonical_outputs(self) -> None:
        content = CAPABILITY_MAP.read_text(encoding="utf-8")

        self.assertIn('      - "synthesis/qualitative_data_dictionary.md"', content)
        self.assertIn('      - "synthesis/thematic_codebook.md"', content)
        self.assertIn('      - "manuscript/discussion.md"', content)
        self.assertIn('      - "manuscript/discussion_story_spine.md"', content)
        self.assertIn('      - "revision/limitations_audit.md"', content)
        self.assertIn('      - "revision/limitation_mitigations.md"', content)
        self.assertNotIn('      - "manuscript_fragments/06_discussion.md"', content)
        self.assertNotIn('      - "tools/discussion_story_spine.md"', content)
        self.assertNotIn('      - "tools/limitations_audit.md"', content)
        self.assertNotIn('      - "tools/limitation_mitigations.md"', content)

    def test_qualitative_coding_is_attached_to_integrated_synthesis(self) -> None:
        content = CAPABILITY_MAP.read_text(encoding="utf-8")
        e4_match = re.search(r"^  E4:\n(?P<body>(?:    .*\n)+)", content, re.MULTILINE)
        e5_match = re.search(r"^  E5:\n(?P<body>(?:    .*\n)+)", content, re.MULTILINE)

        self.assertIsNotNone(e4_match)
        self.assertIsNotNone(e5_match)
        self.assertNotIn('      - "qualitative-coding"', e4_match.group("body"))
        self.assertIn('      - "qualitative-coding"', e5_match.group("body"))

    def test_workflow_contract_includes_new_canonical_outputs(self) -> None:
        contract = WORKFLOW_CONTRACT.read_text(encoding="utf-8")
        reference = WORKFLOW_REFERENCE.read_text(encoding="utf-8")
        stage_e = STAGE_E_REFERENCE.read_text(encoding="utf-8")

        for expected in (
            "synthesis/qualitative_data_dictionary.md",
            "synthesis/thematic_codebook.md",
            "manuscript/discussion.md",
            "manuscript/discussion_story_spine.md",
            "revision/limitations_audit.md",
            "revision/limitation_mitigations.md",
        ):
            self.assertIn(expected, contract)
            self.assertIn(expected, reference)

        self.assertIn("synthesis/qualitative_data_dictionary.md", stage_e)
        self.assertIn("synthesis/thematic_codebook.md", stage_e)


if __name__ == "__main__":
    unittest.main()
