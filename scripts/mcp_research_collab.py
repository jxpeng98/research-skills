#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from bridges.providers.research_collab import (
    BROKER_URL_ENV,
    GeminiBrokerClientError,
    broker_client_from_env,
)


def main() -> None:
    try:
        raw = sys.stdin.read()
        if not raw.strip():
            print(json.dumps({"status": "error", "summary": "No input provided", "data": {}}))
            return

        payload = json.loads(raw)
        if not isinstance(payload, dict):
            print(json.dumps({"status": "error", "summary": "Input payload must be a JSON object", "data": {}}))
            return

        client = broker_client_from_env()
        if client is None:
            print(
                json.dumps(
                    {
                        "status": "not_configured",
                        "summary": f"Research collab broker not configured. Set {BROKER_URL_ENV}.",
                        "data": {"provider_mode": "gemini_broker"},
                    }
                )
            )
            return

        task_packet = payload.get("task_packet", {})
        if not isinstance(task_packet, dict):
            task_packet = {}
        action = str(
            task_packet.get("action")
            or payload.get("action")
            or "health"
        ).strip().lower()

        if action == "health":
            response = client.health()
        elif action == "reset":
            response = client.reset(reason=str(task_packet.get("reason", "")).strip() or None)
        elif action == "prompt":
            prompt = str(task_packet.get("prompt") or payload.get("prompt") or "").strip()
            cwd_text = str(task_packet.get("cwd") or payload.get("cwd") or Path.cwd())
            runtime_options = task_packet.get("runtime_options", {})
            if not isinstance(runtime_options, dict):
                runtime_options = {}
            response = client.prompt(
                prompt=prompt,
                cwd=Path(cwd_text),
                runtime_options=runtime_options,
                reset_session=bool(task_packet.get("reset_session", False)),
            )
        else:
            response = {
                "status": "warning",
                "summary": f"Unknown research-collab action: {action}",
                "data": {"provider_mode": "gemini_broker"},
            }
        print(json.dumps(response, ensure_ascii=False))
    except GeminiBrokerClientError as exc:
        print(
            json.dumps(
                {
                    "status": "error",
                    "summary": f"Research collab broker request failed: {exc}",
                    "data": {"error": str(exc), "provider_mode": "gemini_broker"},
                }
            )
        )
    except Exception as exc:
        print(
            json.dumps(
                {
                    "status": "error",
                    "summary": f"Research collab provider exception: {exc}",
                    "data": {"error": str(exc), "provider_mode": "gemini_broker"},
                }
            )
        )


if __name__ == "__main__":
    main()
