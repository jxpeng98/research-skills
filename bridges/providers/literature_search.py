from __future__ import annotations

import re
from datetime import datetime, timezone
from typing import Any, Callable


SearchFn = Callable[[str, int], dict[str, Any]]

MAX_QUERY_VARIANTS = 4
DEFAULT_PER_QUERY_LIMIT = 8
STOPWORDS = {
    "a",
    "an",
    "and",
    "are",
    "be",
    "by",
    "for",
    "from",
    "how",
    "in",
    "into",
    "is",
    "of",
    "on",
    "or",
    "that",
    "the",
    "their",
    "this",
    "to",
    "what",
    "when",
    "where",
    "which",
    "why",
    "with",
}
DOI_PREFIX_RE = re.compile(r"^(?:https?://(?:dx\.)?doi\.org/|doi:\s*)", re.I)
NON_ALNUM_RE = re.compile(r"[^a-z0-9]+")


def build_query_variants(task_packet: dict[str, Any]) -> list[dict[str, str]]:
    seen: set[str] = set()
    variants: list[dict[str, str]] = []

    def add_variant(query: str, rationale: str) -> None:
        cleaned = " ".join(str(query).strip().split())
        if not cleaned:
            return
        key = cleaned.casefold()
        if key in seen:
            return
        seen.add(key)
        variants.append(
            {
                "query_id": f"q{len(variants) + 1}",
                "query": cleaned,
                "rationale": rationale,
            }
        )

    topic = str(task_packet.get("topic", "")).strip()
    if topic:
        add_variant(topic, "topic seed")

    direct_query = str(task_packet.get("query", "")).strip()
    if direct_query:
        add_variant(direct_query, "explicit query")

    research_question = str(task_packet.get("research_question", "")).strip()
    if research_question:
        add_variant(research_question, "research question")
        distilled = _distill_question(research_question)
        if distilled and distilled.casefold() != research_question.casefold():
            add_variant(distilled, "distilled research question keywords")

    keyword_bundle = _build_keyword_bundle(task_packet.get("keywords"))
    if keyword_bundle:
        add_variant(keyword_bundle, "keyword bundle")

    for alias_key in ("target_title", "title"):
        alias_value = str(task_packet.get(alias_key, "")).strip()
        if alias_value:
            add_variant(alias_value, f"{alias_key} seed")

    return variants[:MAX_QUERY_VARIANTS]


def run_scholarly_search(
    task_packet: dict[str, Any],
    search_fn: SearchFn,
    *,
    retrieved_at: str | None = None,
) -> dict[str, Any]:
    query_variants = build_query_variants(task_packet)
    if not query_variants:
        return {
            "status": "warning",
            "summary": "Empty topic/query context, no scholarly search performed.",
            "provenance": [],
            "data": {
                "provider_mode": "builtin_semantic_scholar_baseline",
                "query_variants": [],
                "search_results": [],
                "dedup_log": [],
                "search_log": [],
                "artifact_bundle": _artifact_bundle(),
            },
        }

    per_query_limit = _resolve_per_query_limit(task_packet)
    timestamp = retrieved_at or datetime.now(timezone.utc).replace(microsecond=0).isoformat()
    normalized_results: list[dict[str, Any]] = []
    search_log: list[dict[str, Any]] = []
    failures: list[dict[str, str]] = []

    for variant in query_variants:
        response = search_fn(variant["query"], per_query_limit)
        error = str(response.get("error", "")).strip()
        hits = response.get("data", [])
        if not isinstance(hits, list):
            hits = []

        if error:
            failures.append({"query_id": variant["query_id"], "query": variant["query"], "error": error})

        search_log.append(
            {
                "query_id": variant["query_id"],
                "query": variant["query"],
                "provider": "semantic_scholar",
                "retrieved_at": timestamp,
                "retrieved_count": len(hits),
                "status": "error" if error else "ok",
                "error": error,
            }
        )

        for offset, hit in enumerate(hits, start=1):
            if not isinstance(hit, dict):
                continue
            normalized_results.append(
                normalize_search_hit(
                    hit,
                    query_id=variant["query_id"],
                    query_text=variant["query"],
                    retrieved_at=timestamp,
                    ordinal=offset,
                )
            )

    unique_results, dedup_log = dedupe_search_results(normalized_results)
    duplicate_count = len(dedup_log)

    if unique_results:
        status = "warning" if failures else "ok"
        summary = (
            f"Found {len(unique_results)} unique papers across {len(query_variants)} query variants "
            f"({len(normalized_results)} raw hits, {duplicate_count} deduplicated)."
        )
    elif failures:
        status = "error"
        summary = (
            f"Scholarly search failed for all {len(query_variants)} query variants; "
            f"last error: {failures[-1]['error']}"
        )
    else:
        status = "warning"
        summary = (
            f"No papers returned across {len(query_variants)} query variants for the current topic."
        )

    return {
        "status": status,
        "summary": summary,
        "provenance": ["https://api.semanticscholar.org/graph/v1"],
        "data": {
            "provider_mode": "builtin_semantic_scholar_baseline",
            "query_variants": query_variants,
            "per_query_limit": per_query_limit,
            "raw_result_count": len(normalized_results),
            "unique_result_count": len(unique_results),
            "duplicate_count": duplicate_count,
            "search_results": unique_results,
            "dedup_log": dedup_log,
            "search_log": search_log,
            "failures": failures,
            "artifact_bundle": _artifact_bundle(),
        },
    }


def normalize_search_hit(
    hit: dict[str, Any],
    *,
    query_id: str,
    query_text: str,
    retrieved_at: str,
    ordinal: int,
) -> dict[str, Any]:
    paper_id = str(hit.get("paperId") or "").strip()
    external_ids = hit.get("externalIds", {})
    if not isinstance(external_ids, dict):
        external_ids = {}
    doi = _normalize_doi(external_ids.get("DOI"))

    title = " ".join(str(hit.get("title", "")).split())
    authors = _flatten_authors(hit.get("authors"))
    year = _safe_int(hit.get("year"))
    venue = " ".join(str(hit.get("venue", "")).split())
    abstract = " ".join(str(hit.get("abstract", "")).split())
    url = str(hit.get("url", "")).strip()
    citation_count = _safe_int(hit.get("citationCount"))
    open_access_url = _extract_open_access_url(hit)
    record_id = paper_id or f"{query_id}-{ordinal}"

    return {
        "record_id": f"s2:{record_id}",
        "source": "semantic_scholar",
        "query_id": query_id,
        "query_text": query_text,
        "retrieved_at": retrieved_at,
        "paper_id": paper_id,
        "title": title,
        "authors": authors,
        "year": year,
        "venue": venue,
        "doi": doi,
        "url": url,
        "abstract": abstract,
        "citation_count": citation_count,
        "open_access_pdf_url": open_access_url,
    }


def dedupe_search_results(records: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], list[dict[str, str]]]:
    canonical_records: list[dict[str, Any]] = []
    canonical_by_key: dict[str, dict[str, Any]] = {}
    dedup_log: list[dict[str, str]] = []

    for record in records:
        match_key, match_basis = _record_match_key(record)
        existing = canonical_by_key.get(match_key)
        if existing is None:
            record["query_ids"] = [record.get("query_id", "")]
            canonical_records.append(record)
            canonical_by_key[match_key] = record
            continue

        _merge_record(existing, record)
        dedup_log.append(
            {
                "candidate_record_id": str(record.get("record_id", "")),
                "canonical_record_id": str(existing.get("record_id", "")),
                "decision": "merge_duplicate",
                "match_basis": match_basis,
                "resolver": "builtin_scholarly_search",
                "notes": f"Merged query {record.get('query_id', '')} into canonical record.",
            }
        )

    for record in canonical_records:
        query_ids = [query_id for query_id in record.pop("query_ids", []) if query_id]
        if query_ids:
            record["query_ids"] = ";".join(sorted(set(query_ids)))

    return canonical_records, dedup_log


def _artifact_bundle() -> dict[str, str]:
    return {
        "search_strategy": "search_strategy.md",
        "search_log": "search_log.md",
        "search_results": "search_results.csv",
        "dedup_log": "dedup_log.csv",
    }


def _resolve_per_query_limit(task_packet: dict[str, Any]) -> int:
    for key in ("per_query_limit", "limit", "search_limit"):
        value = task_packet.get(key)
        try:
            parsed = int(str(value).strip())
        except (TypeError, ValueError):
            continue
        return max(1, min(parsed, 25))
    return DEFAULT_PER_QUERY_LIMIT


def _build_keyword_bundle(raw_keywords: Any) -> str:
    if not isinstance(raw_keywords, list):
        return ""
    cleaned: list[str] = []
    for item in raw_keywords:
        text = " ".join(str(item).strip().split())
        if not text:
            continue
        cleaned.append(f"\"{text}\"" if " " in text else text)
    return " ".join(cleaned[:6])


def _distill_question(question: str) -> str:
    terms: list[str] = []
    seen: set[str] = set()
    for token in re.findall(r"[A-Za-z0-9][A-Za-z0-9-]{2,}", question.lower()):
        if token in STOPWORDS:
            continue
        if token in seen:
            continue
        seen.add(token)
        terms.append(token)
        if len(terms) >= 8:
            break
    return " ".join(terms)


def _flatten_authors(raw_authors: Any) -> str:
    if not isinstance(raw_authors, list):
        return ""
    names: list[str] = []
    for author in raw_authors:
        if isinstance(author, dict):
            name = " ".join(str(author.get("name", "")).split())
            if name:
                names.append(name)
    return "; ".join(names)


def _normalize_doi(raw: Any) -> str:
    value = " ".join(str(raw or "").strip().split())
    if not value:
        return ""
    value = DOI_PREFIX_RE.sub("", value)
    return value.rstrip(".,);]").lower()


def _safe_int(raw: Any) -> int | None:
    try:
        return int(raw)
    except (TypeError, ValueError):
        return None


def _extract_open_access_url(hit: dict[str, Any]) -> str:
    payload = hit.get("openAccessPdf")
    if isinstance(payload, dict):
        return str(payload.get("url", "")).strip()
    return ""


def _record_match_key(record: dict[str, Any]) -> tuple[str, str]:
    doi = str(record.get("doi", "")).strip().lower()
    if doi:
        return f"doi:{doi}", "doi"

    paper_id = str(record.get("paper_id", "")).strip()
    if paper_id:
        return f"paper_id:{paper_id}", "paper_id"

    title = _normalize_title(record.get("title"))
    year = str(record.get("year") or "").strip()
    if title and year:
        return f"title_year:{title}:{year}", "title+year"
    if title:
        return f"title:{title}", "title"
    return f"record:{record.get('record_id', '')}", "record_id"


def _normalize_title(raw: Any) -> str:
    value = " ".join(str(raw or "").strip().lower().split())
    return NON_ALNUM_RE.sub("", value)


def _merge_record(canonical: dict[str, Any], candidate: dict[str, Any]) -> None:
    query_ids = canonical.setdefault("query_ids", [])
    if isinstance(query_ids, list):
        query_ids.append(str(candidate.get("query_id", "")))

    for key in ("doi", "url", "abstract", "venue", "open_access_pdf_url"):
        if not canonical.get(key) and candidate.get(key):
            canonical[key] = candidate[key]

    if not canonical.get("authors") and candidate.get("authors"):
        canonical["authors"] = candidate["authors"]

    if not canonical.get("paper_id") and candidate.get("paper_id"):
        canonical["paper_id"] = candidate["paper_id"]

    canonical_citations = canonical.get("citation_count")
    candidate_citations = candidate.get("citation_count")
    if isinstance(candidate_citations, int):
        if not isinstance(canonical_citations, int) or candidate_citations > canonical_citations:
            canonical["citation_count"] = candidate_citations

