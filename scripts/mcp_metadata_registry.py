#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any


DOI_RE = re.compile(r"(?:https?://(?:dx\.)?doi\.org/|doi:\s*)?(10\.\d{4,9}/[-._;()/:A-Z0-9]+)", re.I)
ARXIV_RE = re.compile(r"(?:arXiv:\s*)?(\d{4}\.\d{4,5}(?:v\d+)?)", re.I)
PMID_RE = re.compile(r"(?:PMID:\s*)?(\d{6,9})", re.I)
OPENALEX_RE = re.compile(r"\b(W\d{6,})\b", re.I)

TEXT_FILE_SUFFIXES = {".md", ".txt", ".bib", ".json", ".yaml", ".yml", ".csv"}
MAX_SCAN_FILES = 12
MAX_SCAN_BYTES = 150_000


def _artifact_project_root(task_packet: dict[str, Any], cwd: Path) -> Path:
    topic = str(task_packet.get("topic", "")).strip()
    artifact_root = str(task_packet.get("artifact_root", "RESEARCH/[topic]/"))
    return cwd / artifact_root.replace("[topic]", topic)


def _read_candidate_files(project_root: Path, required_outputs: list[str]) -> list[tuple[str, str]]:
    candidates: list[Path] = []
    for rel_path in required_outputs:
        target = project_root / rel_path
        if target.is_file() and target.suffix.lower() in TEXT_FILE_SUFFIXES:
            candidates.append(target)

    if project_root.exists():
        fallback_names = ("bibliography.bib", "search_log.md", "search_strategy.md")
        for filename in fallback_names:
            target = project_root / filename
            if target.is_file() and target not in candidates:
                candidates.append(target)

    texts: list[tuple[str, str]] = []
    for path in candidates[:MAX_SCAN_FILES]:
        try:
            text = path.read_text(encoding="utf-8")[:MAX_SCAN_BYTES]
        except UnicodeDecodeError:
            text = path.read_text(encoding="utf-8", errors="ignore")[:MAX_SCAN_BYTES]
        except OSError:
            continue
        texts.append((str(path), text))
    return texts


def _normalize_doi(raw: str) -> str:
    cleaned = raw.strip().rstrip(".,);]")
    match = DOI_RE.search(cleaned)
    if not match:
        return cleaned.lower()
    return match.group(1).lower()


def _normalize_arxiv(raw: str) -> str:
    cleaned = raw.strip()
    match = ARXIV_RE.search(cleaned)
    if not match:
        return cleaned
    return match.group(1).lower()


def _normalize_pmid(raw: str) -> str:
    cleaned = raw.strip()
    match = PMID_RE.search(cleaned)
    if not match:
        return cleaned
    return match.group(1)


def _extract_identifiers(source_name: str, text: str) -> list[dict[str, str]]:
    records: list[dict[str, str]] = []
    for match in DOI_RE.finditer(text):
        raw_value = match.group(0)
        records.append(
            {
                "type": "doi",
                "raw": raw_value,
                "normalized": _normalize_doi(raw_value),
                "source": source_name,
            }
        )
    for match in ARXIV_RE.finditer(text):
        raw_value = match.group(0)
        records.append(
            {
                "type": "arxiv",
                "raw": raw_value,
                "normalized": _normalize_arxiv(raw_value),
                "source": source_name,
            }
        )
    for match in PMID_RE.finditer(text):
        raw_value = match.group(0)
        records.append(
            {
                "type": "pmid",
                "raw": raw_value,
                "normalized": _normalize_pmid(raw_value),
                "source": source_name,
            }
        )
    for match in OPENALEX_RE.finditer(text):
        raw_value = match.group(1)
        records.append(
            {
                "type": "openalex",
                "raw": raw_value,
                "normalized": raw_value.upper(),
                "source": source_name,
            }
        )
    return records


def _dedupe_records(records: list[dict[str, str]]) -> list[dict[str, str]]:
    deduped: list[dict[str, str]] = []
    seen: set[tuple[str, str]] = set()
    for record in records:
        key = (record["type"], record["normalized"])
        if key in seen:
            continue
        seen.add(key)
        deduped.append(record)
    return deduped


def _extract_context_hints(task_packet: dict[str, Any]) -> dict[str, Any]:
    hints: dict[str, Any] = {}
    for key in ("topic", "paper_type", "task_id", "target_doi", "doi", "title", "target_title"):
        value = task_packet.get(key)
        if isinstance(value, str) and value.strip():
            hints[key] = value.strip()
    return hints


def main() -> None:
    try:
        raw = sys.stdin.read()
        if not raw.strip():
            print(json.dumps({"status": "error", "summary": "No input provided", "data": {}}))
            return

        payload = json.loads(raw)
        task_packet = payload.get("task_packet", {})
        if not isinstance(task_packet, dict):
            task_packet = {}
        cwd = Path.cwd()
        project_root = _artifact_project_root(task_packet, cwd)
        required_outputs = [
            str(item) for item in task_packet.get("required_outputs", []) if str(item).strip()
        ]

        source_texts: list[tuple[str, str]] = [("task_packet", json.dumps(task_packet, ensure_ascii=False))]
        source_texts.extend(_read_candidate_files(project_root, required_outputs))

        records: list[dict[str, str]] = []
        provenance: list[str] = []
        for source_name, text in source_texts:
            if source_name != "task_packet":
                provenance.append(source_name)
            records.extend(_extract_identifiers(source_name, text))

        deduped = _dedupe_records(records)
        context_hints = _extract_context_hints(task_packet)

        if deduped:
            summary = (
                f"Builtin metadata reference normalized {len(deduped)} unique identifiers "
                f"from task packet and local artifacts."
            )
            status = "ok"
        else:
            summary = (
                "Builtin metadata reference provider is available, but no DOI/arXiv/PMID/OpenAlex "
                "identifiers were found in the task packet or local artifacts."
            )
            status = "warning"

        print(
            json.dumps(
                {
                    "status": status,
                    "summary": summary,
                    "provenance": provenance[:6],
                    "data": {
                        "provider_mode": "builtin_local_reference",
                        "project_root": str(project_root),
                        "scanned_sources": [name for name, _ in source_texts],
                        "identifier_count": len(deduped),
                        "identifiers": deduped[:50],
                        "context_hints": context_hints,
                    },
                },
                ensure_ascii=False,
            )
        )
    except Exception as exc:
        print(
            json.dumps(
                {
                    "status": "error",
                    "summary": f"Metadata registry provider exception: {exc}",
                    "data": {"error": str(exc)},
                },
                ensure_ascii=False,
            )
        )


if __name__ == "__main__":
    main()
