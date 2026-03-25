#!/usr/bin/env python3
import json
import sys
from bridges.providers.literature_search import run_scholarly_search
from bridges.providers.s2_client import search_paper


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

        output = run_scholarly_search(task_packet, search_paper)
        print(json.dumps(output, ensure_ascii=False))
    except Exception as e:
        print(json.dumps({
            "status": "error",
            "summary": f"Scholarly search provider exception: {str(e)}",
            "data": {"error": str(e)}
        }))

if __name__ == "__main__":
    main()
