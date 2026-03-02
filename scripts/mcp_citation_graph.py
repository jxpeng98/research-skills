#!/usr/bin/env python3
import json
import sys
from pathlib import Path
from bridges.providers.s2_client import get_citations, get_references

def main():
    try:
        input_data = sys.stdin.read()
        if not input_data.strip():
            print(json.dumps({"status": "error", "summary": "No input provided"}))
            return

        payload = json.loads(input_data)
        task_packet = payload.get("task_packet", {})
        # Note: in a real implementation, you'd extract specific Semantic Scholar IDs 
        # from the task_packet's contextual data (like notes/ or bibliography.bib).
        # For this reference implementation, we require a paper_id in the packet.
        paper_id = task_packet.get("target_paper_id", "")
        
        if not paper_id:
            print(json.dumps({
                "status": "warning", 
                "summary": "No 'target_paper_id' found in task_packet. Cannot build graph.",
                "data": {"error": "Missing target_paper_id"}
            }))
            return

        citations = get_citations(paper_id, limit=5)
        references = get_references(paper_id, limit=5)
        
        if "error" in citations or "error" in references:
            err = citations.get("error", "") or references.get("error", "")
            print(json.dumps({
                "status": "error", 
                "summary": f"S2 Graph API Error: {err}",
                "data": {"error": err}
            }))
            return

        cite_data = citations.get("data", [])
        ref_data = references.get("data", [])
        
        output = {
            "status": "ok",
            "summary": f"Fetched {len(cite_data)} citations and {len(ref_data)} references for {paper_id}",
            "provenance": ["https://api.semanticscholar.org/graph/v1"],
            "data": {
                "citations": cite_data,
                "references": ref_data
            }
        }
        
        print(json.dumps(output, ensure_ascii=False))
        
    except Exception as e:
        print(json.dumps({
            "status": "error",
            "summary": f"Citation graph provider exception: {str(e)}",
            "data": {"error": str(e)}
        }))

if __name__ == "__main__":
    main()
