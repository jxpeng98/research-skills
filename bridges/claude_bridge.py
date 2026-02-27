"""
Claude CLI Bridge for research-skills.
Wraps Anthropic Claude Code CLI.

Python 3.12+ required.
"""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .base_bridge import BaseBridge, BridgeResponse, ModelType, escape_prompt


class ClaudeBridge(BaseBridge):
    """Bridge to Claude Code CLI for writing, reasoning, and review tasks."""

    model_type = ModelType.CLAUDE

    def __init__(
        self,
        model: str | None = None,
        permission_mode: str | None = None,
        output_format: str = "stream-json",
    ):
        self.model = model
        self.permission_mode = permission_mode
        self.output_format = output_format

    def build_command(
        self,
        prompt: str,
        cwd: Path,
        session_id: str | None = None,
        **kwargs,
    ) -> list[str]:
        cmd = [
            "claude",
            "-p",
            "--output-format",
            self.output_format,
        ]
        if self.model:
            cmd.extend(["--model", self.model])
        if self.permission_mode:
            cmd.extend(["--permission-mode", self.permission_mode])
        if session_id:
            cmd.extend(["--resume", session_id])
        cmd.append(escape_prompt(prompt))
        return cmd

    def parse_output(self, lines: list[str]) -> BridgeResponse:
        all_messages: list[dict] = []
        plain_lines: list[str] = []
        agent_messages: list[str] = []
        session_id: str | None = None
        errors: list[str] = []

        for line in lines:
            try:
                data = json.loads(line)
                if isinstance(data, dict):
                    all_messages.append(data)
                    session_id = session_id or self._extract_session_id(data)
                    agent_messages.extend(self._extract_assistant_messages(data))
                    error_text = self._extract_error_text(data)
                    if error_text:
                        errors.append(error_text)
                else:
                    plain_lines.append(line)
            except json.JSONDecodeError:
                plain_lines.append(line)
            except Exception as exc:
                errors.append(f"Parse error: {exc}")

        content = "".join(agent_messages).strip()
        if not content and plain_lines:
            content = "\n".join(item for item in plain_lines if item.strip()).strip()

        if not content:
            return BridgeResponse(
                success=False,
                model="claude",
                session_id=session_id,
                error=(
                    "; ".join(errors)
                    if errors
                    else "No assistant messages received from Claude CLI."
                ),
                raw_messages=all_messages or None,
            )

        return BridgeResponse(
            success=True,
            model="claude",
            session_id=session_id,
            content=content,
            error="; ".join(errors) if errors else None,
            raw_messages=all_messages or None,
        )

    def _extract_session_id(self, data: dict[str, Any]) -> str | None:
        direct_keys = (
            "session_id",
            "sessionId",
            "conversation_id",
            "conversationId",
            "thread_id",
            "threadId",
        )
        for key in direct_keys:
            value = data.get(key)
            if isinstance(value, str) and value.strip():
                return value.strip()
        for key in ("session", "conversation", "thread"):
            nested = data.get(key)
            if isinstance(nested, dict):
                found = self._extract_session_id(nested)
                if found:
                    return found
        return None

    def _extract_assistant_messages(self, data: dict[str, Any]) -> list[str]:
        texts: list[str] = []

        role = str(data.get("role", "")).strip().lower()
        msg_type = str(data.get("type", "")).strip().lower()
        message = data.get("message")
        is_assistant = role == "assistant" or msg_type in {
            "assistant",
            "assistant_message",
            "assistant-response",
        }

        if isinstance(message, dict):
            msg_role = str(message.get("role", "")).strip().lower()
            if msg_role == "assistant":
                is_assistant = True

        if is_assistant:
            for key in ("content", "text", "delta", "output_text"):
                texts.extend(self._extract_text(data.get(key)))
            if isinstance(message, dict):
                for key in ("content", "text", "delta"):
                    texts.extend(self._extract_text(message.get(key)))
        return texts

    def _extract_text(self, value: Any) -> list[str]:
        if value is None:
            return []
        if isinstance(value, str):
            return [value]
        if isinstance(value, list):
            out: list[str] = []
            for item in value:
                out.extend(self._extract_text(item))
            return out
        if isinstance(value, dict):
            out: list[str] = []
            if isinstance(value.get("text"), str):
                out.append(value["text"])
            for key in ("content", "delta", "message", "output_text"):
                out.extend(self._extract_text(value.get(key)))
            return out
        return []

    def _extract_error_text(self, data: dict[str, Any]) -> str:
        error = data.get("error")
        if isinstance(error, str) and error.strip():
            return error.strip()
        if isinstance(error, dict):
            message = error.get("message")
            if isinstance(message, str) and message.strip():
                return message.strip()
        msg_type = str(data.get("type", "")).strip().lower()
        if "error" in msg_type:
            message = data.get("message")
            if isinstance(message, str) and message.strip():
                return message.strip()
        return ""
