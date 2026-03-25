from __future__ import annotations

import csv
import io
import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable

from bridges.providers.literature_search import dedupe_search_results, normalize_search_hit, record_match_key


CitationFn = Callable[[str, int], dict[str, Any]]
ReferenceFn = Callable[[str, int], dict[str, Any]]
SearchFn = Callable[[str, int], dict[str, Any]]

DOI_RE = re.compile(r"(?:https?://(?:dx\.)?doi\.org/|doi:\s*)?(10\.\d{4,9}/[-._;()/:A-Z0-9]+)", re.I)
PAPER_ID_RE = re.compile(r"(?:paper[_ ]?id|semantic scholar id)\s*[:=]\s*([A-Za-z0-9:._-]+)", re.I)
MAX_NOTE_FILES = 8
MAX_SCAN_BYTES = 80_000
DEFAULT_SEED_LIMIT = 5
DEFAULT_GRAPH_LIMIT = 6


def run_citation_graph(
    task_packet: dict[str, Any],
    cwd: Path,
    *,
    search_fn: SearchFn,
    citations_fn: CitationFn,
    references_fn: ReferenceFn,
    retrieved_at: str | None = None,
) -> dict[str, Any]:
    timestamp = retrieved_at or datetime.now(timezone.utc).replace(microsecond=0).isoformat()
    project_root = _artifact_project_root(task_packet, cwd)
    seed_limit = _resolve_limit(task_packet, "seed_limit", DEFAULT_SEED_LIMIT, upper=10)
    graph_limit = _resolve_limit(task_packet, "graph_limit", DEFAULT_GRAPH_LIMIT, upper=20)

    seed_candidates = extract_seed_candidates(task_packet, project_root)
    resolved_seeds, resolution_failures = resolve_seed_candidates(seed_candidates, search_fn, limit=seed_limit)

    if not resolved_seeds:
        return {
            "status": "warning",
            "summary": "No citation seeds could be resolved from task packet or local literature artifacts.",
            "provenance": ["https://api.semanticscholar.org/graph/v1"],
            "data": {
                "provider_mode": "builtin_semantic_scholar_graph_baseline",
                "seed_candidates": seed_candidates,
                "resolved_seeds": [],
                "resolution_failures": resolution_failures,
                "search_results": [],
                "dedup_log": [],
                "snowball_log": [],
                "artifact_bundle": _artifact_bundle(),
            },
        }

    existing_records = _read_existing_search_results(project_root)
    existing_index = {record_match_key(record)[0]: record for record in existing_records}

    raw_candidates: list[dict[str, Any]] = []
    dedup_log: list[dict[str, str]] = []
    snowball_log: list[dict[str, str]] = []
    graph_failures: list[dict[str, str]] = []

    for seed in resolved_seeds:
        seed_identifier = seed["seed_id"]
        for direction, fetch_fn in (("forward", citations_fn), ("backward", references_fn)):
            response = fetch_fn(seed_identifier, graph_limit)
            error = str(response.get("error", "")).strip()
            data = response.get("data", [])
            if not isinstance(data, list):
                data = []

            if error:
                graph_failures.append(
                    {
                        "seed_id": seed_identifier,
                        "seed_label": seed["label"],
                        "direction": direction,
                        "error": error,
                    }
                )
                snowball_log.append(
                    {
                        "seed_record_id": seed["seed_record_id"],
                        "seed_label": seed["label"],
                        "direction": direction,
                        "candidate_record_id": "",
                        "candidate_title": "",
                        "decision": "provider_error",
                        "reason": error,
                    }
                )
                continue

            for ordinal, item in enumerate(data, start=1):
                paper = _unwrap_graph_entry(item, direction)
                if not paper:
                    continue
                normalized = normalize_search_hit(
                    paper,
                    query_id=f"{seed['query_seed_id']}-{direction}",
                    query_text=f"{direction} snowball from {seed['label']}",
                    retrieved_at=timestamp,
                    ordinal=ordinal,
                    source="semantic_scholar_graph",
                )
                normalized["seed_record_id"] = seed["seed_record_id"]
                normalized["seed_label"] = seed["label"]
                normalized["direction"] = direction

                existing_match_key, existing_basis = record_match_key(normalized)
                existing = existing_index.get(existing_match_key)
                if existing is not None:
                    dedup_log.append(
                        {
                            "candidate_record_id": str(normalized.get("record_id", "")),
                            "canonical_record_id": str(existing.get("record_id", "")),
                            "decision": "drop_duplicate",
                            "match_basis": existing_basis,
                            "resolver": "builtin_citation_graph",
                            "notes": f"{direction} snowball candidate already exists in corpus.",
                        }
                    )
                    snowball_log.append(
                        {
                            "seed_record_id": seed["seed_record_id"],
                            "seed_label": seed["label"],
                            "direction": direction,
                            "candidate_record_id": str(normalized.get("record_id", "")),
                            "candidate_title": str(normalized.get("title", "")),
                            "decision": "drop_duplicate",
                            "reason": f"already in corpus via {existing_basis}",
                        }
                    )
                    continue

                raw_candidates.append(normalized)

    unique_candidates, internal_dedup = dedupe_search_results(raw_candidates)
    dedup_log.extend(
        {
            **entry,
            "decision": "merge_duplicate",
            "resolver": "builtin_citation_graph",
        }
        for entry in internal_dedup
    )
    internal_duplicate_ids = {entry["candidate_record_id"] for entry in internal_dedup}

    for candidate in raw_candidates:
        record_id = str(candidate.get("record_id", ""))
        if record_id in internal_duplicate_ids:
            reason = "duplicate of another snowball candidate"
            decision = "merge_duplicate"
        else:
            reason = "new candidate from snowball expansion"
            decision = "new_candidate"
        snowball_log.append(
            {
                "seed_record_id": str(candidate.get("seed_record_id", "")),
                "seed_label": str(candidate.get("seed_label", "")),
                "direction": str(candidate.get("direction", "")),
                "candidate_record_id": record_id,
                "candidate_title": str(candidate.get("title", "")),
                "decision": decision,
                "reason": reason,
            }
        )

    if unique_candidates:
        status = "warning" if graph_failures or resolution_failures else "ok"
        summary = (
            f"Expanded {len(resolved_seeds)} citation seeds into {len(unique_candidates)} new candidates "
            f"({len(raw_candidates)} raw candidates, {len(dedup_log)} dedup decisions)."
        )
    elif graph_failures:
        status = "error"
        summary = (
            f"Citation graph failed for all resolved seeds; last error: {graph_failures[-1]['error']}"
        )
    else:
        status = "warning"
        summary = f"Resolved {len(resolved_seeds)} seeds but found no new citation candidates."

    return {
        "status": status,
        "summary": summary,
        "provenance": ["https://api.semanticscholar.org/graph/v1"],
        "data": {
            "provider_mode": "builtin_semantic_scholar_graph_baseline",
            "project_root": str(project_root),
            "seed_candidates": seed_candidates,
            "resolved_seeds": resolved_seeds,
            "resolution_failures": resolution_failures,
            "graph_failures": graph_failures,
            "search_results": unique_candidates,
            "dedup_log": dedup_log,
            "snowball_log": snowball_log,
            "artifact_bundle": _artifact_bundle(),
        },
    }


def extract_seed_candidates(task_packet: dict[str, Any], project_root: Path) -> list[dict[str, str]]:
    candidates: list[dict[str, str]] = []
    seen: set[tuple[str, str]] = set()

    def add_candidate(identifier_type: str, value: str, source: str, label: str = "") -> None:
        cleaned = " ".join(str(value).strip().split())
        if not cleaned:
            return
        normalized = _normalize_candidate(identifier_type, cleaned)
        key = (identifier_type, normalized.casefold())
        if key in seen:
            return
        seen.add(key)
        candidates.append(
            {
                "seed_type": identifier_type,
                "seed_value": normalized,
                "source": source,
                "label": label or normalized,
            }
        )

    direct_fields = {
        "target_paper_id": "paper_id",
        "paper_id": "paper_id",
        "target_doi": "doi",
        "doi": "doi",
        "target_title": "title",
        "title": "title",
    }
    for field_name, identifier_type in direct_fields.items():
        value = task_packet.get(field_name)
        if isinstance(value, str) and value.strip():
            add_candidate(identifier_type, value, f"task_packet:{field_name}", value)

    for row in _read_existing_search_results(project_root)[:12]:
        if row.get("paper_id"):
            add_candidate("paper_id", row["paper_id"], "search_results.csv", row.get("title", ""))
        elif row.get("doi"):
            add_candidate("doi", row["doi"], "search_results.csv", row.get("title", ""))
        elif row.get("title"):
            add_candidate("title", row["title"], "search_results.csv", row.get("title", ""))

    bib_path = project_root / "bibliography.bib"
    if bib_path.exists():
        for entry in _read_bibliography_entries(bib_path)[:12]:
            if entry.get("doi"):
                add_candidate("doi", entry["doi"], "bibliography.bib", entry.get("title", ""))
            elif entry.get("title"):
                add_candidate("title", entry["title"], "bibliography.bib", entry.get("title", ""))

    notes_dir = project_root / "notes"
    if notes_dir.exists():
        for path in sorted(notes_dir.glob("*.md"))[:MAX_NOTE_FILES]:
            try:
                text = path.read_text(encoding="utf-8")[:MAX_SCAN_BYTES]
            except UnicodeDecodeError:
                text = path.read_text(encoding="utf-8", errors="ignore")[:MAX_SCAN_BYTES]
            except OSError:
                continue
            paper_id_match = PAPER_ID_RE.search(text)
            if paper_id_match:
                add_candidate("paper_id", paper_id_match.group(1), str(path), path.stem)
            doi_match = DOI_RE.search(text)
            if doi_match:
                add_candidate("doi", doi_match.group(1), str(path), path.stem)

    return candidates


def resolve_seed_candidates(
    candidates: list[dict[str, str]],
    search_fn: SearchFn,
    *,
    limit: int,
) -> tuple[list[dict[str, str]], list[dict[str, str]]]:
    resolved: list[dict[str, str]] = []
    failures: list[dict[str, str]] = []
    seen: set[str] = set()

    for index, candidate in enumerate(candidates, start=1):
        if len(resolved) >= limit:
            break

        seed_type = candidate["seed_type"]
        seed_value = candidate["seed_value"]
        seed_id: str | None = None

        if seed_type == "paper_id":
            seed_id = seed_value
        elif seed_type == "doi":
            seed_id = f"DOI:{seed_value}"
        elif seed_type == "title":
            response = search_fn(seed_value, 3)
            error = str(response.get("error", "")).strip()
            if error:
                failures.append(
                    {
                        "seed_value": seed_value,
                        "source": candidate["source"],
                        "error": error,
                    }
                )
                continue
            hits = response.get("data", [])
            if not isinstance(hits, list) or not hits:
                failures.append(
                    {
                        "seed_value": seed_value,
                        "source": candidate["source"],
                        "error": "title seed could not be resolved",
                    }
                )
                continue
            best_hit = _pick_best_title_hit(seed_value, hits)
            seed_id = str(best_hit.get("paperId", "")).strip()

        if not seed_id or seed_id in seen:
            continue
        seen.add(seed_id)
        resolved.append(
            {
                "seed_id": seed_id,
                "seed_record_id": f"seed:{index}",
                "query_seed_id": f"seed{index}",
                "seed_type": seed_type,
                "seed_value": seed_value,
                "label": candidate.get("label") or seed_value,
                "source": candidate["source"],
            }
        )

    return resolved, failures


def _artifact_project_root(task_packet: dict[str, Any], cwd: Path) -> Path:
    topic = str(task_packet.get("topic", "")).strip()
    artifact_root = str(task_packet.get("artifact_root", "RESEARCH/[topic]/"))
    return cwd / artifact_root.replace("[topic]", topic)


def _read_existing_search_results(project_root: Path) -> list[dict[str, str]]:
    path = project_root / "search_results.csv"
    if not path.exists():
        return []
    try:
        content = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        content = path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return []

    rows: list[dict[str, str]] = []
    reader = csv.DictReader(io.StringIO(content))
    for row in reader:
        normalized = {str(key).strip(): " ".join(str(value or "").split()) for key, value in row.items()}
        rows.append(normalized)
    return rows


def _read_bibliography_entries(path: Path) -> list[dict[str, str]]:
    try:
        content = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        content = path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return []

    entries: list[dict[str, str]] = []
    for block in content.split("@"):
        if "{" not in block:
            continue
        _, remainder = block.split("{", 1)
        citekey, _, body = remainder.partition(",")
        doi_match = re.search(r"\bdoi\s*=\s*[{\"]([^}\"]+)", body, re.I)
        title_match = re.search(r"\btitle\s*=\s*[{\"]([^}\"]+)", body, re.I)
        entries.append(
            {
                "citekey": citekey.strip(),
                "doi": _normalize_candidate("doi", doi_match.group(1)) if doi_match else "",
                "title": " ".join(title_match.group(1).split()) if title_match else "",
            }
        )
    return entries


def _unwrap_graph_entry(item: Any, direction: str) -> dict[str, Any] | None:
    if not isinstance(item, dict):
        return None
    nested_key = "citingPaper" if direction == "forward" else "citedPaper"
    nested = item.get(nested_key)
    if isinstance(nested, dict):
        return nested
    return item


def _pick_best_title_hit(title: str, hits: list[dict[str, Any]]) -> dict[str, Any]:
    normalized_title = _normalize_title(title)
    for hit in hits:
        if not isinstance(hit, dict):
            continue
        if _normalize_title(hit.get("title", "")) == normalized_title:
            return hit
    for hit in hits:
        if isinstance(hit, dict):
            return hit
    return {}


def _normalize_candidate(identifier_type: str, value: str) -> str:
    if identifier_type == "doi":
        match = DOI_RE.search(value)
        if match:
            return match.group(1).rstrip(".,);]").lower()
        return value.lower()
    return value.strip()


def _normalize_title(value: Any) -> str:
    return re.sub(r"[^a-z0-9]+", "", " ".join(str(value or "").lower().split()))


def _artifact_bundle() -> dict[str, str]:
    return {
        "snowball_log": "snowball_log.md",
        "search_results": "search_results.csv",
        "dedup_log": "dedup_log.csv",
    }


def _resolve_limit(task_packet: dict[str, Any], key: str, default: int, *, upper: int) -> int:
    value = task_packet.get(key)
    try:
        parsed = int(str(value).strip())
    except (TypeError, ValueError):
        return default
    return max(1, min(parsed, upper))

