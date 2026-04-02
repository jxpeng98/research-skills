from __future__ import annotations

import csv
import io
import json
import re
from pathlib import Path
from typing import Any

from bridges.providers.overlay_runtime import invoke_overlay_json


DOI_RE = re.compile(r"(?:https?://(?:dx\.)?doi\.org/|doi:\s*)?(10\.\d{4,9}/[-._;()/:A-Z0-9]+)", re.I)
ARXIV_RE = re.compile(r"(?:arXiv:\s*)?(\d{4}\.\d{4,5}(?:v\d+)?)", re.I)
PMID_RE = re.compile(r"(?:PMID:\s*)?(\d{6,9})", re.I)
OPENALEX_RE = re.compile(r"\b(W\d{6,})\b", re.I)
TEXT_FILE_SUFFIXES = {".md", ".txt", ".bib", ".json", ".yaml", ".yml", ".csv", ".ris"}
MAX_SCAN_FILES = 16
MAX_SCAN_BYTES = 180_000
NOTE_FIELD_RE = re.compile(r"^\|\s*\*?\*?([^|*]+?)\*?\*?\s*\|\s*(.*?)\s*\|$", re.M)
NON_ALNUM_RE = re.compile(r"[^a-z0-9]+")
TITLE_WORD_RE = re.compile(r"[A-Za-z0-9][A-Za-z0-9-]{2,}")
STOPWORDS = {
    "a",
    "an",
    "and",
    "for",
    "from",
    "in",
    "of",
    "on",
    "the",
    "to",
    "with",
}
METADATA_ENRICH_ENV = "RESEARCH_MCP_METADATA_REGISTRY_ENRICH_CMD"
SOURCE_PROVIDER_PRIORITIES = {
    "openalex": 90,
    "crossref": 80,
    "csl_json": 70,
    "ris": 65,
    "bibtex": 60,
    "paper_note": 50,
    "semantic_scholar": 45,
    "search_results": 40,
    "retrieval_manifest": 30,
    "external_enrichment": 35,
}
FIELD_PROVIDER_PRIORITIES = {
    "title": {"openalex": 95, "crossref": 82},
    "authors": {"openalex": 95, "crossref": 78},
    "year": {"openalex": 92, "crossref": 82},
    "venue": {"openalex": 95, "crossref": 82},
    "url": {"crossref": 88, "openalex": 72},
    "publisher": {"crossref": 94, "openalex": 74},
    "volume": {"crossref": 94},
    "issue": {"crossref": 94},
    "pages": {"crossref": 94},
    "oa_url": {"openalex": 96},
    "openalex_id": {"openalex": 99},
}
FIELD_POLICY_VERSION = "openalex_core_crossref_structural_v1"
FILL_ONLY_FIELDS = {"doi", "citekey", "openalex_id"}
PROVENANCE_TRACKED_FIELDS = {
    "title",
    "authors",
    "year",
    "venue",
    "doi",
    "url",
    "abstract",
    "citekey",
    "publisher",
    "volume",
    "issue",
    "pages",
    "oa_url",
    "openalex_id",
}


def artifact_project_root(task_packet: dict[str, Any], cwd: Path) -> Path:
    topic = str(task_packet.get("topic", "")).strip()
    artifact_root = str(task_packet.get("artifact_root", "RESEARCH/[topic]/"))
    return cwd / artifact_root.replace("[topic]", topic)


def read_candidate_files(project_root: Path, required_outputs: list[str]) -> list[Path]:
    candidates: list[Path] = []

    def add_path(target: Path) -> None:
        if not target.is_file():
            return
        if target.suffix.lower() not in TEXT_FILE_SUFFIXES:
            return
        if target not in candidates:
            candidates.append(target)

    for rel_path in required_outputs:
        add_path(project_root / rel_path)

    for filename in (
        "bibliography.bib",
        "references.json",
        "references.ris",
        "search_results.csv",
        "retrieval_manifest.csv",
        "search_log.md",
        "search_strategy.md",
    ):
        add_path(project_root / filename)

    notes_dir = project_root / "notes"
    if notes_dir.exists():
        for path in sorted(notes_dir.glob("*.md"))[:MAX_SCAN_FILES]:
            add_path(path)

    return candidates[:MAX_SCAN_FILES]


def read_source_texts(project_root: Path, required_outputs: list[str]) -> list[tuple[str, str]]:
    texts: list[tuple[str, str]] = []
    for path in read_candidate_files(project_root, required_outputs):
        try:
            text = path.read_text(encoding="utf-8")[:MAX_SCAN_BYTES]
        except UnicodeDecodeError:
            text = path.read_text(encoding="utf-8", errors="ignore")[:MAX_SCAN_BYTES]
        except OSError:
            continue
        texts.append((str(path), text))
    return texts


def normalize_doi(raw: str) -> str:
    cleaned = raw.strip().rstrip(".,);]")
    match = DOI_RE.search(cleaned)
    if not match:
        return cleaned.lower()
    return match.group(1).lower()


def normalize_arxiv(raw: str) -> str:
    cleaned = raw.strip()
    match = ARXIV_RE.search(cleaned)
    if not match:
        return cleaned
    return match.group(1).lower()


def normalize_pmid(raw: str) -> str:
    cleaned = raw.strip()
    match = PMID_RE.search(cleaned)
    if not match:
        return cleaned
    return match.group(1)


def extract_identifiers(source_name: str, text: str) -> list[dict[str, str]]:
    records: list[dict[str, str]] = []
    for match in DOI_RE.finditer(text):
        raw_value = match.group(0)
        records.append(
            {
                "type": "doi",
                "raw": raw_value,
                "normalized": normalize_doi(raw_value),
                "source": source_name,
            }
        )
    for match in ARXIV_RE.finditer(text):
        raw_value = match.group(0)
        records.append(
            {
                "type": "arxiv",
                "raw": raw_value,
                "normalized": normalize_arxiv(raw_value),
                "source": source_name,
            }
        )
    for match in PMID_RE.finditer(text):
        raw_value = match.group(0)
        records.append(
            {
                "type": "pmid",
                "raw": raw_value,
                "normalized": normalize_pmid(raw_value),
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


def dedupe_identifiers(records: list[dict[str, str]]) -> list[dict[str, str]]:
    deduped: list[dict[str, str]] = []
    seen: set[tuple[str, str]] = set()
    for record in records:
        key = (record["type"], record["normalized"])
        if key in seen:
            continue
        seen.add(key)
        deduped.append(record)
    return deduped


def extract_context_hints(task_packet: dict[str, Any]) -> dict[str, Any]:
    hints: dict[str, Any] = {}
    for key in (
        "topic",
        "paper_type",
        "task_id",
        "target_doi",
        "doi",
        "title",
        "target_title",
        "reference_style",
    ):
        value = task_packet.get(key)
        if isinstance(value, str) and value.strip():
            hints[key] = value.strip()
    return hints


def collect_reference_records(project_root: Path, required_outputs: list[str]) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for path in read_candidate_files(project_root, required_outputs):
        suffix = path.suffix.lower()
        try:
            text = path.read_text(encoding="utf-8")[:MAX_SCAN_BYTES]
        except UnicodeDecodeError:
            text = path.read_text(encoding="utf-8", errors="ignore")[:MAX_SCAN_BYTES]
        except OSError:
            continue

        if suffix == ".bib":
            records.extend(_parse_bibtex_records(text, str(path)))
        elif suffix == ".json" and path.name == "references.json":
            records.extend(_parse_csl_json_records(text, str(path)))
        elif suffix == ".ris":
            records.extend(_parse_ris_records(text, str(path)))
        elif suffix == ".csv" and path.name == "search_results.csv":
            records.extend(_parse_search_results(text, str(path)))
        elif suffix == ".csv" and path.name == "retrieval_manifest.csv":
            records.extend(_parse_retrieval_manifest(text, str(path)))
        elif suffix == ".md" and path.parent.name == "notes":
            note_record = _parse_note_record(text, str(path))
            if note_record:
                records.append(note_record)
    return records


def merge_reference_records(records: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], list[dict[str, str]]]:
    canonical_records: list[dict[str, Any]] = []
    canonical_by_key: dict[str, dict[str, Any]] = {}
    dedup_log: list[dict[str, str]] = []

    for record in records:
        merge_key, match_basis = _record_merge_key(record)
        existing = canonical_by_key.get(merge_key)
        if existing is None:
            new_record = dict(record)
            new_record["source_paths"] = sorted(set(record.get("source_paths", [])))
            canonical_records.append(new_record)
            canonical_by_key[merge_key] = new_record
            continue

        _merge_record(existing, record)
        dedup_log.append(
            {
                "candidate_record_id": str(record.get("record_id", "")),
                "canonical_record_id": str(existing.get("record_id", "")),
                "decision": "merge_duplicate",
                "match_basis": match_basis,
                "resolver": "builtin_metadata_registry",
                "notes": f"Merged metadata from {record.get('source_format', 'unknown')} into canonical reference.",
            }
        )

    assign_citekeys(canonical_records)
    return canonical_records, dedup_log


def assign_citekeys(records: list[dict[str, Any]]) -> None:
    used: set[str] = set()
    for record in records:
        base = record.get("citekey") or _generate_citekey(record)
        citekey = base
        suffix_ord = 0
        while citekey in used:
            suffix_ord += 1
            suffix = chr(ord("a") + suffix_ord)
            citekey = f"{base}{suffix}"
        used.add(citekey)
        record["citekey"] = citekey


def summarize_reference_state(project_root: Path, records: list[dict[str, Any]]) -> dict[str, Any]:
    source_formats = sorted({str(record.get("source_format", "")) for record in records if record.get("source_format")})
    has_bib = (project_root / "bibliography.bib").exists()
    has_json = (project_root / "references.json").exists()
    has_ris = (project_root / "references.ris").exists()
    preferred_input = "bibliography.bib" if has_bib else "references.json" if has_json else "references.ris" if has_ris else "notes/search_results"
    return {
        "source_formats": source_formats,
        "preferred_input_mode": preferred_input,
        "canonical_export": "bibliography.bib",
        "alt_exports": ["references.json", "references.ris"],
        "supports_non_bib_workflows": True,
    }


def apply_external_enrichment(
    records: list[dict[str, Any]],
    *,
    cwd: Path,
    context_hints: dict[str, Any],
    timeout_seconds: int = 20,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    if not records:
        return records, {"configured": False}

    payload = {
        "provider": "metadata-registry",
        "mode": "enrich",
        "records": records,
        "context_hints": context_hints,
    }
    parsed, overlay_info = invoke_overlay_json(
        env_name=METADATA_ENRICH_ENV,
        payload=payload,
        cwd=cwd,
        timeout_seconds=timeout_seconds,
        label="External enrichment",
    )
    if parsed is None:
        return records, overlay_info

    merged_records, enrichment_info = merge_external_enrichment_payload(records, parsed)
    return merged_records, {
        **overlay_info,
        **enrichment_info,
    }


def merge_external_enrichment_payload(
    records: list[dict[str, Any]],
    payload: dict[str, Any],
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    data = payload.get("data", {})
    if not isinstance(data, dict):
        data = {}
    raw_records = data.get("records", [])
    if not isinstance(raw_records, list):
        raw_records = []
    external_records = [
        _normalize_external_record(item, index)
        for index, item in enumerate(raw_records, start=1)
        if isinstance(item, dict)
    ]
    if not external_records:
        return records, {
            "status": "warning",
            "summary": str(payload.get("summary", "External enrichment returned no records.")),
            "provenance": _normalize_provenance(payload.get("provenance")),
        }

    merged_records, applied_count, merge_trace = _merge_external_records(records, external_records)
    status = str(payload.get("status", "ok")).strip().lower() or "ok"
    if status not in {"ok", "warning", "error"}:
        status = "warning"
    return merged_records, {
        "status": status,
        "summary": str(payload.get("summary", f"External enrichment merged {applied_count} records.")).strip(),
        "provenance": _normalize_provenance(payload.get("provenance")),
        "record_count": len(external_records),
        "applied_count": applied_count,
        "field_policy_version": FIELD_POLICY_VERSION,
        "merge_trace": merge_trace[:100],
    }


def _parse_search_results(text: str, source_name: str) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    reader = csv.DictReader(io.StringIO(text))
    for index, row in enumerate(reader, start=1):
        title = _clean_string(row.get("title"))
        if not title:
            continue
        rows.append(
            _base_record(
                record_id=_clean_string(row.get("record_id")) or f"search:{index}",
                source_format="search_results.csv",
                source_name=source_name,
                title=title,
                authors=_normalize_authors(_clean_string(row.get("authors"))),
                year=_safe_int(row.get("year")),
                venue=_clean_string(row.get("venue")),
                doi=normalize_doi(_clean_string(row.get("doi"))) if _clean_string(row.get("doi")) else "",
                url=_clean_string(row.get("url")),
                abstract=_clean_string(row.get("abstract")),
            )
        )
    return rows


def _parse_retrieval_manifest(text: str, source_name: str) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    reader = csv.DictReader(io.StringIO(text))
    for index, row in enumerate(reader, start=1):
        doi_raw = _clean_string(row.get("doi"))
        citekey = _clean_string(row.get("citekey"))
        if not doi_raw and not citekey:
            continue
        rows.append(
            _base_record(
                record_id=_clean_string(row.get("record_id")) or f"retrieval:{index}",
                source_format="retrieval_manifest.csv",
                source_name=source_name,
                title="",
                authors=[],
                year=None,
                venue="",
                doi=normalize_doi(doi_raw) if doi_raw else "",
                url=_clean_string(row.get("access_url")),
                abstract="",
                citekey=citekey,
            )
        )
    return rows


def _parse_note_record(text: str, source_name: str) -> dict[str, Any] | None:
    fields: dict[str, str] = {}
    for field, value in NOTE_FIELD_RE.findall(text):
        normalized_field = _clean_string(field).lower()
        fields[normalized_field] = _clean_string(value)

    title_match = re.search(r"^#\s+(.+)$", text, re.M)
    title = _clean_string(title_match.group(1)) if title_match else ""
    if not title and not fields:
        return None

    authors = _normalize_authors(fields.get("authors", ""))
    year = _safe_int(fields.get("year"))
    venue = fields.get("venue", "")
    doi = normalize_doi(fields["doi"]) if fields.get("doi") else ""
    url = fields.get("url", "")
    return _base_record(
        record_id=Path(source_name).stem,
        source_format="paper_note",
        source_name=source_name,
        title=title,
        authors=authors,
        year=year,
        venue=venue,
        doi=doi,
        url=url,
        abstract="",
    )


def _parse_bibtex_records(text: str, source_name: str) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for index, block in enumerate(text.split("@"), start=1):
        if "{" not in block:
            continue
        _, remainder = block.split("{", 1)
        citekey, _, body = remainder.partition(",")
        title = _extract_bib_field(body, "title")
        doi = _extract_bib_field(body, "doi")
        if not title and not doi:
            continue
        author_field = _extract_bib_field(body, "author")
        records.append(
            _base_record(
                record_id=f"bib:{citekey.strip() or index}",
                source_format="bibtex",
                source_name=source_name,
                title=title,
                authors=_normalize_authors(author_field),
                year=_safe_int(_extract_bib_field(body, "year")),
                venue=_extract_bib_field(body, "journal") or _extract_bib_field(body, "booktitle"),
                doi=normalize_doi(doi) if doi else "",
                url=_extract_bib_field(body, "url"),
                abstract=_extract_bib_field(body, "abstract"),
                citekey=citekey.strip(),
            )
        )
    return records


def _parse_csl_json_records(text: str, source_name: str) -> list[dict[str, Any]]:
    try:
        payload = json.loads(text)
    except json.JSONDecodeError:
        return []
    items: list[dict[str, Any]]
    if isinstance(payload, list):
        items = [item for item in payload if isinstance(item, dict)]
    elif isinstance(payload, dict):
        items = [payload]
    else:
        return []

    records: list[dict[str, Any]] = []
    for index, item in enumerate(items, start=1):
        author_tokens: list[str] = []
        for author in item.get("author", []):
            if not isinstance(author, dict):
                continue
            family = _clean_string(author.get("family"))
            given = _clean_string(author.get("given"))
            display = ", ".join(part for part in (family, given) if part)
            if display:
                author_tokens.append(display)
        year = None
        issued = item.get("issued")
        if isinstance(issued, dict):
            date_parts = issued.get("date-parts")
            if isinstance(date_parts, list) and date_parts and isinstance(date_parts[0], list) and date_parts[0]:
                year = _safe_int(date_parts[0][0])
        records.append(
            _base_record(
                record_id=_clean_string(item.get("id")) or f"csl:{index}",
                source_format="csl_json",
                source_name=source_name,
                title=_clean_string(item.get("title")),
                authors=_normalize_authors("; ".join(author_tokens)),
                year=year,
                venue=_clean_string(item.get("container-title")),
                doi=normalize_doi(_clean_string(item.get("DOI"))) if _clean_string(item.get("DOI")) else "",
                url=_clean_string(item.get("URL")),
                abstract=_clean_string(item.get("abstract")),
                citekey="",
            )
        )
    return records


def _parse_ris_records(text: str, source_name: str) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    chunks = re.split(r"\nER\s+-\s*\n", text.strip(), flags=re.I)
    for index, chunk in enumerate(chunks, start=1):
        lines = [line.rstrip() for line in chunk.splitlines() if line.strip()]
        if not lines:
            continue
        fields: dict[str, list[str]] = {}
        for line in lines:
            if "  - " not in line:
                continue
            tag, value = line.split("  - ", 1)
            fields.setdefault(tag.strip().upper(), []).append(_clean_string(value))
        title = _first_nonempty(fields, "TI", "T1")
        if not title:
            continue
        authors = fields.get("AU", []) or fields.get("A1", [])
        year = _safe_int(_first_nonempty(fields, "PY", "Y1"))
        venue = _first_nonempty(fields, "JO", "JF", "T2")
        doi = normalize_doi(_first_nonempty(fields, "DO")) if _first_nonempty(fields, "DO") else ""
        url = _first_nonempty(fields, "UR")
        abstract = _first_nonempty(fields, "AB")
        citekey = _first_nonempty(fields, "ID")
        records.append(
            _base_record(
                record_id=citekey or f"ris:{index}",
                source_format="ris",
                source_name=source_name,
                title=title,
                authors=_normalize_authors("; ".join(authors)),
                year=year,
                venue=venue,
                doi=doi,
                url=url,
                abstract=abstract,
                citekey=citekey,
            )
        )
    return records


def _base_record(
    *,
    record_id: str,
    source_format: str,
    source_name: str,
    title: str,
    authors: list[str],
    year: int | None,
    venue: str,
    doi: str,
    url: str,
    abstract: str,
    citekey: str = "",
    source_provider: str = "",
) -> dict[str, Any]:
    provider = _normalize_source_provider(source_provider or source_format)
    record = {
        "record_id": record_id,
        "source_format": source_format,
        "source_provider": provider,
        "source_paths": [source_name],
        "title": title,
        "authors": authors,
        "year": year,
        "venue": venue,
        "doi": doi,
        "url": url,
        "abstract": abstract,
        "citekey": citekey,
    }
    record["field_provenance"] = _build_field_provenance(record)
    return record


def _merge_record(
    existing: dict[str, Any],
    candidate: dict[str, Any],
    merge_trace: list[dict[str, Any]] | None = None,
) -> None:
    for field in (
        "title",
        "authors",
        "year",
        "venue",
        "doi",
        "url",
        "abstract",
        "citekey",
        "publisher",
        "volume",
        "issue",
        "pages",
        "oa_url",
        "openalex_id",
    ):
        _merge_field(existing, candidate, field, merge_trace)
    existing_paths = set(existing.get("source_paths", []))
    existing_paths.update(candidate.get("source_paths", []))
    existing["source_paths"] = sorted(existing_paths)
    _merge_record_provenance(existing, candidate)
    for key, value in candidate.items():
        if key in {
            "record_id",
            "source_format",
            "source_provider",
            "source_paths",
            "title",
            "authors",
            "year",
            "venue",
            "doi",
            "url",
            "abstract",
            "citekey",
            "publisher",
            "volume",
            "issue",
            "pages",
            "oa_url",
            "openalex_id",
            "field_provenance",
        }:
            continue
        if value in ("", None, []):
            continue
        if key not in existing or existing.get(key) in ("", None, []):
            existing[key] = value


def _record_merge_key(record: dict[str, Any]) -> tuple[str, str]:
    doi = _clean_string(record.get("doi")).lower()
    if doi:
        return f"doi:{doi}", "doi"
    title = _normalize_title(record.get("title"))
    year = str(record.get("year") or "").strip()
    first_author = _author_key(record.get("authors", []))
    if title and year and first_author:
        return f"title_year_author:{title}:{year}:{first_author}", "title+year+author"
    if title and year:
        return f"title_year:{title}:{year}", "title+year"
    if title:
        return f"title:{title}", "title"
    return f"record:{record.get('record_id', '')}", "record_id"


def _generate_citekey(record: dict[str, Any]) -> str:
    author_token = _author_key(record.get("authors", [])) or "anon"
    year_token = str(record.get("year") or "nd")
    title_token = _title_keyword(record.get("title")) or "paper"
    return f"{author_token}{year_token}{title_token}"


def _title_keyword(raw: Any) -> str:
    for token in TITLE_WORD_RE.findall(str(raw or "").lower()):
        if token in STOPWORDS:
            continue
        return NON_ALNUM_RE.sub("", token)
    return ""


def _author_key(raw_authors: Any) -> str:
    authors = raw_authors if isinstance(raw_authors, list) else _normalize_authors(str(raw_authors or ""))
    if not authors:
        return ""
    first = authors[0]
    if "," in first:
        family = first.split(",", 1)[0]
    else:
        family = first.split()[-1]
    return NON_ALNUM_RE.sub("", family.lower())


def _normalize_authors(raw: str) -> list[str]:
    if not raw.strip():
        return []
    if " and " in raw:
        tokens = [item.strip() for item in raw.split(" and ") if item.strip()]
    elif ";" in raw:
        tokens = [item.strip() for item in raw.split(";") if item.strip()]
    else:
        tokens = [raw.strip()]
    return [" ".join(token.split()) for token in tokens]


def _extract_bib_field(body: str, field: str) -> str:
    pattern = re.compile(rf"\b{re.escape(field)}\s*=\s*([{{\"])(.+?)(?:[}}\"])\s*,?", re.I | re.S)
    match = pattern.search(body)
    if not match:
        return ""
    return " ".join(match.group(2).split())


def _normalize_title(raw: Any) -> str:
    return NON_ALNUM_RE.sub("", " ".join(str(raw or "").lower().split()))


def _clean_string(raw: Any) -> str:
    return " ".join(str(raw or "").split())


def _safe_int(raw: Any) -> int | None:
    try:
        return int(str(raw).strip())
    except (TypeError, ValueError):
        return None


def _first_nonempty(mapping: dict[str, list[str]], *keys: str) -> str:
    for key in keys:
        values = mapping.get(key, [])
        for value in values:
            if value:
                return value
    return ""


def _normalize_external_record(item: dict[str, Any], index: int) -> dict[str, Any]:
    authors_raw = item.get("authors", [])
    if isinstance(authors_raw, list):
        authors = [" ".join(str(author).split()) for author in authors_raw if str(author).strip()]
    else:
        authors = _normalize_authors(str(authors_raw or ""))
    provider = _normalize_source_provider(
        _clean_string(item.get("source_provider"))
        or _clean_string(item.get("source"))
        or _clean_string(item.get("provider"))
        or _clean_string(item.get("source_format"))
        or "external_enrichment"
    )
    record = _base_record(
        record_id=_clean_string(item.get("record_id")) or _clean_string(item.get("id")) or f"external:{index}",
        source_format=_clean_string(item.get("source_format")) or provider or "external_enrichment",
        source_name=_clean_string(item.get("source")) or provider or "external_enrichment",
        title=_clean_string(item.get("title")),
        authors=authors,
        year=_safe_int(item.get("year")),
        venue=_clean_string(item.get("venue")),
        doi=normalize_doi(_clean_string(item.get("doi"))) if _clean_string(item.get("doi")) else "",
        url=_clean_string(item.get("url")),
        abstract=_clean_string(item.get("abstract")),
        citekey=_clean_string(item.get("citekey")),
        source_provider=provider,
    )
    for key in ("openalex_id", "publisher", "volume", "issue", "pages", "oa_url", "citation_count"):
        value = item.get(key)
        if value not in ("", None, []):
            record[key] = value
            if key in PROVENANCE_TRACKED_FIELDS:
                record.setdefault("field_provenance", {})[key] = _field_provenance_meta(record)
    return record


def _merge_external_records(
    records: list[dict[str, Any]],
    external_records: list[dict[str, Any]],
) -> tuple[list[dict[str, Any]], int, list[dict[str, Any]]]:
    merged = [dict(record) for record in records]
    index = {_record_merge_key(record)[0]: record for record in merged}
    applied_count = 0
    merge_trace: list[dict[str, Any]] = []
    for external in external_records:
        key, _ = _record_merge_key(external)
        existing = index.get(key)
        if existing is None:
            merged.append(external)
            index[key] = external
            applied_count += 1
            merge_trace.append(
                {
                    "record_id": str(external.get("record_id", "")),
                    "decision": "append_record",
                    "provider": str(external.get("source_provider", "")),
                    "match_key": key,
                }
            )
            continue
        _merge_record(existing, external, merge_trace)
        applied_count += 1
    assign_citekeys(merged)
    return merged, applied_count, merge_trace


def _normalize_provenance(raw: Any) -> list[str]:
    if isinstance(raw, list):
        return [str(item) for item in raw]
    if raw:
        return [str(raw)]
    return [METADATA_ENRICH_ENV]


def _merge_field(
    existing: dict[str, Any],
    candidate: dict[str, Any],
    field: str,
    merge_trace: list[dict[str, Any]] | None = None,
) -> None:
    candidate_value = candidate.get(field)
    if candidate_value in ("", None, []):
        return

    existing_value = existing.get(field)
    candidate_provider = _normalize_source_provider(candidate.get("source_provider") or candidate.get("source_format"))
    existing_provider = _normalize_source_provider(existing.get("source_provider") or existing.get("source_format"))
    if existing_value in ("", None, []):
        existing[field] = candidate_value
        _set_field_provenance(existing, field, candidate)
        _append_merge_trace(
            merge_trace,
            existing=existing,
            field=field,
            decision="fill_missing",
            existing_provider=existing_provider,
            candidate_provider=candidate_provider,
        )
        return

    if field in FILL_ONLY_FIELDS:
        _append_merge_trace(
            merge_trace,
            existing=existing,
            field=field,
            decision="keep_existing_fill_only",
            existing_provider=existing_provider,
            candidate_provider=candidate_provider,
        )
        return

    if not _should_prefer_candidate(existing, candidate, field):
        _append_merge_trace(
            merge_trace,
            existing=existing,
            field=field,
            decision="keep_existing",
            existing_provider=existing_provider,
            candidate_provider=candidate_provider,
        )
        return

    existing[field] = candidate_value
    _set_field_provenance(existing, field, candidate)
    _append_merge_trace(
        merge_trace,
        existing=existing,
        field=field,
        decision="prefer_candidate",
        existing_provider=existing_provider,
        candidate_provider=candidate_provider,
    )


def _should_prefer_candidate(existing: dict[str, Any], candidate: dict[str, Any], field: str) -> bool:
    existing_priority = _field_priority(existing, field)
    candidate_priority = _field_priority(candidate, field)

    existing_value = existing.get(field)
    candidate_value = candidate.get(field)

    if field == "authors":
        existing_len = len(existing_value) if isinstance(existing_value, list) else len(_normalize_authors(str(existing_value or "")))
        candidate_len = len(candidate_value) if isinstance(candidate_value, list) else len(_normalize_authors(str(candidate_value or "")))
        return candidate_priority > existing_priority and candidate_len >= existing_len

    if field == "year":
        return candidate_priority > existing_priority

    if field in {"title", "venue", "publisher", "volume", "issue", "pages"}:
        return candidate_priority > existing_priority and _string_quality(candidate_value) >= _string_quality(existing_value)

    if field == "url":
        return _url_quality(candidate_value) > _url_quality(existing_value) or (
            candidate_priority > existing_priority and _url_quality(candidate_value) >= _url_quality(existing_value)
        )

    if field == "oa_url":
        return candidate_priority >= existing_priority

    if field == "abstract":
        return candidate_priority > existing_priority and len(str(candidate_value)) >= len(str(existing_value))

    return candidate_priority > existing_priority


def _field_priority(record: dict[str, Any], field: str) -> int:
    provenance = record.get("field_provenance", {})
    meta = provenance.get(field) if isinstance(provenance, dict) else None
    if isinstance(meta, dict):
        provider = _normalize_source_provider(meta.get("provider") or meta.get("source_format"))
        if provider:
            return FIELD_PROVIDER_PRIORITIES.get(field, {}).get(
                provider,
                SOURCE_PROVIDER_PRIORITIES.get(provider, 20),
            )
    provider = _normalize_source_provider(record.get("source_provider") or record.get("source_format"))
    return FIELD_PROVIDER_PRIORITIES.get(field, {}).get(
        provider,
        SOURCE_PROVIDER_PRIORITIES.get(provider, 20),
    )


def _build_field_provenance(record: dict[str, Any]) -> dict[str, dict[str, Any]]:
    provenance: dict[str, dict[str, Any]] = {}
    for field in PROVENANCE_TRACKED_FIELDS:
        value = record.get(field)
        if value in ("", None, []):
            continue
        provenance[field] = _field_provenance_meta(record)
    return provenance


def _field_provenance_meta(record: dict[str, Any]) -> dict[str, Any]:
    source_paths = record.get("source_paths", [])
    source_path = source_paths[0] if isinstance(source_paths, list) and source_paths else ""
    return {
        "provider": _normalize_source_provider(record.get("source_provider") or record.get("source_format")),
        "source_format": _clean_string(record.get("source_format")),
        "source_path": _clean_string(source_path),
    }


def _set_field_provenance(existing: dict[str, Any], field: str, candidate: dict[str, Any]) -> None:
    provenance = existing.setdefault("field_provenance", {})
    if isinstance(provenance, dict):
        provenance[field] = _field_provenance_meta(candidate)


def _append_merge_trace(
    merge_trace: list[dict[str, Any]] | None,
    *,
    existing: dict[str, Any],
    field: str,
    decision: str,
    existing_provider: str,
    candidate_provider: str,
) -> None:
    if merge_trace is None:
        return
    merge_trace.append(
        {
            "record_id": str(existing.get("record_id", "")),
            "field": field,
            "decision": decision,
            "existing_provider": existing_provider or "-",
            "candidate_provider": candidate_provider or "-",
        }
    )


def _merge_record_provenance(existing: dict[str, Any], candidate: dict[str, Any]) -> None:
    existing_provider = _normalize_source_provider(existing.get("source_provider") or existing.get("source_format"))
    candidate_provider = _normalize_source_provider(candidate.get("source_provider") or candidate.get("source_format"))
    if SOURCE_PROVIDER_PRIORITIES.get(candidate_provider, 20) > SOURCE_PROVIDER_PRIORITIES.get(existing_provider, 20):
        existing["source_provider"] = candidate_provider


def _normalize_source_provider(raw: Any) -> str:
    value = _clean_string(raw).lower().replace("-", "_").replace(" ", "_")
    aliases = {
        "search_results.csv": "search_results",
        "retrieval_manifest.csv": "retrieval_manifest",
        "paper_note": "paper_note",
        "semantic_scholar": "semantic_scholar",
    }
    return aliases.get(value, value)


def _string_quality(value: Any) -> int:
    return len(_clean_string(value))


def _url_quality(value: Any) -> int:
    lowered = _clean_string(value).lower()
    if not lowered:
        return 0
    if lowered.endswith(".pdf") or "/pdf" in lowered:
        return 3
    if "openalex.org" in lowered or "doi.org" in lowered:
        return 2
    return 1
