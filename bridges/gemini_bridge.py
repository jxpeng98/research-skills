"""
Gemini CLI Bridge for research-skills.
Wraps Google Gemini CLI (https://github.com/google-gemini/gemini-cli).

Python 3.12+ required.
"""
from __future__ import annotations

import json
from pathlib import Path

from .base_bridge import BaseBridge, BridgeResponse, ModelType, escape_prompt


class GeminiBridge(BaseBridge):
    """Bridge to Google Gemini CLI for code analysis and explanation."""
    
    model_type = ModelType.GEMINI
    
    def __init__(
        self,
        sandbox: bool = False,
        model: str | None = None,
    ):
        """
        Initialize Gemini bridge.
        
        Args:
            sandbox: Run in sandbox mode
            model: Specific model to use (default: gemini default)
        """
        self.sandbox = sandbox
        self.model = model
    
    def build_command(
        self,
        prompt: str,
        cwd: Path,
        session_id: str | None = None,
        **kwargs,
    ) -> list[str]:
        """Build Gemini CLI command."""
        cmd = [
            "gemini",
            "--prompt", escape_prompt(prompt),
            "-o", "stream-json",
        ]
        
        if self.sandbox:
            cmd.append("--sandbox")
        
        if self.model:
            cmd.extend(["--model", self.model])
        
        if session_id:
            cmd.extend(["--resume", session_id])
        
        return cmd
    
    def parse_output(self, lines: list[str]) -> BridgeResponse:
        """Parse Gemini JSON stream output."""
        all_messages: list[dict] = []
        agent_messages = ""
        session_id: str | None = None
        errors: list[str] = []
        
        # Deprecation warning to filter out
        deprecation_msg = (
            "The --prompt (-p) flag has been deprecated and will be removed"
        )
        
        for line in lines:
            try:
                data = json.loads(line)
                all_messages.append(data)
                
                # Extract assistant messages
                msg_type = data.get("type", "")
                msg_role = data.get("role", "")
                
                if msg_type == "message" and msg_role == "assistant":
                    content = data.get("content", "")
                    # Filter out deprecation warnings
                    if deprecation_msg not in content:
                        agent_messages += content
                
                # Extract session ID
                if data.get("session_id"):
                    session_id = data["session_id"]
                    
            except json.JSONDecodeError:
                errors.append(f"JSON decode error: {line[:100]}")
            except Exception as e:
                errors.append(f"Parse error: {e}")
        
        # Build response
        if not session_id:
            return BridgeResponse(
                success=False,
                model="gemini",
                error="Failed to get SESSION_ID from Gemini",
                raw_messages=all_messages,
            )
        
        if not agent_messages:
            return BridgeResponse(
                success=False,
                model="gemini",
                session_id=session_id,
                error="No agent messages received. Gemini may be performing tool calls.",
                raw_messages=all_messages,
            )
        
        return BridgeResponse(
            success=True,
            model="gemini",
            session_id=session_id,
            content=agent_messages,
            error="; ".join(errors) if errors else None,
            raw_messages=all_messages,
        )
