from __future__ import annotations

import io
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

import yaml

from scripts.run_academic_quality_evals import main, run_evals


REPO_ROOT = Path(__file__).resolve().parents[1]
CASE_DIR = REPO_ROOT / "evals" / "academic_quality" / "cases"
FIXTURE_ROOT = REPO_ROOT / "evals" / "academic_quality" / "fixtures"
CANONICAL_SUITE_PATH = REPO_ROOT / "evals" / "runner" / "run_suite.py"
LEGACY_SUITE_PATH = REPO_ROOT / "scripts" / "run_academic_quality_evals.py"
EXPECTED_CASES = {
    "code-first-methods.yaml",
    "economics-did-invalid-parallel-trends.yaml",
    "empirical-causal-design.yaml",
    "finance-event-study-leakage.yaml",
    "q1-rq-method-mismatch.yaml",
    "q2-unsupported-claim.yaml",
    "q3-reporting-gap.yaml",
    "q4-reproducibility-gap.yaml",
    "qualitative-coding.yaml",
    "reviewer-rebuttal.yaml",
    "systematic-review.yaml",
    "theory-contribution.yaml",
}


class AcademicQualityEvalTests(unittest.TestCase):
    def test_canonical_cli_is_cwd_independent_and_owns_legacy_api(self) -> None:
        self.assertEqual("evals.runner.run_suite", run_evals.__module__)

        with tempfile.TemporaryDirectory() as temp_dir:
            canonical = subprocess.run(
                [sys.executable, str(CANONICAL_SUITE_PATH)],
                cwd=temp_dir,
                capture_output=True,
                text=True,
                check=False,
            )
            legacy = subprocess.run(
                [sys.executable, str(LEGACY_SUITE_PATH), str(CASE_DIR)],
                cwd=temp_dir,
                capture_output=True,
                text=True,
                check=False,
            )

        self.assertEqual(0, canonical.returncode, canonical.stdout + canonical.stderr)
        self.assertEqual(0, legacy.returncode, legacy.stdout + legacy.stderr)
        canonical_summary = canonical.stdout.strip().splitlines()[-1]
        legacy_summary = legacy.stdout.strip().splitlines()[-1]
        self.assertEqual(canonical_summary, legacy_summary)
        self.assertIn("12 passed, 0 failed", canonical_summary)

    def test_canonical_cli_fails_closed_for_empty_and_failed_suites(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            case_dir = root / "cases"
            fixture_root = root / "fixtures"
            case_dir.mkdir()
            fixture_root.mkdir()

            empty = self._run_suite_cli(case_dir, fixture_root)
            self.assertEqual(1, empty.returncode, empty.stdout + empty.stderr)
            self.assertIn("0 passed, 0 failed", empty.stdout)

            (case_dir / "missing.yaml").write_text(
                yaml.safe_dump(
                    {
                        "schema_version": "1.0",
                        "case_id": "missing",
                        "pipeline": "academic-quality",
                        "input": {"topic": "missing evidence"},
                        "expected_outputs": {
                            "finding": {
                                "artifact": "quality_findings.md",
                                "required": True,
                                "assertions": [
                                    {
                                        "type": "contains_all",
                                        "values": ["required finding"],
                                    }
                                ],
                            }
                        },
                    },
                    sort_keys=False,
                ),
                encoding="utf-8",
            )
            failed = self._run_suite_cli(case_dir, fixture_root)

        self.assertEqual(1, failed.returncode, failed.stdout + failed.stderr)
        self.assertIn("0 passed, 1 failed", failed.stdout)

    def test_evaluation_ci_invokes_only_the_canonical_suite_path(self) -> None:
        workflow = (
            REPO_ROOT / ".github" / "workflows" / "evaluation-truth.yml"
        ).read_text(encoding="utf-8")

        self.assertEqual(1, workflow.count('branches: ["2.x"]'))
        self.assertNotIn("\n  push:\n", workflow)
        self.assertEqual(1, workflow.count("python evals/runner/run_suite.py"))
        self.assertNotIn("run_academic_quality_evals.py", workflow)

    def test_optional_suite_receipts_preserve_pass_fail_and_missing_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as temporary, redirect_stdout(io.StringIO()):
            root = Path(temporary)
            case_dir, fixtures = root / "cases", root / "fixtures"
            case_dir.mkdir()
            name = "q2-unsupported-claim"
            shutil.copy2(CASE_DIR / f"{name}.yaml", case_dir)
            shutil.copytree(FIXTURE_ROOT / name, fixtures / name)
            artifact = fixtures / name / "quality_findings.md"
            for label, content in (("valid", artifact.read_text()), ("wrong", "unrelated"), ("missing", None)):
                if content is None:
                    artifact.unlink()
                else:
                    artifact.write_text(content)
                self.assertEqual(label == "valid", run_evals(case_dir, fixtures, root / label).success)
                receipt = json.loads((root / label / f"{name}.json").read_text())
                self.assertEqual(label == "valid", receipt["case"]["status"] == "pass")
                if label == "missing":
                    self.assertGreater(receipt["summary"]["required_missing"], 0)

    def test_all_cases_use_v1_inputs_and_contained_fixture_assertions(self) -> None:
        self.assertEqual(EXPECTED_CASES, {path.name for path in CASE_DIR.glob("*.yaml")})

        for case_path in sorted(CASE_DIR.glob("*.yaml")):
            case = yaml.safe_load(case_path.read_text(encoding="utf-8"))
            with self.subTest(case=case_path.name):
                self.assertEqual("1.0", case.get("schema_version"))
                self.assertEqual(case_path.stem, case.get("case_id"))
                self.assertIsInstance(case.get("pipeline"), str)
                self.assertTrue(case["pipeline"].strip())
                self.assertNotIn("expected_dimensions", case)
                self.assertIsInstance(case.get("input"), dict)
                self.assertIsInstance(case["input"].get("topic"), str)
                self.assertTrue(case["input"]["topic"].strip())

                expected_outputs = case.get("expected_outputs")
                self.assertIsInstance(expected_outputs, dict)
                self.assertTrue(expected_outputs)
                fixture_dir = (FIXTURE_ROOT / case["case_id"]).resolve()
                self.assertTrue(fixture_dir.is_relative_to(FIXTURE_ROOT.resolve()))
                for expected in expected_outputs.values():
                    self.assertIs(type(expected.get("required")), bool)
                    self.assertTrue(expected["required"])
                    assertions = expected.get("assertions")
                    self.assertIsInstance(assertions, list)
                    self.assertTrue(assertions)
                    artifact = (fixture_dir / expected["artifact"]).resolve()
                    self.assertTrue(artifact.is_relative_to(fixture_dir))
                    self.assertTrue(artifact.is_file(), artifact)

    def test_batch_executes_all_twelve_cases_without_scores(self) -> None:
        result = self._run_evals(CASE_DIR, FIXTURE_ROOT)

        self.assertEqual(12, result.case_count)
        self.assertEqual(12, result.passed_cases)
        self.assertEqual(0, result.failed_cases)
        self.assertTrue(result.success)
        self.assertFalse(hasattr(result, "dimension_scores"))

        stdout = io.StringIO()
        with redirect_stdout(stdout):
            self.assertEqual(0, main([str(CASE_DIR)]))
        self.assertIn("12 passed, 0 failed", stdout.getvalue())

    def test_removing_a_required_finding_makes_the_batch_fail(self) -> None:
        finding = (
            "Central causal claim is not linked to evidence ledger entries or "
            "verified citations."
        )
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            case_dir = root / "cases"
            fixture_root = root / "fixtures"
            case_dir.mkdir()
            shutil.copy2(CASE_DIR / "q2-unsupported-claim.yaml", case_dir)
            shutil.copytree(
                FIXTURE_ROOT / "q2-unsupported-claim",
                fixture_root / "q2-unsupported-claim",
            )
            artifact = fixture_root / "q2-unsupported-claim" / "quality_findings.md"
            content = artifact.read_text(encoding="utf-8")
            self.assertIn(finding, content)
            artifact.write_text(content.replace(finding, ""), encoding="utf-8")

            result = self._run_evals(case_dir, fixture_root)

        self.assertEqual(1, result.case_count)
        self.assertEqual(0, result.passed_cases)
        self.assertEqual(1, result.failed_cases)
        self.assertFalse(result.success)

    def test_batch_fails_closed_for_invalid_or_missing_evidence(self) -> None:
        valid_case = {
            "schema_version": "1.0",
            "case_id": "case",
            "pipeline": "academic-quality",
            "input": {"topic": "bounded academic quality check"},
            "expected_outputs": {
                "finding": {
                    "artifact": "quality_findings.md",
                    "required": True,
                    "assertions": [
                        {"type": "contains_all", "values": ["required finding"]}
                    ],
                }
            },
        }
        scenarios = {
            "missing output": valid_case,
            "zero assertions": self._case_with(valid_case, assertions=[]),
            "unknown assertion": self._case_with(
                valid_case, assertions=[{"type": "unknown-finding"}]
            ),
            "path escape": self._case_with(
                valid_case, artifact="../quality_findings.md"
            ),
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            empty_root = Path(temp_dir)
            (empty_root / "cases").mkdir()
            (empty_root / "fixtures").mkdir()
            self.assertFalse(
                self._run_evals(
                    empty_root / "cases", empty_root / "fixtures"
                ).success
            )

        for name, case in scenarios.items():
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temp_dir:
                root = Path(temp_dir)
                case_dir = root / "cases"
                fixture_root = root / "fixtures"
                case_dir.mkdir()
                (fixture_root / "case").mkdir(parents=True)
                (case_dir / "case.yaml").write_text(
                    yaml.safe_dump(case, sort_keys=False), encoding="utf-8"
                )
                if name != "missing output":
                    (fixture_root / "case" / "quality_findings.md").write_text(
                        "required finding\n", encoding="utf-8"
                    )
                    (fixture_root / "quality_findings.md").write_text(
                        "required finding\n", encoding="utf-8"
                    )

                result = self._run_evals(case_dir, fixture_root)

                self.assertEqual(1, result.failed_cases)
                self.assertFalse(result.success)

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            case_dir = root / "cases"
            fixture_root = root / "fixtures"
            case_dir.mkdir()
            fixture_root.mkdir()
            (case_dir / "case.yaml").write_text("schema_version: [", encoding="utf-8")
            result = self._run_evals(case_dir, fixture_root)
            self.assertEqual(1, result.failed_cases)
            self.assertFalse(result.success)

    @staticmethod
    def _case_with(case: dict, **changes: object) -> dict:
        updated = yaml.safe_load(yaml.safe_dump(case))
        updated["expected_outputs"]["finding"].update(changes)
        return updated

    @staticmethod
    def _run_evals(case_dir: Path, fixture_root: Path):
        with redirect_stdout(io.StringIO()):
            return run_evals(case_dir, fixture_root)

    @staticmethod
    def _run_suite_cli(
        case_dir: Path, fixture_root: Path
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable,
                str(CANONICAL_SUITE_PATH),
                str(case_dir),
                "--fixture-root",
                str(fixture_root),
            ],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )


if __name__ == "__main__":
    unittest.main()
