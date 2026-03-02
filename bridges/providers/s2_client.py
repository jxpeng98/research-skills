import json
import urllib.parse
import urllib.request
from typing import Any

S2_GRAPH_BASE = "https://api.semanticscholar.org/graph/v1"

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
    try:
        req = urllib.request.Request(
            url,
            headers={"User-Agent": "Research-Skills-MCP/1.0"}
        )
        with urllib.request.urlopen(req, timeout=15) as response:
            content = response.read()
            return dict(json.loads(content))
    except Exception as exc:
        return {"error": str(exc), "data": []}
