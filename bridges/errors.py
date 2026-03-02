from dataclasses import dataclass
from typing import Optional

@dataclass
class ErrorCode:
    code: str
    message: str
    fix_hint: str

class ResearchError(Exception):
    """Base exception for all research-skills CLI errors."""
    def __init__(self, err_def: ErrorCode, detail: str = ""):
        self.err_def = err_def
        self.detail = detail
        full_message = f"[{err_def.code}] {err_def.message}"
        if detail:
            full_message += f"\nDetails: {detail}"
        full_message += f"\n👉 How to Fix: {err_def.fix_hint}"
        super().__init__(full_message)

class EnvError(ResearchError):
    pass

class ConfigError(ResearchError):
    pass

class ExecutionError(ResearchError):
    pass

class MCPError(ResearchError):
    pass

# ==========================================
# Standardized Error Definitions
# ==========================================
ERR_ENV_MISSING_KEY = ErrorCode(
    code="ERR-RS-ENV-001",
    message="Missing or invalid API key.",
    fix_hint="Check your environment variables or .env file. e.g. export ANTHROPIC_API_KEY='sk-...'"
)
ERR_ENV_MISSING_CLI = ErrorCode(
    code="ERR-RS-ENV-002",
    message="Required CLI tool is not installed or not in PATH.",
    fix_hint="Install the required CLI tool (e.g., `npm install -g @anthropic-ai/claude-code`) or disable that runtime."
)

ERR_CFG_INVALID_PROFILE = ErrorCode(
    code="ERR-RS-CFG-001",
    message="Unknown or invalid agent profile specified.",
    fix_hint="Check `--profile` argument. Use `task-run --help` to see available default profiles, or ensure your local JSON file exists."
)
ERR_CFG_MISSING_STANDARD = ErrorCode(
    code="ERR-RS-CFG-002",
    message="Could not read or parse standard YAML contracts.",
    fix_hint="Ensure `standards/research-workflow-contract.yaml` exists and is valid YAML."
)
ERR_CFG_INVALID_TASK = ErrorCode(
    code="ERR-RS-CFG-003",
    message="Unknown Task ID or invalid phase logic requested.",
    fix_hint="Check your `--task-id`. Must match a valid entry in `research-workflow-contract.yaml`."
)

ERR_EXE_PARALLEL_FAIL = ErrorCode(
    code="ERR-RS-EXE-001",
    message="All required parallel agents failed to execute.",
    fix_hint="Check agent logs. Usually caused by context limits or immediate crashes. Try running with `--model claude` (Single mode) for clearer stack traces."
)
ERR_EXE_TIMEOUT = ErrorCode(
    code="ERR-RS-EXE-002",
    message="Subprocess execution timed out.",
    fix_hint="The task took too long. Try processing smaller chunks or increasing hardcoded timeouts."
)

ERR_MCP_PROVIDER_MISSING = ErrorCode(
    code="ERR-RS-MCP-001",
    message="Required MCP provider is not configured.",
    fix_hint="Set the appropriate RESEARCH_MCP_*_CMD environment variable or disable strict MCP mode."
)
ERR_MCP_EXECUTION_FAIL = ErrorCode(
    code="ERR-RS-MCP-002",
    message="MCP provider command failed to execute or returned an error.",
    fix_hint="Run `python3 -m bridges.orchestrator doctor --cwd .` to test MCP commands. Check syntax in your .env file."
)
