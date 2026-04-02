from __future__ import annotations

import csv
import io
import re
from pathlib import Path
from typing import Any

from bridges.providers.metadata_registry import artifact_project_root


TABLE_ROW_SPLIT_RE = re.compile(r"\s*\|\s*")
HEADER_ALIASES = {
    "decision_include_exclude": "decision",
    "exclude_reason": "exclusion_reason",
}


def run_screening_tracker(task_packet: dict[str, Any], cwd: Path) -> dict[str, Any]:
    project_root = artifact_project_root(task_packet, cwd)
    screening_dir = project_root / "screening"
    title_abstract_path = screening_dir / "title_abstract.md"
    full_text_path = screening_dir / "full_text.md"
    prisma_flow_path = screening_dir / "prisma_flow.md"
    retrieval_manifest_path = project_root / "retrieval_manifest.csv"

    title_abstract_rows = _parse_title_abstract_rows(title_abstract_path)
    full_text_rows = _parse_full_text_rows(full_text_path)
    manifest_rows = _parse_manifest_rows(retrieval_manifest_path)
    prisma_counts = _parse_prisma_counts(prisma_flow_path)

    checkpoint_state = _build_checkpoint_state(
        title_abstract_path=title_abstract_path,
        title_abstract_rows=title_abstract_rows,
        full_text_path=full_text_path,
        full_text_rows=full_text_rows,
        prisma_flow_path=prisma_flow_path,
        prisma_counts=prisma_counts,
        retrieval_manifest_path=retrieval_manifest_path,
        manifest_rows=manifest_rows,
    )
    resume_state = _build_resume_state(
        title_abstract_rows=title_abstract_rows,
        full_text_rows=full_text_rows,
        manifest_rows=manifest_rows,
        prisma_counts=prisma_counts,
    )
    provenance = [
        str(path)
        for path in (
            title_abstract_path,
            full_text_path,
            prisma_flow_path,
            retrieval_manifest_path,
        )
        if path.exists()
    ]

    summary = (
        "Builtin screening tracker checkpoint stub inspected "
        f"{len(provenance)} literature workflow artifacts and derived "
        f"{len(resume_state['next_actions'])} next-step checkpoints."
    )
    status = "ok" if provenance else "warning"
    if provenance and resume_state["bundle_status"] != "ready":
        status = "warning"

    return {
        "status": status,
        "summary": summary,
        "provenance": provenance,
        "data": {
            "provider_mode": "builtin_screening_checkpoint_stub",
            "project_root": str(project_root),
            "checkpoint_state": checkpoint_state,
            "resume_state": resume_state,
            "title_abstract_rows": title_abstract_rows[:200],
            "full_text_rows": full_text_rows[:200],
            "retrieval_manifest_rows": manifest_rows[:200],
            "prisma_flow_counts": prisma_counts,
        },
    }


def _build_checkpoint_state(
    *,
    title_abstract_path: Path,
    title_abstract_rows: list[dict[str, str]],
    full_text_path: Path,
    full_text_rows: list[dict[str, str]],
    prisma_flow_path: Path,
    prisma_counts: dict[str, int],
    retrieval_manifest_path: Path,
    manifest_rows: list[dict[str, str]],
) -> dict[str, Any]:
    manifest_pending = [
        row
        for row in manifest_rows
        if row.get("retrieval_status", "").startswith("not_retrieved:")
    ]
    manifest_accessible = [
        row
        for row in manifest_rows
        if row.get("retrieval_status", "") in {"retrieved_oa", "retrieved_preprint", "abstract_only"}
    ]
    full_text_decided = [
        row for row in full_text_rows if row.get("decision", "").lower() in {"include", "exclude"}
    ]
    return {
        "title_abstract": {
            "artifact": "screening/title_abstract.md",
            "exists": title_abstract_path.exists(),
            "row_count": len(title_abstract_rows),
            "completed": len(title_abstract_rows) > 0,
        },
        "retrieval_manifest": {
            "artifact": "retrieval_manifest.csv",
            "exists": retrieval_manifest_path.exists(),
            "row_count": len(manifest_rows),
            "pending_count": len(manifest_pending),
            "accessible_count": len(manifest_accessible),
            "completed": retrieval_manifest_path.exists() and len(manifest_pending) == 0,
        },
        "full_text": {
            "artifact": "screening/full_text.md",
            "exists": full_text_path.exists(),
            "row_count": len(full_text_rows),
            "decision_count": len(full_text_decided),
            "completed": full_text_path.exists() and len(full_text_decided) > 0,
        },
        "prisma_flow": {
            "artifact": "screening/prisma_flow.md",
            "exists": prisma_flow_path.exists(),
            "count_keys": sorted(prisma_counts.keys()),
            "completed": prisma_flow_path.exists() and bool(prisma_counts),
        },
    }


def _build_resume_state(
    *,
    title_abstract_rows: list[dict[str, str]],
    full_text_rows: list[dict[str, str]],
    manifest_rows: list[dict[str, str]],
    prisma_counts: dict[str, int],
) -> dict[str, Any]:
    title_abstract_complete = len(title_abstract_rows) > 0
    pending_manifest = [
        row for row in manifest_rows if row.get("retrieval_status", "").startswith("not_retrieved:")
    ]
    accessible_manifest = [
        row
        for row in manifest_rows
        if row.get("retrieval_status", "") in {"retrieved_oa", "retrieved_preprint", "abstract_only"}
    ]
    decided_ids = {
        row.get("record_id", "").strip()
        for row in full_text_rows
        if row.get("decision", "").strip().lower() in {"include", "exclude"}
    }
    undecided_accessible = [
        row
        for row in accessible_manifest
        if row.get("record_id", "").strip() not in decided_ids
    ]

    next_actions: list[dict[str, Any]] = []
    if not title_abstract_complete:
        next_actions.append(
            {
                "step": "title_abstract_screening",
                "status": "pending",
                "reason": "No stage-1 screening rows were found.",
            }
        )
    if pending_manifest:
        next_actions.append(
            {
                "step": "fulltext_retrieval",
                "status": "in_progress",
                "pending_records": len(pending_manifest),
                "reason": "Retrieval manifest still contains unresolved full-text rows.",
            }
        )
    if undecided_accessible:
        next_actions.append(
            {
                "step": "fulltext_screening",
                "status": "in_progress",
                "pending_records": len(undecided_accessible),
                "reason": "Accessible full-text rows exist without include/exclude decisions.",
            }
        )
    if title_abstract_complete and not pending_manifest and not undecided_accessible and not prisma_counts:
        next_actions.append(
            {
                "step": "prisma_flow_refresh",
                "status": "pending",
                "reason": "Screening artifacts exist but PRISMA flow counts were not found.",
            }
        )

    bundle_status = "ready" if not next_actions and (title_abstract_complete or full_text_rows or manifest_rows) else "not_started"
    if next_actions:
        bundle_status = "in_progress"

    return {
        "bundle_status": bundle_status,
        "next_actions": next_actions,
        "decided_full_text_records": len(decided_ids),
        "pending_retrieval_records": len(pending_manifest),
        "pending_full_text_decisions": len(undecided_accessible),
    }


def _parse_title_abstract_rows(path: Path) -> list[dict[str, str]]:
    if not path.exists():
        return []
    text = _read_text(path)
    rows = _parse_markdown_table_by_headers(text, {"id", "title", "year", "include", "exclude reason"})
    if rows:
        return rows
    return _parse_markdown_table_by_headers(text, {"id", "title", "include"})


def _parse_full_text_rows(path: Path) -> list[dict[str, str]]:
    if not path.exists():
        return []
    text = _read_text(path)
    return _parse_markdown_table_by_headers(
        text,
        {"record_id", "decision", "exclusion_reason", "fulltext_status", "notes"},
    )


def _parse_manifest_rows(path: Path) -> list[dict[str, str]]:
    if not path.exists():
        return []
    text = _read_text(path)
    reader = csv.DictReader(io.StringIO(text))
    rows: list[dict[str, str]] = []
    for row in reader:
        rows.append({str(key): " ".join(str(value or "").split()) for key, value in row.items() if key})
    return rows


def _parse_prisma_counts(path: Path) -> dict[str, int]:
    if not path.exists():
        return {}
    text = _read_text(path)
    counts: dict[str, int] = {}
    for line in text.splitlines():
        if ":" not in line:
            continue
        key, raw_value = line.split(":", 1)
        match = re.search(r"\b(\d+)\b", raw_value)
        if not match:
            continue
        normalized_key = "_".join(key.strip().lower().split())
        counts[normalized_key] = int(match.group(1))
    return counts


def _parse_markdown_table_by_headers(text: str, expected_headers: set[str]) -> list[dict[str, str]]:
    lines = [line.rstrip() for line in text.splitlines()]
    for index in range(len(lines) - 1):
        header_line = lines[index]
        divider_line = lines[index + 1]
        if "|" not in header_line or "|" not in divider_line:
            continue
        headers = _split_markdown_row(header_line)
        divider = _split_markdown_row(divider_line)
        normalized_headers = {_canonical_header(_normalize_header(header)) for header in headers}
        if not expected_headers.issubset(normalized_headers):
            continue
        if not divider or not all(set(cell) <= {"-", ":"} for cell in divider):
            continue
        rows: list[dict[str, str]] = []
        for row_line in lines[index + 2 :]:
            if "|" not in row_line:
                break
            cells = _split_markdown_row(row_line)
            if len(cells) != len(headers):
                break
            rows.append(
                {
                    _canonical_header(_normalize_header(header)): value
                    for header, value in zip(headers, cells, strict=True)
                }
            )
        return rows
    return []


def _split_markdown_row(line: str) -> list[str]:
    stripped = line.strip()
    if stripped.startswith("|"):
        stripped = stripped[1:]
    if stripped.endswith("|"):
        stripped = stripped[:-1]
    return [" ".join(cell.split()) for cell in TABLE_ROW_SPLIT_RE.split(stripped)]


def _normalize_header(value: str) -> str:
    lowered = value.strip().lower()
    normalized = re.sub(r"[^a-z0-9]+", "_", lowered).strip("_")
    return normalized


def _canonical_header(value: str) -> str:
    return HEADER_ALIASES.get(value, value)


def _read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return path.read_text(encoding="utf-8", errors="ignore")
