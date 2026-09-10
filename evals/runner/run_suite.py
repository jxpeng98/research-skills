#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from evals.runner.run_eval import main as run_case_cli, run_case  # noqa: E402


DEFAULT_CASE_DIR = REPO_ROOT / "evals" / "academic_quality" / "cases"


@dataclass(frozen=True)
class EvalRunResult:
    case_count: int
    passed_cases: int
    failed_cases: int

    @property
    def success(self) -> bool:
        return self.case_count > 0 and self.failed_cases == 0


def run_evals(case_dir: Path, fixture_root: Path | None = None,
              receipt_root: Path | None = None) -> EvalRunResult:
    case_dir = Path(case_dir)
    case_paths = sorted(case_dir.glob("*.yaml"), key=lambda path: path.name)
    outputs = Path(fixture_root) if fixture_root is not None else case_dir.parent / "fixtures"
    passed_cases = sum(
        (run_case(case_path, outputs / case_path.stem) if receipt_root is None else
         run_case_cli([str(case_path), str(outputs / case_path.stem), "--json-receipt",
                       str(receipt_root / f"{case_path.stem}.json")]) == 0)
        for case_path in case_paths
    )
    return EvalRunResult(
        case_count=len(case_paths),
        passed_cases=passed_cases,
        failed_cases=len(case_paths) - passed_cases,
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate the captured Evaluation Truth V1 suite offline."
    )
    parser.add_argument("case_dir", nargs="?", type=Path, default=DEFAULT_CASE_DIR)
    parser.add_argument("--fixture-root", type=Path)
    parser.add_argument("--receipt-root", type=Path)
    args = parser.parse_args(argv)

    result = run_evals(args.case_dir, args.fixture_root, args.receipt_root)
    status = "PASS" if result.success else "FAIL"
    print(
        f"[{status}] {result.passed_cases} passed, {result.failed_cases} failed "
        f"across {result.case_count} academic-quality cases"
    )
    return 0 if result.success else 1


if __name__ == "__main__":
    raise SystemExit(main())
