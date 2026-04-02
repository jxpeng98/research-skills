from __future__ import annotations

import unittest
from pathlib import Path

from research_skills.workflow_contract_doc import generate_workflow_contract_reference


REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_CONTRACT_DOC = REPO_ROOT / "research-paper-workflow" / "references" / "workflow-contract.md"


class WorkflowContractDocTests(unittest.TestCase):
    def test_generated_workflow_contract_matches_repo_copy(self) -> None:
        generated = generate_workflow_contract_reference(REPO_ROOT)
        actual = WORKFLOW_CONTRACT_DOC.read_text(encoding="utf-8")

        self.assertEqual(actual, generated)

    def test_generated_workflow_contract_includes_generated_marker_and_k_stage(self) -> None:
        generated = generate_workflow_contract_reference(REPO_ROOT)

        self.assertIn(
            "Auto-generated from `standards/research-workflow-contract.yaml`",
            generated,
        )
        self.assertIn("| `K4` | K | Beamer build | `presentation/beamer/`, `presentation/slides.bib` |", generated)
        self.assertIn("`references/stage-K-presentation.md`", generated)


if __name__ == "__main__":
    unittest.main()
