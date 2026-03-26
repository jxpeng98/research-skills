from __future__ import annotations

import json
import os
import shlex
import subprocess
from pathlib import Path
from typing import Any


def invoke_overlay_json(
    *,
    env_name: str,
    payload: dict[str, Any],
    cwd: Path,
    timeout_seconds: int,
    label: str,
) -> tuple[dict[str, Any] | None, dict[str, Any]]:
    command = os.environ.get(env_name, "").strip()
    if not command:
        return None, {"configured": False}

    try:
        parsed_cmd = shlex.split(command)
        run_result = subprocess.run(
            parsed_cmd,
            input=json.dumps(payload, ensure_ascii=False),
            capture_output=True,
            text=True,
            cwd=str(cwd),
            timeout=timeout_seconds,
            check=False,
        )
    except subprocess.TimeoutExpired:
        return None, {
            "configured": True,
            "status": "error",
            "summary": f"{label} timed out after {timeout_seconds}s.",
            "provenance": [env_name],
        }
    except OSError as exc:
        return None, {
            "configured": True,
            "status": "error",
            "summary": f"{label} failed to start: {exc}",
            "provenance": [env_name],
        }

    if run_result.returncode != 0:
        stderr = (run_result.stderr or "").strip()
        return None, {
            "configured": True,
            "status": "error",
            "summary": f"{label} exited with code {run_result.returncode}.",
            "provenance": [env_name, stderr] if stderr else [env_name],
        }

    stdout = (run_result.stdout or "").strip()
    if not stdout:
        return None, {
            "configured": True,
            "status": "warning",
            "summary": f"{label} returned empty output.",
            "provenance": [env_name],
        }

    try:
        parsed = json.loads(stdout)
    except json.JSONDecodeError:
        return None, {
            "configured": True,
            "status": "warning",
            "summary": f"{label} returned non-JSON output.",
            "provenance": [env_name, stdout[:280]],
        }

    return parsed, {"configured": True}
