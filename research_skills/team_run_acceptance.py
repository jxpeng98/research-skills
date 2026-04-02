from __future__ import annotations

import json
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


@dataclass
class TeamRunAcceptanceSummary:
    task_id: str
    paper_type: str
    topic: str
    command: list[str]
    exit_code: int
    confidence: float
    barrier_status: str
    work_units: str
    successful_shards: str
    profile: str
    blocking_observations: list[str]
    routing_notes: list[str]
    raw_result: dict[str, Any]


def extract_section_lines(merged_analysis: str, heading: str) -> list[str]:
    lines = merged_analysis.splitlines()
    capture = False
    collected: list[str] = []
    for line in lines:
        if line.startswith("## "):
            if capture:
                break
            capture = line.strip() == heading
            continue
        if capture:
            collected.append(line)
    return collected


def extract_metric(merged_analysis: str, label: str) -> str:
    prefix = f"- {label}: "
    for line in merged_analysis.splitlines():
        if line.startswith(prefix):
            return line[len(prefix):].strip()
    return "unknown"


def summarize_team_run_result(
    raw_result: dict[str, Any],
    command: list[str],
    exit_code: int,
) -> TeamRunAcceptanceSummary:
    task_description = str(raw_result.get("task_description", ""))
    parts = task_description.split(" ", 2)
    task_id = parts[0] if len(parts) >= 1 else "unknown"
    paper_type = parts[1] if len(parts) >= 2 else "unknown"
    topic = parts[2] if len(parts) >= 3 else "unknown"
    merged_analysis = str(raw_result.get("merged_analysis", ""))
    routing_notes = [
        line[2:].strip()
        for line in extract_section_lines(merged_analysis, "## Routing Notes")
        if line.startswith("- ")
    ]
    observations = [
        note for note in routing_notes
        if any(token in note.lower() for token in ("failed", "blocked", "warning", "error", "not_configured"))
    ]
    return TeamRunAcceptanceSummary(
        task_id=task_id,
        paper_type=paper_type,
        topic=topic,
        command=command,
        exit_code=exit_code,
        confidence=float(raw_result.get("confidence", 0.0)),
        barrier_status=extract_metric(merged_analysis, "Barrier status"),
        work_units=extract_metric(merged_analysis, "Work units dispatched"),
        successful_shards=extract_metric(merged_analysis, "Successful shards"),
        profile=extract_metric(merged_analysis, "Profile"),
        blocking_observations=observations,
        routing_notes=routing_notes,
        raw_result=raw_result,
    )


def render_team_run_receipt(
    summary: TeamRunAcceptanceSummary,
    generated_at: datetime,
    git_commit: str,
) -> str:
    command_str = " ".join(summary.command)
    observations = "\n".join(f"- {item}" for item in summary.blocking_observations) or "- none"
    routing_notes = "\n".join(f"- {item}" for item in summary.routing_notes) or "- none"
    raw_json = json.dumps(summary.raw_result, indent=2, ensure_ascii=False)
    return f"""# Team-Run Acceptance Receipt — {summary.task_id}

- Generated At: {generated_at.astimezone(timezone.utc).strftime("%Y-%m-%d %H:%M:%SZ")}
- Task ID: {summary.task_id}
- Paper Type: {summary.paper_type}
- Topic: {summary.topic}
- Git Commit: {git_commit}
- Command Exit Code: {summary.exit_code}

## Command

```bash
{command_str}
```

## Outcome Summary

- Confidence: {summary.confidence:.2f}
- Barrier Status: {summary.barrier_status}
- Work Units Dispatched: {summary.work_units}
- Successful Shards: {summary.successful_shards}
- Profile: {summary.profile}

## Blocking / Degrade Observations

{observations}

## Routing Notes

{routing_notes}

## Raw Result JSON

```json
{raw_json}
```
"""


def run_team_run_command(command: list[str], cwd: Path) -> tuple[int, str]:
    completed = subprocess.run(
        command,
        cwd=str(cwd),
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    stdout = completed.stdout.strip()
    stderr = completed.stderr.strip()
    combined = stdout if not stderr else f"{stdout}\n{stderr}".strip()
    return completed.returncode, combined

