from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = REPO_ROOT / "scripts" / "validate_project_artifacts.py"

spec = importlib.util.spec_from_file_location("validate_project_artifacts", MODULE_PATH)
assert spec is not None and spec.loader is not None
validate_project_artifacts = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = validate_project_artifacts
spec.loader.exec_module(validate_project_artifacts)


class ValidateProjectArtifactsTests(unittest.TestCase):
    def test_expected_task_id_accepts_j_and_k_stages(self) -> None:
        pattern = validate_project_artifacts.EXPECTED_TASK_ID

        self.assertTrue(pattern.match("J1"))
        self.assertTrue(pattern.match("J4"))
        self.assertTrue(pattern.match("K1"))
        self.assertTrue(pattern.match("K4"))

    def test_expected_task_id_still_rejects_unknown_stage_letters(self) -> None:
        pattern = validate_project_artifacts.EXPECTED_TASK_ID

        self.assertFalse(pattern.match("L1"))
        self.assertFalse(pattern.match("Z9"))

    def test_build_task_plan_supports_j_and_k_tasks_from_contract(self) -> None:
        contract = validate_project_artifacts.read_contract(REPO_ROOT)

        j_plan = validate_project_artifacts.build_task_plan(contract, "J1")
        k_plan = validate_project_artifacts.build_task_plan(contract, "K1")

        self.assertEqual(j_plan["task_id"], "J1")
        self.assertTrue(j_plan["requires_all_order"])
        self.assertEqual(k_plan["task_id"], "K1")
        self.assertTrue(k_plan["requires_all_order"])


if __name__ == "__main__":
    unittest.main()
