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
from typing import Generator
from enum import Enum


class ModelType(Enum):
    """Supported AI models."""
    CODEX = "codex"
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
    gemini_response: BridgeResponse | None = None
    merged_analysis: str = ""
    confidence: float = 0.0
    recommendations: list[str] = field(default_factory=list)
    
    def to_json(self) -> str:
        """Export as JSON string."""
        data = {
            "mode": self.mode,
            "task_description": self.task_description,
            "confidence": self.confidence,
            "merged_analysis": self.merged_analysis,
            "recommendations": self.recommendations,
        }
        if self.codex_response:
            data["codex"] = asdict(self.codex_response)
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
        
        try:
            cmd = self.build_command(prompt, cwd, **kwargs)
            lines = list(self._run_command(cmd, cwd))
            return self.parse_output(lines)
        except Exception as e:
            return BridgeResponse.from_error(cli_name, f"Execution error: {e}")
    
    def _run_command(self, cmd: list[str], cwd: Path) -> Generator[str, None, None]:
        """Stream command output line by line."""
        env = os.environ.copy()
        
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
        
        output_queue: queue.Queue[str | None] = queue.Queue()
        graceful_delay = 0.3
        
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
        
        thread = threading.Thread(target=read_output)
        thread.start()
        
        while True:
            try:
                line = output_queue.get(timeout=0.5)
                if line is None:
                    break
                yield line
            except queue.Empty:
                if process.poll() is not None and not thread.is_alive():
                    break
        
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
        
        thread.join(timeout=5)
        
        # Drain remaining output
        while not output_queue.empty():
            try:
                line = output_queue.get_nowait()
                if line is not None:
                    yield line
            except queue.Empty:
                break
