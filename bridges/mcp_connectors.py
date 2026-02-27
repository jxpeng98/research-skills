from __future__ import annotations

import json
import os
import shlex
import subprocess
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any


@dataclass
class MCPEvidence:
    provider: str
    status: str
    summary: str
    provenance: list[str] = field(default_factory=list)
    data: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


class MCPConnector:
    """Collect evidence from local and external MCP providers."""

    VALID_STATUSES = {"ok", "warning", "error", "not_configured"}

    def __init__(
        self,
        timeout_seconds: int = 20,
        env_prefix: str = "RESEARCH_MCP_",
    ):
        self.timeout_seconds = timeout_seconds
        self.env_prefix = env_prefix

    def collect(
        self,
        provider: str,
        task_packet: dict[str, Any],
        cwd: Path,
    ) -> MCPEvidence:
        if provider == "filesystem":
            return self._collect_filesystem(task_packet, cwd)
        return self._collect_external_command(provider, task_packet, cwd)

    def _collect_filesystem(
        self,
        task_packet: dict[str, Any],
        cwd: Path,
    ) -> MCPEvidence:
        topic = str(task_packet.get("topic", "")).strip()
        artifact_root = str(task_packet.get("artifact_root", "RESEARCH/[topic]/"))
        project_root = cwd / artifact_root.replace("[topic]", topic)
        required_outputs = [
            str(item) for item in task_packet.get("required_outputs", []) if str(item).strip()
        ]

        existing_count = 0
        provenance: list[str] = []
        for rel_path in required_outputs:
            target = project_root / rel_path
            if target.exists():
                existing_count += 1
                provenance.append(str(target))

        summary = (
            f"Detected {existing_count}/{len(required_outputs)} required outputs under "
            f"{project_root}"
        )
        status = "ok" if existing_count == len(required_outputs) else "warning"
        return MCPEvidence(
            provider="filesystem",
            status=status,
            summary=summary,
            provenance=provenance,
            data={
                "project_root": str(project_root),
                "required_output_count": len(required_outputs),
                "existing_output_count": existing_count,
            },
        )

    def _collect_external_command(
        self,
        provider: str,
        task_packet: dict[str, Any],
        cwd: Path,
    ) -> MCPEvidence:
        env_name = self._provider_env_var(provider)
        command = os.environ.get(env_name, "").strip()
        if not command:
            return MCPEvidence(
                provider=provider,
                status="not_configured",
                summary=f"External MCP not configured. Set {env_name}.",
                provenance=[env_name],
            )

        payload = {
            "provider": provider,
            "task_packet": task_packet,
        }
        try:
            parsed_cmd = shlex.split(command)
            run_result = subprocess.run(
                parsed_cmd,
                input=json.dumps(payload, ensure_ascii=False),
                capture_output=True,
                text=True,
                cwd=str(cwd),
                timeout=self.timeout_seconds,
                check=False,
            )
        except subprocess.TimeoutExpired:
            return MCPEvidence(
                provider=provider,
                status="error",
                summary=f"External MCP command timed out after {self.timeout_seconds}s.",
                provenance=[env_name],
            )
        except OSError as exc:
            return MCPEvidence(
                provider=provider,
                status="error",
                summary=f"External MCP command failed to start: {exc}",
                provenance=[env_name],
            )

        if run_result.returncode != 0:
            stderr = (run_result.stderr or "").strip()
            return MCPEvidence(
                provider=provider,
                status="error",
                summary=f"External MCP command exited with code {run_result.returncode}.",
                provenance=[env_name, stderr] if stderr else [env_name],
            )

        stdout = (run_result.stdout or "").strip()
        if not stdout:
            return MCPEvidence(
                provider=provider,
                status="warning",
                summary="External MCP command returned empty output.",
                provenance=[env_name],
            )

        try:
            parsed = json.loads(stdout)
        except json.JSONDecodeError:
            return MCPEvidence(
                provider=provider,
                status="warning",
                summary="External MCP command returned non-JSON output.",
                provenance=[env_name, stdout[:280]],
            )

        raw_status = str(parsed.get("status", "ok")).strip().lower()
        status = raw_status if raw_status in self.VALID_STATUSES else "warning"
        summary = str(parsed.get("summary", f"External MCP response from {provider}.")).strip()
        raw_provenance = parsed.get("provenance", [])
        if isinstance(raw_provenance, list):
            provenance = [str(item) for item in raw_provenance]
        elif raw_provenance:
            provenance = [str(raw_provenance)]
        else:
            provenance = []
        data = parsed.get("data", {})
        if not isinstance(data, dict):
            data = {"raw_data": data}

        return MCPEvidence(
            provider=provider,
            status=status,
            summary=summary,
            provenance=provenance,
            data=data,
        )

    def _provider_env_var(self, provider: str) -> str:
        key = provider.upper().replace("-", "_")
        return f"{self.env_prefix}{key}_CMD"
