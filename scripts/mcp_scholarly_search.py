#!/usr/bin/env python3
import json
import sys
from pathlib import Path
from bridges.providers.s2_client import search_paper

def main():
    try:
        input_data = sys.stdin.read()
        if not input_data.strip():
            print(json.dumps({"status": "error", "summary": "No input provided"}))
            return

        payload = json.loads(input_data)
        task_packet = payload.get("task_packet", {})
        topic = task_packet.get("topic", "")
        
        if not topic:
            print(json.dumps({"status": "warning", "summary": "Empty topic, no search performed."}))
            return

        # Perform the actual search
        results = search_paper(topic, limit=10)
        
        if "error" in results:
            print(json.dumps({
                "status": "error", 
                "summary": f"S2 API Error: {results['error']}",
                "data": {"error": results["error"]}
            }))
            return

        data_array = results.get("data", [])
        
        output = {
            "status": "ok",
            "summary": f"Found {len(data_array)} papers on Semantic Scholar for '{topic}'",
            "provenance": ["https://api.semanticscholar.org/graph/v1"],
            "data": {
                "results": data_array
            }
        }
        
        print(json.dumps(output, ensure_ascii=False))
        
    except Exception as e:
        print(json.dumps({
            "status": "error",
            "summary": f"Scholarly search provider exception: {str(e)}",
            "data": {"error": str(e)}
        }))

if __name__ == "__main__":
    main()
