from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

import yaml


REPO_ROOT = Path(__file__).resolve().parents[1]
EVAL_CASES_DIR = REPO_ROOT / "eval" / "cases"
PIPELINES_DIR = REPO_ROOT / "pipelines"
RUN_EVAL_PATH = REPO_ROOT / "eval" / "runner" / "run_eval.py"
TARGET_PIPELINES = {"systematic-review-prisma", "empirical-study", "theory-paper"}


def load_yaml(path: Path) -> dict:
    return yaml.safe_load(path.read_text(encoding="utf-8"))


def load_eval_runner():
    spec = importlib.util.spec_from_file_location("test_eval_runner_module", RUN_EVAL_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load eval runner from {RUN_EVAL_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


RUN_EVAL = load_eval_runner()


class EvalCaseCoverageTests(unittest.TestCase):
    def test_every_eval_case_references_an_existing_pipeline(self) -> None:
        pipeline_ids = {load_yaml(path)["id"] for path in sorted(PIPELINES_DIR.glob("*.yaml"))}

        for case_path in sorted(EVAL_CASES_DIR.glob("*.yaml")):
            case = load_yaml(case_path)
            self.assertIn(case["pipeline"], pipeline_ids, case_path.name)

    def test_every_eval_case_skill_matches_its_pipeline(self) -> None:
        pipelines = {
            load_yaml(path)["id"]: load_yaml(path)
            for path in sorted(PIPELINES_DIR.glob("*.yaml"))
        }

        for case_path in sorted(EVAL_CASES_DIR.glob("*.yaml")):
            case = load_yaml(case_path)
            pipeline_skills = {step["skill"] for step in pipelines[case["pipeline"]]["steps"]}
            expected_skills = set(case.get("expected_outputs", {}))
            self.assertTrue(expected_skills.issubset(pipeline_skills), case_path.name)

    def test_target_refactored_pipelines_have_eval_cases(self) -> None:
        covered_pipelines = {
            load_yaml(path)["pipeline"] for path in sorted(EVAL_CASES_DIR.glob("*.yaml"))
        }
        self.assertTrue(TARGET_PIPELINES.issubset(covered_pipelines))

    def test_eval_runner_passes_for_all_cases_with_minimal_fixtures(self) -> None:
        for case_path in sorted(EVAL_CASES_DIR.glob("*.yaml")):
            case = load_yaml(case_path)
            with self.subTest(case=case["case_id"]):
                with tempfile.TemporaryDirectory() as temp_dir:
                    output_dir = Path(temp_dir)
                    self._materialize_case_outputs(case, output_dir)
                    self.assertTrue(RUN_EVAL.run_case(str(case_path), str(output_dir)))

    def _materialize_case_outputs(self, case: dict, output_dir: Path) -> None:
        for expected in case.get("expected_outputs", {}).values():
            artifact = expected["artifact"]
            content = self._build_content(expected.get("must_contain", []))
            artifact_path = output_dir / artifact

            if artifact.endswith("/"):
                artifact_path.mkdir(parents=True, exist_ok=True)
                (artifact_path / "fixture.md").write_text(content, encoding="utf-8")
                continue

            artifact_path.parent.mkdir(parents=True, exist_ok=True)
            artifact_path.write_text(content, encoding="utf-8")

    def _build_content(self, patterns: list[object]) -> str:
        lines = []
        for pattern in patterns:
            text = str(pattern)
            if " or " in text:
                text = text.split(" or ", 1)[0]
            lines.append(text.strip().strip('"').strip("'"))
        return "\n".join(lines) + "\n"


if __name__ == "__main__":
    unittest.main()
