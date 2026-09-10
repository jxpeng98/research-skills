"""Test-only read-resource MCP: immutable repository content in memory, no project I/O."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import sys

SERVER = "qiongli_eval_resources"
TOOL = "read_resource"


def read_result(resources: dict, arguments: dict) -> dict:
    if (not isinstance(arguments, dict) or set(arguments) != {"path"}
            or not isinstance(arguments["path"], str) or arguments["path"] not in resources):
        raise ValueError("Unknown resource; use a package-relative path from the Qiongli entry")
    path = arguments["path"]
    text = resources[path]
    value = {"path": path, "sha256": hashlib.sha256(text.encode()).hexdigest(), "text": text}
    return {"content": [{"type": "text", "text": json.dumps(value, ensure_ascii=False)}],
            "structuredContent": value, "isError": False}


def dispatch(request: dict, resources: dict) -> dict | None:
    if not isinstance(request, dict) or request.get("jsonrpc") != "2.0":
        raise ValueError("Invalid JSON-RPC request")
    if "id" not in request:
        return None
    reply = {"jsonrpc": "2.0", "id": request["id"]}
    method = request.get("method")
    if method == "initialize":
        reply["result"] = {"protocolVersion": request["params"]["protocolVersion"],
                           "serverInfo": {"name": SERVER, "version": "1.0"},
                           "capabilities": {"tools": {}}}
    elif method == "ping":
        reply["result"] = {}
    elif method == "tools/list":
        reply["result"] = {"tools": [{"name": TOOL,
            "description": "Read a Qiongli package-relative workflow, skill card or reference. Test-only repository content; no project state or write access.",
            "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}},
                            "required": ["path"], "additionalProperties": False},
            "annotations": {"readOnlyHint": True, "destructiveHint": False, "openWorldHint": False}}]}
    elif method == "tools/call":
        try:
            params = request.get("params", {})
            if params.get("name") != TOOL:
                raise ValueError("Unsupported tool")
            reply["result"] = read_result(resources, params.get("arguments"))
        except (ValueError, TypeError, AttributeError) as error:
            reply["result"] = {"content": [{"type": "text", "text": str(error)}], "isError": True}
    else:
        reply["error"] = {"code": -32601, "message": "Unsupported method"}
    return reply


def observed_reads(raw: str, resources: dict) -> tuple[str, list[str]]:
    """Validate actual tool starts/results; text claiming a read never counts."""
    events = [json.loads(line) for line in raw.splitlines() if line.strip()]
    pending, completed, reads, remaining = {}, set(), [], []
    last_read, last_answer = -1, -1
    for index, event in enumerate(events):
        if not isinstance(event, dict):
            raise ValueError("Invalid event")
        kind, item = event.get("type"), event.get("item", {})
        if not isinstance(item, dict):
            raise ValueError("Invalid event item")
        if item.get("type") != "mcp_tool_call":
            remaining.append(event)
            if kind == "item.completed" and item.get("type") == "agent_message":
                last_answer = index
            continue
        if index < 2 or index == len(events) - 1:
            raise ValueError("Resource call outside a completed turn")
        if item.get("server") != SERVER or item.get("tool") != TOOL:
            raise ValueError("Unexpected tool or server")
        expected = read_result(resources, item.get("arguments"))
        call_id = item.get("id")
        if not isinstance(call_id, str) or not call_id:
            raise ValueError("Missing tool call ID")
        if kind == "item.started":
            if call_id in pending or call_id in completed:
                raise ValueError("Repeated tool start")
            if item.get("status") != "in_progress" or item.get("error") is not None or item.get("result") is not None:
                raise ValueError("Invalid or failed tool start")
            pending[call_id] = item["arguments"]
        elif kind == "item.completed":
            if pending.pop(call_id, None) != item["arguments"] or item.get("status") != "completed" or item.get("error") is not None:
                raise ValueError("Unmatched or unsuccessful tool completion")
            result = item.get("result")
            if (not isinstance(result, dict) or result.get("isError", False) is not False
                    or result.get("content") != expected["content"]
                    or ("structuredContent" in result and result["structuredContent"] != expected["structuredContent"])):
                raise ValueError("Resource result differs from captured source bytes")
            reads.append(item["arguments"]["path"])
            completed.add(call_id)
            last_read = index
        else:
            raise ValueError("Unsupported MCP tool event")
    if pending or last_read > last_answer:
        raise ValueError("Incomplete resource read or missing answer after reads")
    return "\n".join(map(json.dumps, remaining)), reads


if __name__ == "__main__":
    resources = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
    for line in sys.stdin:
        try:
            response = dispatch(json.loads(line), resources)
        except (ValueError, KeyError, TypeError):
            response = {"jsonrpc": "2.0", "id": None, "error": {"code": -32600, "message": "Invalid request"}}
        if response is not None:
            print(json.dumps(response, ensure_ascii=False), flush=True)
