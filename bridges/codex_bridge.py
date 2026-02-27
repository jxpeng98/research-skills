"""
Codex CLI Bridge for research-skills.
Wraps OpenAI Codex CLI (https://github.com/openai/codex).

Python 3.12+ required.
"""
from __future__ import annotations

import json
from pathlib import Path
from typing import Literal

from .base_bridge import BaseBridge, BridgeResponse, ModelType, escape_prompt


SandboxLevel = Literal["read-only", "workspace-write", "danger-full-access"]


class CodexBridge(BaseBridge):
    """Bridge to OpenAI Codex CLI for code generation and analysis."""
    
    model_type = ModelType.CODEX
    
    def __init__(
        self,
        sandbox: SandboxLevel = "read-only",
        skip_git_check: bool = True,
        model: str | None = None,
    ):
        """
        Initialize Codex bridge.
        
        Args:
            sandbox: Sandbox policy (read-only, workspace-write, danger-full-access)
            skip_git_check: Allow running outside git repo
            model: Specific model to use (default: codex default)
        """
        self.sandbox = sandbox
        self.skip_git_check = skip_git_check
        self.model = model
    
    def build_command(
        self,
        prompt: str,
        cwd: Path,
        session_id: str | None = None,
        images: list[Path] | None = None,
        **kwargs,
    ) -> list[str]:
        """Build Codex CLI command."""
        cmd = [
            "codex", "exec",
            "--sandbox", self.sandbox,
            "--cd", str(cwd),
            "--json",
        ]
        
        if self.skip_git_check:
            cmd.append("--skip-git-repo-check")
        
        if self.model:
            cmd.extend(["--model", self.model])
        
        if images:
            cmd.extend(["--image", ",".join(str(p) for p in images)])
        
        if session_id:
            cmd.extend(["resume", session_id])
        
        cmd.extend(["--", escape_prompt(prompt)])
        
        return cmd
    
    def parse_output(self, lines: list[str]) -> BridgeResponse:
        """Parse Codex JSON stream output."""
        all_messages: list[dict] = []
        agent_messages = ""
        thread_id: str | None = None
        errors: list[str] = []
        
        for line in lines:
            try:
                data = json.loads(line)
                all_messages.append(data)
                
                # Extract agent messages
                item = data.get("item", {})
                if item.get("type") == "agent_message":
                    agent_messages += item.get("text", "")
                
                # Extract session/thread ID
                if data.get("thread_id"):
                    thread_id = data["thread_id"]
                
                # Check for errors
                msg_type = data.get("type", "")
                if "fail" in msg_type or "error" in msg_type:
                    error_msg = (
                        data.get("error", {}).get("message", "") or
                        data.get("message", "")
                    )
                    if error_msg and "Reconnecting" not in error_msg:
                        errors.append(error_msg)
                        
            except json.JSONDecodeError:
                errors.append(f"JSON decode error: {line[:100]}")
            except Exception as e:
                errors.append(f"Parse error: {e}")
        
        # Build response
        if not thread_id:
            return BridgeResponse(
                success=False,
                model="codex",
                error="Failed to get SESSION_ID from Codex",
                raw_messages=all_messages,
            )
        
        if not agent_messages:
            return BridgeResponse(
                success=False,
                model="codex",
                session_id=thread_id,
                error="No agent messages received. Try with --return-all-messages.",
                raw_messages=all_messages,
            )
        
        return BridgeResponse(
            success=True,
            model="codex",
            session_id=thread_id,
            content=agent_messages,
            error="; ".join(errors) if errors else None,
            raw_messages=all_messages,
        )
