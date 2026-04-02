#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from research_skills.team_run_acceptance import (
    render_team_run_receipt,
    run_team_run_command,
    summarize_team_run_result,
)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run a real team-run probe and capture a markdown acceptance receipt.",
    )
    parser.add_argument("--task-id", required=True)
    parser.add_argument(
        "--paper-type",
        required=True,
        choices=["empirical", "qualitative", "systematic-review", "methods", "theory"],
    )
    parser.add_argument("--topic", required=True)
    parser.add_argument("--cwd", required=True)
    parser.add_argument("--receipt", required=True, help="Output markdown receipt path.")
    parser.add_argument("--profile", default="default")
    parser.add_argument("--max-units", type=int)
    parser.add_argument("--strict-success", action="store_true")
    return parser


def git_commit(cwd: Path) -> str:
    completed = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=str(cwd),
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    if completed.returncode != 0:
        return "unknown"
    return completed.stdout.strip() or "unknown"


def main() -> int:
    args = build_parser().parse_args()
    cwd = Path(args.cwd).resolve()
    receipt_path = Path(args.receipt).resolve()

    command = [
        sys.executable,
        "-m",
        "bridges.orchestrator",
        "team-run",
        "--task-id",
        args.task_id,
        "--paper-type",
        args.paper_type,
        "--topic",
        args.topic,
        "--cwd",
        str(cwd),
        "--profile",
        args.profile,
    ]
    if args.max_units is not None:
        command.extend(["--max-units", str(args.max_units)])

    exit_code, output = run_team_run_command(command, cwd)
    try:
        raw_result = json.loads(output)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"Failed to parse orchestrator output as JSON: {exc}\n{output}") from exc

    summary = summarize_team_run_result(raw_result, command, exit_code)
    receipt = render_team_run_receipt(
        summary,
        generated_at=datetime.now(timezone.utc),
        git_commit=git_commit(cwd),
    )
    receipt_path.parent.mkdir(parents=True, exist_ok=True)
    receipt_path.write_text(receipt, encoding="utf-8")
    print(f"[WRITE] {receipt_path}")

    if args.strict_success and (
        summary.exit_code != 0 or summary.barrier_status.lower() == "blocked" or summary.confidence <= 0.0
    ):
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
