#!/usr/bin/env python3
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from bridges.providers.citation_graph import run_citation_graph
from bridges.providers.s2_client import get_citations, get_references, search_paper


def main() -> None:
    try:
        input_data = sys.stdin.read()
        if not input_data.strip():
            print(json.dumps({"status": "error", "summary": "No input provided"}))
            return

        payload = json.loads(input_data)
        task_packet = payload.get("task_packet", {})
        if not isinstance(task_packet, dict):
            task_packet = {}

        output = run_citation_graph(
            task_packet,
            Path.cwd(),
            search_fn=search_paper,
            citations_fn=get_citations,
            references_fn=get_references,
        )
        print(json.dumps(output, ensure_ascii=False))
    except Exception as e:
        print(json.dumps({
            "status": "error",
            "summary": f"Citation graph provider exception: {str(e)}",
            "data": {"error": str(e)}
        }))

if __name__ == "__main__":
    main()
