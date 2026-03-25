from __future__ import annotations

import csv
import io
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from bridges.providers.literature_search import record_match_key
from bridges.providers.metadata_registry import (
    artifact_project_root,
    collect_reference_records,
    merge_reference_records,
    read_candidate_files,
)


def run_fulltext_retrieval(
    task_packet: dict[str, Any],
    cwd: Path,
    *,
    retrieved_at: str | None = None,
) -> dict[str, Any]:
    project_root = artifact_project_root(task_packet, cwd)
    required_outputs = [
        str(item) for item in task_packet.get("required_outputs", []) if str(item).strip()
    ]
    timestamp = retrieved_at or datetime.now(timezone.utc).replace(microsecond=0).isoformat()

    reference_records = collect_reference_records(project_root, required_outputs)
    merged_records, dedup_log = merge_reference_records(reference_records)
    existing_manifest = _read_existing_manifest(project_root / "retrieval_manifest.csv")
    search_candidates, candidate_provenance = _collect_search_candidates(project_root, required_outputs)
    manifest_rows = _build_manifest_rows(
        merged_records,
        existing_manifest,
        search_candidates,
        project_root=project_root,
        retrieved_at=timestamp,
    )
    screening_rows = _build_screening_rows(manifest_rows)
    summary_counts = _summarize_manifest(manifest_rows)

    if manifest_rows:
        summary = (
            "Builtin fulltext stub prepared "
            f"{len(manifest_rows)} retrieval rows "
            f"({summary_counts['resolved_locally']} resolved locally, "
            f"{summary_counts['oa_candidates']} OA/manual candidates, "
            f"{summary_counts['unresolved']} unresolved)."
        )
        status = "ok"
    else:
        summary = (
            "Builtin fulltext stub is available, but no local literature records were found "
            "to prepare retrieval planning artifacts."
        )
        status = "warning"

    provenance = candidate_provenance
    manifest_path = project_root / "retrieval_manifest.csv"
    if manifest_path.exists():
        provenance.append(str(manifest_path))
    screening_path = project_root / "screening" / "full_text.md"
    if screening_path.exists():
        provenance.append(str(screening_path))

    return {
        "status": status,
        "summary": summary,
        "provenance": list(dict.fromkeys(provenance))[:8],
        "data": {
            "provider_mode": "builtin_fulltext_manifest_stub",
            "project_root": str(project_root),
            "record_count": len(merged_records),
            "retrieval_manifest": manifest_rows[:200],
            "screening_full_text_rows": screening_rows[:200],
            "summary_counts": summary_counts,
            "artifact_bundle": _artifact_bundle(),
            "dedup_log": dedup_log[:100],
            "existing_manifest_count": len(existing_manifest),
            "external_resolver_hint": (
                "Set RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD to connect Zotero or another "
                "full-text resolver for actual downloads."
            ),
        },
    }


def _read_existing_manifest(path: Path) -> list[dict[str, str]]:
    if not path.exists():
        return []
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        text = path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return []

    rows: list[dict[str, str]] = []
    reader = csv.DictReader(io.StringIO(text))
    for row in reader:
        rows.append(
            {
                "record_id": _clean(row.get("record_id")),
                "citekey": _clean(row.get("citekey")),
                "doi": _clean(row.get("doi")).lower(),
                "retrieval_status": _clean(row.get("retrieval_status")),
                "version_label": _clean(row.get("version_label")),
                "source_provider": _clean(row.get("source_provider")),
                "retrieved_at": _clean(row.get("retrieved_at")),
                "fulltext_path": _clean(row.get("fulltext_path")),
                "access_url": _clean(row.get("access_url")),
                "license": _clean(row.get("license")),
                "notes": _clean(row.get("notes")),
            }
        )
    return rows


def _collect_search_candidates(
    project_root: Path,
    required_outputs: list[str],
) -> tuple[dict[str, dict[str, str]], list[str]]:
    candidates: dict[str, dict[str, str]] = {}
    provenance: list[str] = []

    for path in read_candidate_files(project_root, required_outputs):
        if path.name != "search_results.csv":
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            text = path.read_text(encoding="utf-8", errors="ignore")
        except OSError:
            continue
        provenance.append(str(path))
        reader = csv.DictReader(io.StringIO(text))
        for row in reader:
            title = _clean(row.get("title"))
            doi = _clean(row.get("doi")).lower()
            paper_id = _clean(row.get("paper_id"))
            record = {
                "record_id": _clean(row.get("record_id")),
                "title": title,
                "year": _safe_int(row.get("year")),
                "doi": doi,
                "paper_id": paper_id,
            }
            key = record_match_key(record)[0]
            access_url = _clean(row.get("open_access_pdf_url")) or _clean(row.get("url"))
            if not access_url:
                continue
            source_provider = (
                "semantic_scholar_oa_candidate"
                if _clean(row.get("open_access_pdf_url"))
                else "search_result_locator"
            )
            existing = candidates.get(key)
            if existing and existing.get("access_url"):
                continue
            candidates[key] = {
                "access_url": access_url,
                "source_provider": source_provider,
                "paper_id": paper_id,
            }
    return candidates, provenance


def _build_manifest_rows(
    records: list[dict[str, Any]],
    existing_manifest: list[dict[str, str]],
    search_candidates: dict[str, dict[str, str]],
    *,
    project_root: Path,
    retrieved_at: str,
) -> list[dict[str, str]]:
    existing_by_key = _index_existing_manifest(existing_manifest)
    rows: list[dict[str, str]] = []
    consumed: set[int] = set()

    for record in records:
        existing_row, existing_index = _match_existing_row(record, existing_by_key)
        if existing_index is not None:
            consumed.add(existing_index)
        row = _build_manifest_row(
            record,
            existing_row or {},
            search_candidates.get(record_match_key(record)[0], {}),
            project_root=project_root,
            retrieved_at=retrieved_at,
        )
        rows.append(row)

    for index, row in enumerate(existing_manifest):
        if index in consumed:
            continue
        normalized = dict(row)
        resolved_path = _resolve_fulltext_path(project_root, normalized.get("fulltext_path", ""))
        if normalized.get("retrieval_status", "").startswith("retrieved") and not resolved_path:
            normalized["retrieval_status"] = "not_retrieved:path_missing"
            normalized["notes"] = _append_note(
                normalized.get("notes", ""),
                "Manifest referenced a missing fulltext_path.",
            )
        rows.append(normalized)

    return sorted(rows, key=lambda item: (_status_sort_key(item.get("retrieval_status", "")), item.get("citekey", ""), item.get("record_id", "")))


def _build_manifest_row(
    record: dict[str, Any],
    existing_row: dict[str, str],
    candidate: dict[str, str],
    *,
    project_root: Path,
    retrieved_at: str,
) -> dict[str, str]:
    record_id = str(existing_row.get("record_id") or record.get("record_id") or "").strip()
    citekey = str(existing_row.get("citekey") or record.get("citekey") or "").strip()
    doi = str(existing_row.get("doi") or record.get("doi") or "").strip().lower()
    access_url = (
        str(existing_row.get("access_url") or "").strip()
        or str(candidate.get("access_url") or "").strip()
        or str(record.get("url") or "").strip()
    )
    fulltext_path = str(existing_row.get("fulltext_path") or "").strip()
    resolved_path = _resolve_fulltext_path(project_root, fulltext_path)
    version_label = (
        str(existing_row.get("version_label") or "").strip()
        or _infer_version_label(access_url, record)
    )
    retrieval_status = _resolve_retrieval_status(
        existing_status=str(existing_row.get("retrieval_status") or "").strip(),
        resolved_path=resolved_path,
        access_url=access_url,
        doi=doi,
        fallback_url=str(record.get("url") or "").strip(),
    )
    source_provider = (
        str(existing_row.get("source_provider") or "").strip()
        or str(candidate.get("source_provider") or "").strip()
        or ("local_file" if resolved_path else "builtin_fulltext_stub")
    )
    notes = str(existing_row.get("notes") or "").strip()
    if not existing_row:
        if retrieval_status == "not_retrieved:oa_candidate":
            notes = _append_note(
                notes,
                "Candidate access URL identified locally; configure RESEARCH_MCP_FULLTEXT_RETRIEVAL_CMD for actual download.",
            )
        elif retrieval_status == "not_retrieved:needs_provider":
            notes = _append_note(
                notes,
                "Locator metadata exists, but no OA candidate was found locally.",
            )
        elif retrieval_status == "not_retrieved:missing_locator":
            notes = _append_note(
                notes,
                "No DOI, URL, or local full-text path was found for this record.",
            )
    elif retrieval_status == "not_retrieved:path_missing":
        notes = _append_note(notes, "Existing manifest pointed to a missing full-text path.")

    return {
        "record_id": record_id,
        "citekey": citekey,
        "doi": doi,
        "retrieval_status": retrieval_status,
        "version_label": version_label,
        "source_provider": source_provider,
        "retrieved_at": str(existing_row.get("retrieved_at") or "").strip() or retrieved_at,
        "fulltext_path": fulltext_path,
        "access_url": access_url,
        "license": str(existing_row.get("license") or "").strip(),
        "notes": notes,
    }


def _index_existing_manifest(rows: list[dict[str, str]]) -> dict[str, tuple[int, dict[str, str]]]:
    indexed: dict[str, tuple[int, dict[str, str]]] = {}
    for index, row in enumerate(rows):
        for key in _manifest_lookup_keys(row):
            indexed.setdefault(key, (index, row))
    return indexed


def _match_existing_row(
    record: dict[str, Any],
    existing_by_key: dict[str, tuple[int, dict[str, str]]],
) -> tuple[dict[str, str] | None, int | None]:
    for key in _record_lookup_keys(record):
        matched = existing_by_key.get(key)
        if matched:
            return matched[1], matched[0]
    return None, None


def _record_lookup_keys(record: dict[str, Any]) -> list[str]:
    keys: list[str] = []
    doi = str(record.get("doi") or "").strip().lower()
    citekey = str(record.get("citekey") or "").strip()
    record_id = str(record.get("record_id") or "").strip()
    match_key, _ = record_match_key(record)
    for key in (
        f"doi:{doi}" if doi else "",
        f"citekey:{citekey}" if citekey else "",
        f"record_id:{record_id}" if record_id else "",
        match_key,
    ):
        if key and key not in keys:
            keys.append(key)
    return keys


def _manifest_lookup_keys(row: dict[str, str]) -> list[str]:
    keys: list[str] = []
    doi = row.get("doi", "").strip().lower()
    citekey = row.get("citekey", "").strip()
    record_id = row.get("record_id", "").strip()
    for key in (
        f"doi:{doi}" if doi else "",
        f"citekey:{citekey}" if citekey else "",
        f"record_id:{record_id}" if record_id else "",
    ):
        if key and key not in keys:
            keys.append(key)
    return keys


def _resolve_retrieval_status(
    *,
    existing_status: str,
    resolved_path: str,
    access_url: str,
    doi: str,
    fallback_url: str,
) -> str:
    if resolved_path:
        if _looks_like_preprint(access_url or resolved_path):
            return "retrieved_preprint"
        return "retrieved_oa"

    if existing_status.startswith("retrieved"):
        return "not_retrieved:path_missing"
    if existing_status:
        return existing_status
    if access_url:
        return "not_retrieved:oa_candidate"
    if doi or fallback_url:
        return "not_retrieved:needs_provider"
    return "not_retrieved:missing_locator"


def _resolve_fulltext_path(project_root: Path, raw_path: str) -> str:
    candidate = raw_path.strip()
    if not candidate:
        return ""
    target = Path(candidate)
    if not target.is_absolute():
        target = project_root / target
    return candidate if target.exists() else ""


def _infer_version_label(access_url: str, record: dict[str, Any]) -> str:
    if _looks_like_preprint(access_url):
        return "submitted"
    venue = str(record.get("venue") or "").lower()
    if "preprint" in venue:
        return "submitted"
    return ""


def _looks_like_preprint(value: str) -> bool:
    lowered = value.lower()
    return any(token in lowered for token in ("arxiv.org", "biorxiv", "medrxiv", "preprint"))


def _build_screening_rows(manifest_rows: list[dict[str, str]]) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for row in manifest_rows:
        retrieval_status = row.get("retrieval_status", "")
        rows.append(
            {
                "record_id": row.get("record_id", ""),
                "decision": "",
                "exclusion_reason": "",
                "fulltext_status": retrieval_status,
                "notes": row.get("notes", ""),
            }
        )
    return rows


def _summarize_manifest(rows: list[dict[str, str]]) -> dict[str, int]:
    summary = {
        "total": len(rows),
        "resolved_locally": 0,
        "oa_candidates": 0,
        "unresolved": 0,
    }
    for row in rows:
        status = row.get("retrieval_status", "")
        if status in {"retrieved_oa", "retrieved_preprint"}:
            summary["resolved_locally"] += 1
        elif status == "not_retrieved:oa_candidate":
            summary["oa_candidates"] += 1
        else:
            summary["unresolved"] += 1
    return summary


def _artifact_bundle() -> dict[str, str]:
    return {
        "retrieval_manifest": "retrieval_manifest.csv",
        "screening_full_text": "screening/full_text.md",
    }


def _append_note(existing: str, addition: str) -> str:
    if not existing:
        return addition
    if addition in existing:
        return existing
    return f"{existing} {addition}".strip()


def _clean(value: Any) -> str:
    return " ".join(str(value or "").strip().split())


def _safe_int(value: Any) -> int | None:
    try:
        return int(str(value).strip())
    except (TypeError, ValueError):
        return None


def _status_sort_key(status: str) -> tuple[int, str]:
    if status in {"retrieved_oa", "retrieved_preprint"}:
        return (0, status)
    if status == "not_retrieved:oa_candidate":
        return (1, status)
    return (2, status)
