from __future__ import annotations

import csv
import io
import json
import re
from pathlib import Path
from typing import Any


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
) -> dict[str, Any]:
    return {
        "record_id": record_id,
        "source_format": source_format,
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


def _merge_record(existing: dict[str, Any], candidate: dict[str, Any]) -> None:
    for field in ("title", "venue", "doi", "url", "abstract"):
        if not existing.get(field) and candidate.get(field):
            existing[field] = candidate[field]
    if not existing.get("authors") and candidate.get("authors"):
        existing["authors"] = candidate["authors"]
    if existing.get("year") is None and candidate.get("year") is not None:
        existing["year"] = candidate["year"]
    if not existing.get("citekey") and candidate.get("citekey"):
        existing["citekey"] = candidate["citekey"]
    existing_paths = set(existing.get("source_paths", []))
    existing_paths.update(candidate.get("source_paths", []))
    existing["source_paths"] = sorted(existing_paths)


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
