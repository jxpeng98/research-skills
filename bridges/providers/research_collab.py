from __future__ import annotations

import json
import os
import shutil
import subprocess
import threading
from dataclasses import asdict
from datetime import datetime, timezone
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib import error as urllib_error
from urllib import request as urllib_request

from bridges.base_bridge import BridgeResponse
from bridges.command_runtime import split_command
from bridges.gemini_bridge import GeminiBridge


BROKER_URL_ENV = "RESEARCH_GEMINI_BROKER_URL"
BROKER_TOKEN_ENV = "RESEARCH_GEMINI_BROKER_TOKEN"
BROKER_BACKEND_CMD_ENV = "RESEARCH_GEMINI_BROKER_BACKEND_CMD"
BROKER_PROTOCOL_VERSION = "research-gemini-broker/v1"
DEFAULT_BROKER_TIMEOUT_SECONDS = 20.0
DEFAULT_BIND_HOST = "127.0.0.1"
DEFAULT_BIND_PORT = 8767


def gemini_cached_auth_files() -> list[Path]:
    gemini_home = Path(os.environ.get("GEMINI_HOME", "~/.gemini")).expanduser()
    candidates = [
        gemini_home / "oauth_creds.json",
        gemini_home / "google_accounts.json",
    ]
    return [path for path in candidates if path.exists()]


def gemini_noninteractive_auth_status() -> tuple[bool, str]:
    gemini_api_key = os.environ.get("GEMINI_API_KEY", "").strip()
    if gemini_api_key:
        return True, "configured via GEMINI_API_KEY"

    use_vertex = os.environ.get("GOOGLE_GENAI_USE_VERTEXAI", "").strip().lower() in {
        "1",
        "true",
        "yes",
        "on",
    }
    google_api_key = os.environ.get("GOOGLE_API_KEY", "").strip()
    if use_vertex and google_api_key:
        missing = [
            name
            for name in ("GOOGLE_CLOUD_PROJECT", "GOOGLE_CLOUD_LOCATION")
            if not os.environ.get(name, "").strip()
        ]
        if missing:
            return (
                False,
                "Vertex AI auth is partially configured: GOOGLE_API_KEY is set, but missing "
                + ", ".join(missing),
            )
        return True, "configured via Vertex AI API key (GOOGLE_API_KEY)"

    google_application_credentials = os.environ.get(
        "GOOGLE_APPLICATION_CREDENTIALS",
        "",
    ).strip()
    if use_vertex and google_application_credentials:
        cred_path = Path(google_application_credentials).expanduser()
        if not cred_path.exists():
            return (
                False,
                "GOOGLE_APPLICATION_CREDENTIALS is set, but the file does not exist: "
                + str(cred_path),
            )
        missing = [
            name
            for name in ("GOOGLE_CLOUD_PROJECT", "GOOGLE_CLOUD_LOCATION")
            if not os.environ.get(name, "").strip()
        ]
        if missing:
            return (
                False,
                "Vertex AI ADC is partially configured: GOOGLE_APPLICATION_CREDENTIALS is set, "
                "but missing " + ", ".join(missing),
            )
        return True, "configured via Vertex AI ADC (GOOGLE_APPLICATION_CREDENTIALS)"

    cached_files = gemini_cached_auth_files()
    if cached_files:
        cache_list = ", ".join(str(path) for path in cached_files)
        return (
            False,
            "cached Gemini OAuth credentials detected at "
            f"{cache_list}, but Gemini CLI cached login is unreliable in "
            "non-interactive Python subprocesses; prefer GEMINI_API_KEY or Vertex env auth.",
        )

    return (
        False,
        "no reliable non-interactive Gemini auth configured; set GEMINI_API_KEY, "
        "or configure Vertex auth with GOOGLE_GENAI_USE_VERTEXAI plus "
        "GOOGLE_API_KEY/GOOGLE_APPLICATION_CREDENTIALS and project/location env vars.",
    )


class GeminiBrokerClientError(RuntimeError):
    pass


class GeminiBrokerClient:
    def __init__(
        self,
        base_url: str,
        *,
        token: str | None = None,
        timeout_seconds: float = DEFAULT_BROKER_TIMEOUT_SECONDS,
    ) -> None:
        self.base_url = _normalize_broker_url(base_url)
        self.token = token.strip() if token else None
        self.timeout_seconds = timeout_seconds

    def health(self) -> dict[str, Any]:
        return self._request("GET", "/health", None)

    def prompt(
        self,
        *,
        prompt: str,
        cwd: Path,
        runtime_options: dict[str, Any] | None = None,
        reset_session: bool = False,
    ) -> dict[str, Any]:
        payload = {
            "prompt": prompt,
            "cwd": str(cwd),
            "runtime_options": dict(runtime_options or {}),
            "reset_session": bool(reset_session),
        }
        return self._request("POST", "/prompt", payload)

    def reset(self, *, reason: str | None = None) -> dict[str, Any]:
        payload = {}
        if reason:
            payload["reason"] = reason
        return self._request("POST", "/reset", payload)

    def _request(
        self,
        method: str,
        path: str,
        payload: dict[str, Any] | None,
    ) -> dict[str, Any]:
        data = None
        headers = {"Accept": "application/json"}
        if payload is not None:
            data = json.dumps(payload, ensure_ascii=False).encode("utf-8")
            headers["Content-Type"] = "application/json"
        if self.token:
            headers["Authorization"] = f"Bearer {self.token}"

        req = urllib_request.Request(
            self.base_url + path,
            data=data,
            headers=headers,
            method=method,
        )
        try:
            with urllib_request.urlopen(req, timeout=self.timeout_seconds) as resp:
                body = resp.read().decode("utf-8")
        except urllib_error.HTTPError as exc:
            detail = exc.read().decode("utf-8", errors="replace")
            raise GeminiBrokerClientError(
                f"Gemini broker HTTP {exc.code}: {detail or exc.reason}"
            ) from exc
        except urllib_error.URLError as exc:
            raise GeminiBrokerClientError(
                f"Gemini broker request failed: {exc.reason}"
            ) from exc

        try:
            parsed = json.loads(body)
        except json.JSONDecodeError as exc:
            raise GeminiBrokerClientError(
                f"Gemini broker returned non-JSON output: {body[:240]}"
            ) from exc
        if not isinstance(parsed, dict):
            raise GeminiBrokerClientError("Gemini broker returned an invalid JSON payload.")
        return parsed


def broker_client_from_env(
    *,
    timeout_seconds: float = DEFAULT_BROKER_TIMEOUT_SECONDS,
) -> GeminiBrokerClient | None:
    broker_url = os.environ.get(BROKER_URL_ENV, "").strip()
    if not broker_url:
        return None
    broker_token = os.environ.get(BROKER_TOKEN_ENV, "").strip() or None
    return GeminiBrokerClient(
        broker_url,
        token=broker_token,
        timeout_seconds=timeout_seconds,
    )


def broker_status_from_env(
    *,
    timeout_seconds: float = DEFAULT_BROKER_TIMEOUT_SECONDS,
) -> dict[str, Any]:
    client = broker_client_from_env(timeout_seconds=timeout_seconds)
    if client is None:
        return {
            "configured": False,
            "ok": False,
            "detail": f"{BROKER_URL_ENV} not configured",
            "data": {},
        }
    try:
        payload = client.health()
    except GeminiBrokerClientError as exc:
        return {
            "configured": True,
            "ok": False,
            "detail": str(exc),
            "data": {},
        }

    data = payload.get("data", {})
    if not isinstance(data, dict):
        data = {}
    ready = bool(data.get("ready", payload.get("status") == "ok"))
    summary = str(payload.get("summary", "Gemini broker responded.")).strip()
    return {
        "configured": True,
        "ok": ready,
        "detail": summary,
        "data": data,
        "payload": payload,
    }


def bridge_response_from_broker_payload(payload: dict[str, Any]) -> BridgeResponse:
    data = payload.get("data", {})
    if not isinstance(data, dict):
        data = {}
    bridge_payload = data.get("bridge_response", payload)
    if not isinstance(bridge_payload, dict):
        return BridgeResponse.from_error("gemini", "Gemini broker returned an invalid bridge payload.")

    raw_messages = bridge_payload.get("raw_messages")
    if raw_messages is not None and not isinstance(raw_messages, list):
        raw_messages = None
    return BridgeResponse(
        success=bool(bridge_payload.get("success", False)),
        model=str(bridge_payload.get("model", "gemini")),
        session_id=_clean_optional_text(bridge_payload.get("session_id")),
        content=str(bridge_payload.get("content", "")),
        error=_clean_optional_text(bridge_payload.get("error")),
        raw_messages=raw_messages,
    )


class GeminiBrokerBackend:
    name = "base"

    def health(self) -> dict[str, Any]:
        raise NotImplementedError

    def prompt(
        self,
        *,
        prompt: str,
        cwd: Path,
        session_id: str | None = None,
        runtime_options: dict[str, Any] | None = None,
    ) -> BridgeResponse:
        raise NotImplementedError

    def reset(self) -> dict[str, Any]:
        return {
            "status": "ok",
            "summary": f"{self.name} backend reset.",
            "data": {"ready": True},
        }


class GeminiCLIBackend(GeminiBrokerBackend):
    name = "gemini-cli"

    def __init__(self, *, default_model: str | None = None) -> None:
        self.default_model = default_model

    def health(self) -> dict[str, Any]:
        cli_path = shutil.which("gemini")
        if not cli_path:
            return {
                "status": "error",
                "summary": "gemini CLI not found in PATH.",
                "data": {"backend": self.name, "ready": False},
            }

        auth_ok, auth_detail = gemini_noninteractive_auth_status()
        provenance = [cli_path]
        provenance.extend(str(path) for path in gemini_cached_auth_files())
        return {
            "status": "ok" if auth_ok else "warning",
            "summary": (
                "Gemini CLI backend ready for non-interactive broker calls."
                if auth_ok
                else auth_detail
            ),
            "provenance": provenance[:6],
            "data": {
                "backend": self.name,
                "ready": auth_ok,
                "cli_path": cli_path,
                "auth_ready": auth_ok,
                "auth_status": auth_detail,
                "default_model": self.default_model or "",
            },
        }

    def prompt(
        self,
        *,
        prompt: str,
        cwd: Path,
        session_id: str | None = None,
        runtime_options: dict[str, Any] | None = None,
    ) -> BridgeResponse:
        options = dict(runtime_options or {})
        sandbox = bool(options.pop("sandbox", False))
        model = _clean_optional_text(options.pop("model", self.default_model))
        if session_id:
            options["session_id"] = session_id
        options.setdefault("non_interactive", True)
        bridge = GeminiBridge(sandbox=sandbox, model=model)
        return bridge.execute(prompt, cwd, **options)


class CommandJSONBackend(GeminiBrokerBackend):
    name = "command-json"

    def __init__(
        self,
        command: str,
        *,
        timeout_seconds: float = DEFAULT_BROKER_TIMEOUT_SECONDS,
    ) -> None:
        self.command = command.strip()
        self.timeout_seconds = timeout_seconds

    def health(self) -> dict[str, Any]:
        parsed = self._invoke({"mode": "health"}, cwd=Path.cwd())
        data = parsed.get("data", {})
        if not isinstance(data, dict):
            data = {}
        data.setdefault("backend", self.name)
        data.setdefault("command", self.command)
        data.setdefault("ready", parsed.get("status") == "ok")
        parsed["data"] = data
        return parsed

    def prompt(
        self,
        *,
        prompt: str,
        cwd: Path,
        session_id: str | None = None,
        runtime_options: dict[str, Any] | None = None,
    ) -> BridgeResponse:
        payload = {
            "mode": "prompt",
            "prompt": prompt,
            "cwd": str(cwd),
            "session_id": session_id,
            "runtime_options": dict(runtime_options or {}),
        }
        parsed = self._invoke(payload, cwd=cwd)
        return bridge_response_from_broker_payload(parsed)

    def reset(self) -> dict[str, Any]:
        return self._invoke({"mode": "reset"}, cwd=Path.cwd())

    def _invoke(self, payload: dict[str, Any], *, cwd: Path) -> dict[str, Any]:
        try:
            run_result = subprocess.run(
                split_command(self.command),
                input=json.dumps(payload, ensure_ascii=False),
                capture_output=True,
                text=True,
                cwd=str(cwd),
                timeout=self.timeout_seconds,
                check=False,
            )
        except subprocess.TimeoutExpired as exc:
            return {
                "status": "error",
                "summary": f"Broker backend command timed out after {self.timeout_seconds}s.",
                "data": {
                    "backend": self.name,
                    "ready": False,
                    "command": self.command,
                    "error": str(exc),
                },
            }
        except OSError as exc:
            return {
                "status": "error",
                "summary": f"Broker backend command failed to start: {exc}",
                "data": {
                    "backend": self.name,
                    "ready": False,
                    "command": self.command,
                    "error": str(exc),
                },
            }

        stdout = (run_result.stdout or "").strip()
        stderr = (run_result.stderr or "").strip()
        if run_result.returncode != 0:
            return {
                "status": "error",
                "summary": f"Broker backend command exited with code {run_result.returncode}.",
                "provenance": [stderr] if stderr else [],
                "data": {
                    "backend": self.name,
                    "ready": False,
                    "command": self.command,
                },
            }
        if not stdout:
            return {
                "status": "error",
                "summary": "Broker backend command returned empty output.",
                "data": {
                    "backend": self.name,
                    "ready": False,
                    "command": self.command,
                },
            }
        try:
            parsed = json.loads(stdout)
        except json.JSONDecodeError:
            return {
                "status": "error",
                "summary": "Broker backend command returned non-JSON output.",
                "provenance": [stdout[:240]],
                "data": {
                    "backend": self.name,
                    "ready": False,
                    "command": self.command,
                },
            }
        if not isinstance(parsed, dict):
            return {
                "status": "error",
                "summary": "Broker backend command returned an invalid payload.",
                "data": {
                    "backend": self.name,
                    "ready": False,
                    "command": self.command,
                },
            }
        return parsed


class GeminiSessionBroker:
    def __init__(
        self,
        backend: GeminiBrokerBackend | None = None,
        *,
        token: str | None = None,
    ) -> None:
        self.backend = backend or build_default_broker_backend()
        self.token = token.strip() if token else None
        self.started_at = _utcnow()
        self._lock = threading.RLock()
        self._session_id: str | None = None
        self._prompt_count = 0
        self._error_count = 0
        self._auth_lost = False
        self._last_error = ""
        self._last_cwd = ""
        self._last_activity_at = self.started_at

    def is_authorized(self, auth_header: str | None) -> bool:
        if not self.token:
            return True
        expected = f"Bearer {self.token}"
        return (auth_header or "").strip() == expected

    def health(self) -> dict[str, Any]:
        backend_health = self.backend.health()
        data = backend_health.get("data", {})
        if not isinstance(data, dict):
            data = {}
        backend_ready = bool(data.get("ready", backend_health.get("status") == "ok"))
        with self._lock:
            ready = backend_ready and not self._auth_lost
            session_id = self._session_id or ""
            prompt_count = self._prompt_count
            error_count = self._error_count
            auth_lost = self._auth_lost
            last_error = self._last_error
            last_cwd = self._last_cwd
            last_activity_at = self._last_activity_at
        summary = str(backend_health.get("summary", "Gemini broker ready.")).strip()
        if auth_lost:
            summary = (
                "Broker marked Gemini auth as lost after the last prompt failure; reset or re-authenticate before reuse."
            )
        status = "ok" if ready else "warning"
        return {
            "status": status,
            "summary": summary,
            "provenance": list(backend_health.get("provenance", []))[:8],
            "data": {
                **data,
                "protocol_version": BROKER_PROTOCOL_VERSION,
                "backend": data.get("backend", self.backend.name),
                "ready": ready,
                "session_id": session_id,
                "prompt_count": prompt_count,
                "error_count": error_count,
                "auth_lost": auth_lost,
                "last_error": last_error,
                "last_cwd": last_cwd,
                "started_at": self.started_at,
                "last_activity_at": last_activity_at,
            },
        }

    def prompt(self, payload: dict[str, Any]) -> dict[str, Any]:
        prompt = str(payload.get("prompt", "")).strip()
        if not prompt:
            return {
                "status": "error",
                "summary": "Missing prompt.",
                "data": {"backend": self.backend.name, "ready": False},
            }

        cwd = Path(str(payload.get("cwd") or ".")).expanduser()
        if not cwd.exists():
            return {
                "status": "error",
                "summary": f"Working directory does not exist: {cwd}",
                "data": {"backend": self.backend.name, "ready": False},
            }
        runtime_options = payload.get("runtime_options", {})
        if not isinstance(runtime_options, dict):
            runtime_options = {}
        reset_session = bool(payload.get("reset_session", False))

        with self._lock:
            session_id = None if reset_session else self._session_id

        response = self.backend.prompt(
            prompt=prompt,
            cwd=cwd,
            session_id=session_id,
            runtime_options=runtime_options,
        )

        with self._lock:
            self._prompt_count += 1
            self._last_cwd = str(cwd)
            self._last_activity_at = _utcnow()
            if response.success and response.session_id:
                self._session_id = response.session_id
                self._auth_lost = False
                self._last_error = ""
            else:
                self._error_count += 1
                self._last_error = response.error or "Gemini broker prompt failed."
                if _looks_like_auth_loss(response.error):
                    self._auth_lost = True

        summary = (
            f"Broker prompt completed via {self.backend.name}."
            if response.success
            else f"Broker prompt failed via {self.backend.name}: {response.error or 'unknown error'}"
        )
        return {
            "status": "ok" if response.success else "error",
            "summary": summary,
            "data": {
                "backend": self.backend.name,
                "ready": response.success,
                "session_id": response.session_id or self._session_id or "",
                "prompt_count": self._prompt_count,
                "auth_lost": self._auth_lost,
                "bridge_response": asdict(response),
            },
        }

    def reset(self, payload: dict[str, Any] | None = None) -> dict[str, Any]:
        backend_reset = self.backend.reset()
        reason = ""
        if isinstance(payload, dict):
            reason = str(payload.get("reason", "")).strip()
        with self._lock:
            self._session_id = None
            self._auth_lost = False
            self._last_error = ""
            self._last_activity_at = _utcnow()
        summary = str(backend_reset.get("summary", "Gemini broker session state reset.")).strip()
        if reason:
            summary += f" Reason: {reason}."
        data = backend_reset.get("data", {})
        if not isinstance(data, dict):
            data = {}
        return {
            "status": str(backend_reset.get("status", "ok")),
            "summary": summary,
            "provenance": list(backend_reset.get("provenance", []))[:8],
            "data": {
                **data,
                "backend": data.get("backend", self.backend.name),
                "ready": True,
                "session_id": "",
                "auth_lost": False,
            },
        }


class ResearchCollabHTTPServer(ThreadingHTTPServer):
    daemon_threads = True
    allow_reuse_address = True

    def __init__(
        self,
        server_address: tuple[str, int],
        broker: GeminiSessionBroker,
    ) -> None:
        super().__init__(server_address, _ResearchCollabRequestHandler)
        self.broker = broker


class _ResearchCollabRequestHandler(BaseHTTPRequestHandler):
    server_version = "ResearchCollabBroker/1.0"

    def do_GET(self) -> None:  # noqa: N802
        if self.path != "/health":
            self._write_json(
                HTTPStatus.NOT_FOUND,
                {"status": "error", "summary": f"Unknown path: {self.path}"},
            )
            return
        if not self.server.broker.is_authorized(self.headers.get("Authorization")):
            self._write_json(
                HTTPStatus.UNAUTHORIZED,
                {"status": "error", "summary": "Unauthorized"},
            )
            return
        self._write_json(HTTPStatus.OK, self.server.broker.health())

    def do_POST(self) -> None:  # noqa: N802
        if not self.server.broker.is_authorized(self.headers.get("Authorization")):
            self._write_json(
                HTTPStatus.UNAUTHORIZED,
                {"status": "error", "summary": "Unauthorized"},
            )
            return
        payload = self._read_json_body()
        if payload is None:
            self._write_json(
                HTTPStatus.BAD_REQUEST,
                {"status": "error", "summary": "Invalid JSON request body."},
            )
            return
        if self.path == "/prompt":
            response = self.server.broker.prompt(payload)
            code = HTTPStatus.OK if response.get("status") == "ok" else HTTPStatus.BAD_REQUEST
            self._write_json(code, response)
            return
        if self.path == "/reset":
            self._write_json(HTTPStatus.OK, self.server.broker.reset(payload))
            return
        self._write_json(
            HTTPStatus.NOT_FOUND,
            {"status": "error", "summary": f"Unknown path: {self.path}"},
        )

    def log_message(self, format: str, *args: Any) -> None:
        return

    def _read_json_body(self) -> dict[str, Any] | None:
        length = int(self.headers.get("Content-Length", "0") or "0")
        if length <= 0:
            return {}
        raw = self.rfile.read(length)
        try:
            parsed = json.loads(raw.decode("utf-8"))
        except json.JSONDecodeError:
            return None
        return parsed if isinstance(parsed, dict) else None

    def _write_json(self, status: HTTPStatus, payload: dict[str, Any]) -> None:
        body = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def build_default_broker_backend() -> GeminiBrokerBackend:
    backend_cmd = os.environ.get(BROKER_BACKEND_CMD_ENV, "").strip()
    if backend_cmd:
        return CommandJSONBackend(backend_cmd)
    return GeminiCLIBackend()


def create_broker_server(
    *,
    host: str = DEFAULT_BIND_HOST,
    port: int = DEFAULT_BIND_PORT,
    token: str | None = None,
    backend: GeminiBrokerBackend | None = None,
) -> ResearchCollabHTTPServer:
    broker = GeminiSessionBroker(backend=backend, token=token)
    return ResearchCollabHTTPServer((host, port), broker)


def _normalize_broker_url(value: str) -> str:
    normalized = value.strip().rstrip("/")
    if normalized.startswith("http://") or normalized.startswith("https://"):
        return normalized
    return f"http://{normalized}"


def _clean_optional_text(value: Any) -> str | None:
    text = str(value).strip() if value is not None else ""
    return text or None


def _utcnow() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def _looks_like_auth_loss(error: str | None) -> bool:
    text = str(error or "").lower()
    if not text:
        return False
    return any(
        marker in text
        for marker in (
            "authentication page",
            "browser authentication",
            "sign in",
            "oauth",
            "auth",
        )
    )
