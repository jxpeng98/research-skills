"""
Base Bridge for CLI Model Integration.
Provides common infrastructure for calling external AI CLIs.

Python 3.12+ required.
"""
from __future__ import annotations

import json
import os
import sys
import queue
import subprocess
import threading
import time
import shutil
from abc import ABC, abstractmethod
from dataclasses import dataclass, field, asdict
from pathlib import Path
from enum import Enum


class ModelType(Enum):
    """Supported AI models."""
    CODEX = "codex"
    CLAUDE = "claude"
    GEMINI = "gemini"


@dataclass
class BridgeResponse:
    """Standardized response from model bridges."""
    success: bool
    model: str
    session_id: str | None = None
    content: str = ""
    error: str | None = None
    raw_messages: list[dict] | None = None
    
    def to_json(self) -> str:
        """Export as JSON string."""
        return json.dumps(asdict(self), indent=2, ensure_ascii=False)
    
    @classmethod
    def from_error(cls, model: str, error: str) -> "BridgeResponse":
        """Create error response."""
        return cls(success=False, model=model, error=error)


@dataclass
class CollaborationResult:
    """Result from multi-model collaboration."""
    mode: str
    task_description: str
    codex_response: BridgeResponse | None = None
    claude_response: BridgeResponse | None = None
    gemini_response: BridgeResponse | None = None
    merged_analysis: str = ""
    confidence: float = 0.0
    recommendations: list[str] = field(default_factory=list)
    data: dict = field(default_factory=dict)
    
    def to_json(self) -> str:
        """Export as JSON string."""
        data = {
            "mode": self.mode,
            "task_description": self.task_description,
            "confidence": self.confidence,
            "merged_analysis": self.merged_analysis,
            "recommendations": self.recommendations,
        }
        if self.data:
            data["data"] = self.data
        if self.codex_response:
            data["codex"] = asdict(self.codex_response)
        if self.claude_response:
            data["claude"] = asdict(self.claude_response)
        if self.gemini_response:
            data["gemini"] = asdict(self.gemini_response)
        return json.dumps(data, indent=2, ensure_ascii=False)


def configure_stdio() -> None:
    """Configure stdout/stderr to use UTF-8 encoding."""
    if os.name == "nt":
        for stream in (sys.stdout, sys.stderr):
            if hasattr(stream, "reconfigure"):
                try:
                    stream.reconfigure(encoding="utf-8")
                except (ValueError, OSError):
                    pass


def escape_prompt(prompt: str) -> str:
    """Escape special characters in prompt for shell."""
    if os.name == "nt":
        return prompt.replace('\n', '\\n').replace('\r', '\\r').replace('\t', '\\t')
    return prompt


class BaseBridge(ABC):
    """Abstract base for model bridges."""
    
    DEFAULT_TIMEOUT_SECONDS = 300.0
    DEFAULT_NON_INTERACTIVE = True
    DEFAULT_REQUIRE_API_KEY = False
    READ_POLL_SECONDS = 0.5
    TERMINATE_GRACE_SECONDS = 2.0
    WAIT_EXIT_SECONDS = 10.0
    THREAD_JOIN_SECONDS = 5.0
    AUTH_ENV_BY_MODEL = {
        ModelType.CODEX: "OPENAI_API_KEY",
        ModelType.CLAUDE: "ANTHROPIC_API_KEY",
        ModelType.GEMINI: "GOOGLE_API_KEY",
    }
    AUTH_ENV_CANDIDATES_BY_MODEL = {
        ModelType.CODEX: ("OPENAI_API_KEY",),
        ModelType.CLAUDE: ("ANTHROPIC_API_KEY",),
        ModelType.GEMINI: (
            "GEMINI_API_KEY",
            "GOOGLE_API_KEY",
            "GOOGLE_APPLICATION_CREDENTIALS",
        ),
    }
    model_type: ModelType
    
    @abstractmethod
    def build_command(self, prompt: str, cwd: Path, **kwargs) -> list[str]:
        """Build CLI command for the specific model."""
        ...
    
    @abstractmethod
    def parse_output(self, lines: list[str]) -> BridgeResponse:
        """Parse CLI output into structured response."""
        ...
    
    def execute(self, prompt: str, cwd: Path, **kwargs) -> BridgeResponse:
        """Execute the bridge command and return response."""
        # Check if CLI is available
        cli_name = self.model_type.value
        if not shutil.which(cli_name):
            return BridgeResponse.from_error(
                cli_name, 
                f"{cli_name} CLI not found in PATH. Please install it first."
            )
        
        # Check working directory
        if not cwd.exists():
            return BridgeResponse.from_error(
                cli_name,
                f"Working directory does not exist: {cwd}"
            )
        
        timeout_raw = kwargs.pop("timeout_seconds", self.DEFAULT_TIMEOUT_SECONDS)
        non_interactive = bool(
            kwargs.pop("non_interactive", self.DEFAULT_NON_INTERACTIVE)
        )
        require_api_key = bool(
            kwargs.pop("require_api_key", self.DEFAULT_REQUIRE_API_KEY)
        )
        try:
            timeout_seconds = self._normalize_timeout_seconds(timeout_raw)
        except ValueError as exc:
            return BridgeResponse.from_error(cli_name, str(exc))
        if require_api_key:
            auth_candidates = self.AUTH_ENV_CANDIDATES_BY_MODEL.get(
                self.model_type,
                (),
            )
            if auth_candidates and not any(
                os.environ.get(env_name, "").strip() for env_name in auth_candidates
            ):
                return BridgeResponse.from_error(
                    cli_name,
                    "Non-interactive execution requires one of: "
                    + ", ".join(auth_candidates),
                )

        try:
            cmd = self.build_command(prompt, cwd, **kwargs)
            lines, timed_out = self._run_command(
                cmd,
                cwd,
                timeout_seconds=timeout_seconds,
                non_interactive=non_interactive,
            )
            response = self.parse_output(lines)
            if timed_out:
                timeout_text = f"{cli_name} CLI timed out after {int(timeout_seconds)}s."
                if response.success:
                    response.success = False
                response.error = (
                    timeout_text if not response.error else f"{response.error}; {timeout_text}"
                )
            return response
        except KeyboardInterrupt:
            return BridgeResponse.from_error(cli_name, f"{cli_name} CLI execution was interrupted.")
        except Exception as e:
            return BridgeResponse.from_error(cli_name, f"Execution error: {e}")
    
    def _normalize_timeout_seconds(self, value: object) -> float:
        try:
            timeout = float(value)
        except (TypeError, ValueError) as exc:
            raise ValueError("timeout_seconds must be a positive number.") from exc
        if timeout <= 0:
            raise ValueError("timeout_seconds must be greater than 0.")
        return timeout

    def _terminate_process(self, process: subprocess.Popen[str]) -> None:
        if process.poll() is not None:
            return
        process.terminate()
        try:
            process.wait(timeout=self.TERMINATE_GRACE_SECONDS)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()

    def _run_command(
        self,
        cmd: list[str],
        cwd: Path,
        timeout_seconds: float,
        non_interactive: bool,
    ) -> tuple[list[str], bool]:
        """Run command with hard timeout and interruption handling."""
        env = os.environ.copy()
        if non_interactive:
            env.setdefault("CI", "1")
            env.setdefault("NO_COLOR", "1")
            env.setdefault("FORCE_COLOR", "0")
            env.setdefault("TERM", "dumb")
        
        process = subprocess.Popen(
            cmd,
            shell=False,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            universal_newlines=True,
            encoding="utf-8",
            errors="replace",
            cwd=str(cwd),
            env=env,
        )
        
        lines: list[str] = []
        output_queue: queue.Queue[str | None] = queue.Queue()
        graceful_delay = 0.3
        timed_out = False
        deadline = time.monotonic() + timeout_seconds
        
        def is_completed(line: str) -> bool:
            """Check if turn is completed based on JSON output."""
            try:
                data = json.loads(line)
                return data.get("type") == "turn.completed"
            except (json.JSONDecodeError, AttributeError, TypeError):
                return False
        
        def read_output() -> None:
            if process.stdout:
                for line in iter(process.stdout.readline, ""):
                    stripped = line.strip()
                    if stripped:
                        output_queue.put(stripped)
                    if is_completed(stripped):
                        time.sleep(graceful_delay)
                        process.terminate()
                        break
                process.stdout.close()
            output_queue.put(None)
        
        thread = threading.Thread(target=read_output, daemon=True)
        thread.start()
        
        try:
            while True:
                now = time.monotonic()
                remaining = deadline - now
                if remaining <= 0:
                    timed_out = True
                    self._terminate_process(process)
                    break
                try:
                    line = output_queue.get(
                        timeout=min(self.READ_POLL_SECONDS, max(0.05, remaining))
                    )
                    if line is None:
                        break
                    lines.append(line)
                except queue.Empty:
                    if process.poll() is not None and not thread.is_alive():
                        break
        except KeyboardInterrupt:
            self._terminate_process(process)
            raise
        finally:
            try:
                process.wait(timeout=self.WAIT_EXIT_SECONDS)
            except subprocess.TimeoutExpired:
                self._terminate_process(process)
            thread.join(timeout=self.THREAD_JOIN_SECONDS)
        
        # Drain remaining output
        while not output_queue.empty():
            try:
                line = output_queue.get_nowait()
                if line is not None:
                    lines.append(line)
            except queue.Empty:
                break
        return lines, timed_out
