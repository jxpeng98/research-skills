import json
import os
import time
import urllib.error
import urllib.parse
import urllib.request
from typing import Any

S2_GRAPH_BASE = "https://api.semanticscholar.org/graph/v1"
DEFAULT_TIMEOUT_SECONDS = 15
DEFAULT_MAX_ATTEMPTS = 3
RETRYABLE_HTTP_CODES = {429, 500, 502, 503, 504}

def search_paper(query: str, limit: int = 10) -> dict[str, Any]:
    """Search for papers by keyword."""
    if not query.strip():
        return {"data": []}
    
    encoded_query = urllib.parse.quote(query)
    url = f"{S2_GRAPH_BASE}/paper/search?query={encoded_query}&limit={limit}&fields=title,authors,year,abstract,url,citationCount,venue"
    
    return _make_request(url)

def get_paper_details(paper_id: str) -> dict[str, Any]:
    """Get detailed information about a specific paper."""
    url = f"{S2_GRAPH_BASE}/paper/{paper_id}?fields=title,authors,year,abstract,url,citationCount,referenceCount,venue"
    return _make_request(url)

def get_citations(paper_id: str, limit: int = 20) -> dict[str, Any]:
    """Get papers that cite the target paper."""
    url = f"{S2_GRAPH_BASE}/paper/{paper_id}/citations?limit={limit}&fields=title,authors,year,venue,url,citationCount"
    return _make_request(url)

def get_references(paper_id: str, limit: int = 20) -> dict[str, Any]:
    """Get papers referenced by the target paper."""
    url = f"{S2_GRAPH_BASE}/paper/{paper_id}/references?limit={limit}&fields=title,authors,year,venue,url,citationCount"
    return _make_request(url)

def _make_request(url: str) -> dict[str, Any]:
    headers = {
        "User-Agent": "Research-Skills-MCP/1.0",
        "Accept": "application/json",
    }
    api_key = os.environ.get("SEMANTIC_SCHOLAR_API_KEY") or os.environ.get("S2_API_KEY")
    if api_key:
        headers["x-api-key"] = api_key

    for attempt in range(1, DEFAULT_MAX_ATTEMPTS + 1):
        try:
            req = urllib.request.Request(url, headers=headers)
            with urllib.request.urlopen(req, timeout=DEFAULT_TIMEOUT_SECONDS) as response:
                content = response.read()
                return dict(json.loads(content))
        except urllib.error.HTTPError as exc:
            if exc.code in RETRYABLE_HTTP_CODES and attempt < DEFAULT_MAX_ATTEMPTS:
                time.sleep(_retry_delay_seconds(exc, attempt))
                continue
            return {"error": _format_http_error(exc), "data": []}
        except urllib.error.URLError as exc:
            if attempt < DEFAULT_MAX_ATTEMPTS:
                time.sleep(float(attempt))
                continue
            return {"error": str(exc), "data": []}
        except Exception as exc:
            return {"error": str(exc), "data": []}

    return {"error": "Semantic Scholar request exhausted retries.", "data": []}


def _retry_delay_seconds(exc: urllib.error.HTTPError, attempt: int) -> float:
    retry_after = exc.headers.get("Retry-After") if exc.headers else None
    if retry_after:
        try:
            return max(float(retry_after), 1.0)
        except ValueError:
            pass
    return float(2 ** (attempt - 1))


def _format_http_error(exc: urllib.error.HTTPError) -> str:
    reason = str(exc.reason or "").strip()
    if reason:
        return f"HTTP Error {exc.code}: {reason}"
    return f"HTTP Error {exc.code}"
