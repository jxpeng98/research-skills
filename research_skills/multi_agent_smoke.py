from __future__ import annotations

import argparse
import json
import os
import socket
import subprocess
import sys
import time
import shutil
from contextlib import contextmanager
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from bridges.providers.research_collab import (
    BROKER_URL_ENV,
    GEMINI_TRANSPORT_ENV,
    broker_status_from_env,
    gemini_noninteractive_auth_status,
)


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_TIMEOUT_SECONDS = 60.0
DEFAULT_BROKER_STARTUP_SECONDS = 20.0
PASS = "PASS"
WARN = "WARN"
FAIL = "FAIL"
SKIP = "SKIP"


@dataclass
class SmokeCaseResult:
    name: str
    status: str
    detail: str
    duration_seconds: float
    data: dict[str, Any] = field(default_factory=dict)


@dataclass
class SmokeReport:
    generated_at: str
    cwd: str
    topic: str
    transport: str
    codex_required: bool
    gemini_required: bool
    started_broker: bool
    broker_url: str
    cases: list[SmokeCaseResult] = field(default_factory=list)
    environment: dict[str, Any] = field(default_factory=dict)
    outputs: dict[str, str] = field(default_factory=dict)

    @property
    def overall_status(self) -> str:
        return overall_status_from_cases(self.cases)

    def to_dict(self) -> dict[str, Any]:
        return {
            "generated_at": self.generated_at,
            "cwd": self.cwd,
            "topic": self.topic,
            "transport": self.transport,
            "codex_required": self.codex_required,
            "gemini_required": self.gemini_required,
            "started_broker": self.started_broker,
            "broker_url": self.broker_url,
            "overall_status": self.overall_status,
            "environment": self.environment,
            "outputs": self.outputs,
            "cases": [asdict(item) for item in self.cases],
        }


@dataclass
class BrokerProcessHandle:
    process: subprocess.Popen[str]
    log_path: Path
    url: str
    _log_file: Any

    def stop(self) -> None:
        if self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=2.0)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=2.0)
        self._log_file.close()


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run a local Codex-first, Gemini-assisted multi-agent smoke harness.",
    )
    parser.add_argument("--cwd", default=str(REPO_ROOT), help="Target working directory.")
    parser.add_argument("--topic", default="multi-agent-smoke", help="Topic label stored in the report.")
    parser.add_argument(
        "--transport",
        choices=["auto", "broker", "direct"],
        default=os.environ.get(GEMINI_TRANSPORT_ENV, "auto") or "auto",
        help="Gemini transport mode for this smoke run.",
    )
    parser.add_argument(
        "--start-broker",
        action="store_true",
        help="Start a local Gemini ACP broker for the smoke run.",
    )
    parser.add_argument(
        "--broker-host",
        default="127.0.0.1",
        help="Broker host when --start-broker is used.",
    )
    parser.add_argument(
        "--broker-port",
        type=int,
        default=0,
        help="Broker port when --start-broker is used. Default picks a free port.",
    )
    parser.add_argument(
        "--broker-backend",
        choices=["acp", "cli"],
        default="acp",
        help="Broker backend when --start-broker is used.",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=DEFAULT_TIMEOUT_SECONDS,
        help="Per-runtime timeout for Codex and Gemini prompt probes.",
    )
    parser.add_argument(
        "--run-parallel",
        action="store_true",
        help="Also run an orchestrator parallel smoke with Codex synthesis and Gemini assistance.",
    )
    parser.add_argument(
        "--run-fallback-check",
        action="store_true",
        help="If this script started the broker and transport=auto, stop the broker and verify fallback behavior.",
    )
    parser.add_argument(
        "--strict-gemini",
        action="store_true",
        help="Fail the run when Gemini is unavailable instead of recording a warning/skip.",
    )
    parser.add_argument(
        "--json-report",
        default="",
        help="Optional explicit JSON report path. Defaults to output/test_runtime/.",
    )
    parser.add_argument(
        "--md-report",
        default="",
        help="Optional explicit markdown report path. Defaults to output/test_runtime/.",
    )
    return parser


def overall_status_from_cases(cases: list[SmokeCaseResult]) -> str:
    if any(case.status == FAIL for case in cases):
        return FAIL
    if any(case.status == WARN for case in cases):
        return WARN
    return PASS


def build_default_report_paths(root: Path, generated_at: datetime) -> tuple[Path, Path]:
    stamp = generated_at.strftime("%Y%m%dT%H%M%SZ")
    out_dir = root / "output" / "test_runtime"
    return (
        out_dir / f"multi_agent_smoke_{stamp}.json",
        out_dir / f"multi_agent_smoke_{stamp}.md",
    )


def render_report_markdown(report: SmokeReport) -> str:
    lines = [
        "# Multi-Agent Smoke Report",
        "",
        f"- Generated at: {report.generated_at}",
        f"- Working directory: `{report.cwd}`",
        f"- Topic: `{report.topic}`",
        f"- Gemini transport: `{report.transport}`",
        f"- Overall status: `{report.overall_status}`",
        f"- Broker started by harness: `{str(report.started_broker).lower()}`",
    ]
    if report.broker_url:
        lines.append(f"- Broker URL: `{report.broker_url}`")
    lines.extend(
        [
            "",
            "## Environment",
            "",
            f"- Codex CLI in PATH: `{report.environment.get('codex_cli', False)}`",
            f"- Codex auth ready: `{report.environment.get('codex_auth_ready', False)}`",
            f"- Codex auth detail: `{report.environment.get('codex_auth_detail', '')}`",
            f"- Gemini CLI in PATH: `{report.environment.get('gemini_cli', False)}`",
            f"- OPENAI_API_KEY set: `{report.environment.get('openai_api_key', False)}`",
            f"- Gemini direct auth ready: `{report.environment.get('gemini_direct_auth_ready', False)}`",
            f"- Gemini direct auth detail: `{report.environment.get('gemini_direct_auth_detail', '')}`",
            "",
            "## Cases",
            "",
        ]
    )
    for case in report.cases:
        lines.append(f"- `{case.status}` {case.name}: {case.detail}")
    return "\n".join(lines) + "\n"


@contextmanager
def patched_environ(overrides: dict[str, str | None]):
    previous = {key: os.environ.get(key) for key in overrides}
    try:
        for key, value in overrides.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value
        yield
    finally:
        for key, value in previous.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value


def find_free_port(host: str) -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind((host, 0))
        return int(sock.getsockname()[1])


def codex_auth_status(timeout_seconds: float = 5.0) -> tuple[bool, str]:
    if os.environ.get("OPENAI_API_KEY", "").strip():
        return True, "OPENAI_API_KEY configured"
    if not shutil.which("codex"):
        return False, "codex CLI not found in PATH"
    try:
        completed = subprocess.run(
            ["codex", "login", "status"],
            capture_output=True,
            text=True,
            timeout=timeout_seconds,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        return False, f"codex login status failed: {exc}"
    combined = "\n".join(
        part.strip()
        for part in (completed.stdout or "", completed.stderr or "")
        if part.strip()
    )
    if completed.returncode == 0 and "Logged in" in combined:
        return True, combined.splitlines()[-1]
    detail = combined.splitlines()[-1] if combined.strip() else "codex login status did not confirm authentication"
    return False, detail


def evaluate_doctor_output(
    merged_analysis: str,
    *,
    transport: str,
    codex_required: bool,
    gemini_required: bool,
    codex_auth_ready: bool,
) -> tuple[str, str]:
    text = merged_analysis
    missing: list[str] = []
    if "Working directory" not in text:
        missing.append("Working directory check missing")
    if codex_required:
        if "[OK] CLI codex:" not in text:
            missing.append("CLI codex not OK")
        if not codex_auth_ready and "[OK] Env OPENAI_API_KEY: configured" not in text:
            missing.append("OPENAI_API_KEY not configured")
    if gemini_required:
        if "[OK] Gemini transport:" not in text:
            missing.append("Gemini transport line missing")
        if transport == "broker" and "[OK] Gemini broker:" not in text:
            missing.append("Gemini broker not OK")
        if transport == "direct" and "[OK] Gemini direct auth:" not in text:
            missing.append("Gemini direct auth not OK")
    if missing:
        return FAIL, "; ".join(missing)
    return PASS, "doctor output contained all expected readiness markers"


class MultiAgentSmokeRunner:
    def __init__(self, args: argparse.Namespace) -> None:
        self.args = args
        self.cwd = Path(args.cwd).resolve()
        self.timeout_seconds = float(args.timeout_seconds)
        now = datetime.now(timezone.utc).replace(microsecond=0)
        json_path, md_path = build_default_report_paths(REPO_ROOT, now)
        if args.json_report:
            json_path = Path(args.json_report).resolve()
        if args.md_report:
            md_path = Path(args.md_report).resolve()
        self.report = SmokeReport(
            generated_at=now.isoformat(),
            cwd=str(self.cwd),
            topic=str(args.topic),
            transport=str(args.transport),
            codex_required=True,
            gemini_required=True,
            started_broker=bool(args.start_broker),
            broker_url="",
            outputs={
                "json_report": str(json_path),
                "markdown_report": str(md_path),
            },
        )
        self._broker_handle: BrokerProcessHandle | None = None

    def run(self) -> SmokeReport:
        self._snapshot_environment()
        try:
            if self.args.start_broker:
                self._start_broker()
            else:
                self.report.broker_url = os.environ.get(BROKER_URL_ENV, "").strip()

            self._run_case("doctor", self._case_doctor)
            self._run_case("codex_runtime", self._case_codex_runtime)
            self._run_case("gemini_runtime", self._case_gemini_runtime)
            if self.args.run_parallel:
                self._run_case("parallel_codex_gemini", self._case_parallel_codex_gemini)
            if self.args.run_fallback_check:
                self._run_case("gemini_auto_fallback", self._case_gemini_auto_fallback)
        finally:
            self._write_reports()
            if self._broker_handle is not None:
                self._broker_handle.stop()
                self._broker_handle = None
        return self.report

    def _snapshot_environment(self) -> None:
        codex_auth_ready, codex_auth_detail = codex_auth_status()
        direct_auth_ready, direct_auth_detail = gemini_noninteractive_auth_status()
        self.report.environment = {
            "codex_cli": bool(shutil.which("codex")),
            "codex_auth_ready": codex_auth_ready,
            "codex_auth_detail": codex_auth_detail,
            "gemini_cli": bool(shutil.which("gemini")),
            "openai_api_key": bool(os.environ.get("OPENAI_API_KEY", "").strip()),
            "gemini_api_key": bool(os.environ.get("GEMINI_API_KEY", "").strip()),
            "gemini_direct_auth_ready": direct_auth_ready,
            "gemini_direct_auth_detail": direct_auth_detail,
            "existing_broker_url": os.environ.get(BROKER_URL_ENV, "").strip(),
        }

    def _start_broker(self) -> None:
        broker_host = str(self.args.broker_host)
        broker_port = int(self.args.broker_port or 0) or find_free_port(broker_host)
        broker_url = f"http://{broker_host}:{broker_port}"
        out_dir = Path(self.report.outputs["json_report"]).resolve().parent
        out_dir.mkdir(parents=True, exist_ok=True)
        broker_log = out_dir / f"gemini_broker_{broker_port}.log"
        log_file = broker_log.open("w", encoding="utf-8")
        command = [
            sys.executable,
            str(REPO_ROOT / "scripts" / "gemini_session_broker.py"),
            "--backend",
            str(self.args.broker_backend),
            "--host",
            broker_host,
            "--port",
            str(broker_port),
        ]
        process = subprocess.Popen(
            command,
            cwd=str(REPO_ROOT),
            stdout=log_file,
            stderr=subprocess.STDOUT,
            text=True,
        )
        self._broker_handle = BrokerProcessHandle(
            process=process,
            log_path=broker_log,
            url=broker_url,
            _log_file=log_file,
        )
        self.report.outputs["broker_log"] = str(broker_log)
        deadline = time.time() + DEFAULT_BROKER_STARTUP_SECONDS
        with patched_environ({BROKER_URL_ENV: broker_url, GEMINI_TRANSPORT_ENV: str(self.args.transport)}):
            last_error = ""
            while time.time() < deadline:
                if process.poll() is not None:
                    raise RuntimeError(
                        f"Gemini broker exited early. Inspect {broker_log}."
                    )
                try:
                    status = broker_status_from_env(timeout_seconds=5.0)
                except Exception as exc:
                    last_error = str(exc)
                else:
                    if status.get("ok"):
                        self.report.broker_url = broker_url
                        return
                    last_error = str(status.get("detail", "broker not ready")).strip()
                time.sleep(0.5)
        raise RuntimeError(
            f"Gemini broker did not become healthy within {DEFAULT_BROKER_STARTUP_SECONDS}s. "
            f"Last error: {last_error or 'unknown'} Inspect {broker_log}."
        )

    def _run_case(self, name: str, fn) -> None:
        started_at = time.monotonic()
        try:
            status, detail, data = fn()
        except Exception as exc:
            status = FAIL
            detail = f"{type(exc).__name__}: {exc}"
            data = {}
        duration = time.monotonic() - started_at
        self.report.cases.append(
            SmokeCaseResult(
                name=name,
                status=status,
                detail=detail,
                duration_seconds=round(duration, 3),
                data=data,
            )
        )

    def _case_doctor(self) -> tuple[str, str, dict[str, Any]]:
        orchestrator = self._make_orchestrator()
        env_patch = self._runtime_env_patch()
        with patched_environ(env_patch):
            result = orchestrator.doctor(self.cwd)
        status, detail = evaluate_doctor_output(
            result.merged_analysis,
            transport=self.args.transport,
            codex_required=True,
            gemini_required=True,
            codex_auth_ready=bool(self.report.environment.get("codex_auth_ready")),
        )
        return status, detail, {
            "confidence": result.confidence,
            "merged_analysis": result.merged_analysis,
        }

    def _case_codex_runtime(self) -> tuple[str, str, dict[str, Any]]:
        orchestrator = self._make_orchestrator()
        prompt = "Return only the token CODEX_SMOKE_OK."
        response = orchestrator._execute_runtime_agent(
            "codex",
            prompt,
            self.cwd,
            {"non_interactive": True, "timeout_seconds": self.timeout_seconds},
        )
        if response.success and "CODEX_SMOKE_OK" in response.content:
            return PASS, "Codex runtime returned the smoke token.", {"content": response.content}
        return FAIL, response.error or "Codex runtime did not return the smoke token.", {
            "content": response.content,
            "error": response.error,
        }

    def _case_gemini_runtime(self) -> tuple[str, str, dict[str, Any]]:
        orchestrator = self._make_orchestrator()
        prompt = "Return only the token GEMINI_SMOKE_OK."
        env_patch = self._runtime_env_patch()
        runtime_options = {
            "transport": self.args.transport,
            "non_interactive": True,
            "timeout_seconds": self.timeout_seconds,
            "approval_mode": "default",
        }
        with patched_environ(env_patch):
            preflight_error = orchestrator._runtime_preflight_error(
                "gemini",
                self.cwd,
                runtime_options,
            )
            if preflight_error:
                status = FAIL if self.args.strict_gemini else WARN
                return status, preflight_error, {"preflight_error": preflight_error}
            response = orchestrator._execute_runtime_agent(
                "gemini",
                prompt,
                self.cwd,
                runtime_options,
            )
        if response.success and "GEMINI_SMOKE_OK" in response.content:
            return PASS, "Gemini runtime returned the smoke token.", {
                "content": response.content,
                "session_id": response.session_id or "",
                "raw_messages": response.raw_messages or [],
            }
        status = FAIL if self.args.strict_gemini else WARN
        return status, response.error or "Gemini runtime did not return the smoke token.", {
            "content": response.content,
            "error": response.error,
            "session_id": response.session_id or "",
            "raw_messages": response.raw_messages or [],
        }

    def _case_parallel_codex_gemini(self) -> tuple[str, str, dict[str, Any]]:
        from bridges.orchestrator import CollaborationMode

        orchestrator = self._make_orchestrator()
        env_patch = self._runtime_env_patch()
        with patched_environ(env_patch):
            result = orchestrator.execute(
                mode=CollaborationMode.PARALLEL,
                cwd=self.cwd,
                prompt="Return one short analysis sentence mentioning the token PARALLEL_SMOKE_OK.",
                parallel_summarizer="codex",
                profile_file=self._smoke_profile_file(),
                profile="smoke-codex-gemini",
                summarizer_profile="smoke-codex-gemini",
            )
        codex_ok = bool(result.codex_response and result.codex_response.success)
        gemini_ok = bool(result.gemini_response and result.gemini_response.success)
        if codex_ok and gemini_ok:
            return PASS, "Parallel mode succeeded with Codex synthesis and Gemini participation.", {
                "merged_analysis": result.merged_analysis,
            }
        if codex_ok and not gemini_ok:
            detail = "Parallel mode succeeded for Codex, but Gemini did not participate successfully."
            status = FAIL if self.args.strict_gemini else WARN
            return status, detail, {"merged_analysis": result.merged_analysis}
        return FAIL, "Parallel mode did not complete with Codex as the primary execution path.", {
            "merged_analysis": result.merged_analysis,
        }

    def _case_gemini_auto_fallback(self) -> tuple[str, str, dict[str, Any]]:
        if self.args.transport != "auto":
            return SKIP, "Fallback check only applies to transport=auto.", {}
        if self._broker_handle is None:
            return SKIP, "Fallback check requires --start-broker so the harness can safely stop it.", {}

        self._broker_handle.stop()
        self._broker_handle = None
        orchestrator = self._make_orchestrator()
        env_patch = {BROKER_URL_ENV: self.report.broker_url, GEMINI_TRANSPORT_ENV: "auto"}
        with patched_environ(env_patch):
            direct_ready, direct_detail = gemini_noninteractive_auth_status()
            preflight_error = orchestrator._runtime_preflight_error(
                "gemini",
                self.cwd,
                {"transport": "auto", "non_interactive": True, "timeout_seconds": self.timeout_seconds},
            )
            if direct_ready:
                if preflight_error:
                    return FAIL, f"Auto fallback should have recovered to direct auth, but preflight failed: {preflight_error}", {}
                response = orchestrator._execute_runtime_agent(
                    "gemini",
                    "Return only the token GEMINI_AUTO_FALLBACK_OK.",
                    self.cwd,
                    {"transport": "auto", "non_interactive": True, "timeout_seconds": self.timeout_seconds},
                )
                if response.success and "GEMINI_AUTO_FALLBACK_OK" in response.content:
                    return PASS, "Auto fallback recovered to direct Gemini execution after broker shutdown.", {
                        "content": response.content,
                        "direct_auth_detail": direct_detail,
                    }
                return FAIL, response.error or "Auto fallback did not return the expected token.", {
                    "content": response.content,
                    "direct_auth_detail": direct_detail,
                }
            if preflight_error and ("broker unavailable" in preflight_error.lower() or "configured" in preflight_error.lower()):
                return PASS, "Auto fallback correctly reported broker loss when no direct Gemini auth was available.", {
                    "preflight_error": preflight_error,
                    "direct_auth_detail": direct_detail,
                }
            return WARN, "Auto fallback case did not match the expected no-direct-auth failure shape.", {
                "preflight_error": preflight_error or "",
                "direct_auth_detail": direct_detail,
            }

    def _make_orchestrator(self):
        from bridges.orchestrator import ModelOrchestrator

        return ModelOrchestrator(standards_dir=REPO_ROOT / "standards")

    def _smoke_profile_file(self) -> Path:
        out_dir = Path(self.report.outputs["json_report"]).resolve().parent
        out_dir.mkdir(parents=True, exist_ok=True)
        profile_path = out_dir / "multi_agent_smoke_profile.json"
        payload = {
            "profiles": {
                "smoke-codex-gemini": {
                    "persona": "Smoke test profile for Codex-first local validation.",
                    "analysis_style": "Return short direct answers for smoke validation.",
                    "summary_style": "Return short direct answers for smoke validation.",
                    "runtime_options": {
                        "codex": {
                            "non_interactive": True,
                            "timeout_seconds": self.timeout_seconds,
                        },
                        "gemini": {
                            "transport": self.args.transport,
                            "non_interactive": True,
                            "timeout_seconds": self.timeout_seconds,
                            "approval_mode": "default",
                        },
                    },
                }
            }
        }
        profile_path.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")
        self.report.outputs["profile_file"] = str(profile_path)
        return profile_path

    def _runtime_env_patch(self) -> dict[str, str | None]:
        return {
            GEMINI_TRANSPORT_ENV: str(self.args.transport),
            BROKER_URL_ENV: self.report.broker_url or os.environ.get(BROKER_URL_ENV, "").strip() or None,
        }

    def _write_reports(self) -> None:
        json_path = Path(self.report.outputs["json_report"]).resolve()
        md_path = Path(self.report.outputs["markdown_report"]).resolve()
        json_path.parent.mkdir(parents=True, exist_ok=True)
        md_path.parent.mkdir(parents=True, exist_ok=True)
        json_path.write_text(
            json.dumps(self.report.to_dict(), indent=2, ensure_ascii=False),
            encoding="utf-8",
        )
        md_path.write_text(render_report_markdown(self.report), encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    runner = MultiAgentSmokeRunner(args)
    report = runner.run()
    print(f"[multi-agent-smoke] {report.overall_status}")
    print(f"[multi-agent-smoke] json: {report.outputs['json_report']}")
    print(f"[multi-agent-smoke] md: {report.outputs['markdown_report']}")
    return 0 if report.overall_status != FAIL else 1


if __name__ == "__main__":
    raise SystemExit(main())
