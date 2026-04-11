from __future__ import annotations

import json
import os
import queue
import shutil
import subprocess
import threading
from collections import deque
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
GEMINI_TRANSPORT_ENV = "RESEARCH_GEMINI_TRANSPORT"
GEMINI_ACP_COMMAND_ENV = "RESEARCH_GEMINI_ACP_CMD"
BROKER_PROTOCOL_VERSION = "research-gemini-broker/v1"
DEFAULT_BROKER_TIMEOUT_SECONDS = 20.0
DEFAULT_ACP_COMMAND = "gemini --acp"
DEFAULT_ACP_PROTOCOL_VERSION = 1
DEFAULT_ACP_MODE = "plan"
DEFAULT_ACP_REQUEST_TIMEOUT_SECONDS = 120.0
DEFAULT_BIND_HOST = "127.0.0.1"
DEFAULT_BIND_PORT = 8767
VALID_GEMINI_TRANSPORTS = {"auto", "broker", "direct"}
GEMINI_TRANSPORT_ALIASES = {
    "auto": "auto",
    "broker": "broker",
    "mcp": "broker",
    "resident": "broker",
    "direct": "direct",
    "cli": "direct",
    "subprocess": "direct",
}


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


def normalize_gemini_transport(value: object) -> str:
    raw = str(value or "").strip().lower()
    if not raw:
        return "auto"
    return GEMINI_TRANSPORT_ALIASES.get(raw, "auto")


def resolve_gemini_transport(
    runtime_options: dict[str, Any] | None = None,
) -> tuple[str, str]:
    options = dict(runtime_options or {})
    option_value = options.get("transport")
    if option_value is not None and str(option_value).strip():
        return normalize_gemini_transport(option_value), "runtime_options.transport"

    env_value = os.environ.get(GEMINI_TRANSPORT_ENV, "").strip()
    if env_value:
        return normalize_gemini_transport(env_value), GEMINI_TRANSPORT_ENV

    return "auto", "default"


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


class GeminiAcpClientError(RuntimeError):
    pass


class _GeminiACPProcessClient:
    def __init__(
        self,
        command: str,
        *,
        startup_timeout_seconds: float = DEFAULT_BROKER_TIMEOUT_SECONDS,
        request_timeout_seconds: float = DEFAULT_ACP_REQUEST_TIMEOUT_SECONDS,
    ) -> None:
        self.command = command.strip() or DEFAULT_ACP_COMMAND
        self.startup_timeout_seconds = startup_timeout_seconds
        self.request_timeout_seconds = request_timeout_seconds
        self._process: subprocess.Popen[str] | None = None
        self._stdout_thread: threading.Thread | None = None
        self._stderr_thread: threading.Thread | None = None
        self._write_lock = threading.RLock()
        self._state_lock = threading.RLock()
        self._request_counter = 0
        self._pending: dict[str, queue.Queue[dict[str, Any]]] = {}
        self._update_session_id: str | None = None
        self._update_buffer: list[dict[str, Any]] | None = None
        self._stderr_tail: deque[str] = deque(maxlen=40)
        self._initialize_result: dict[str, Any] | None = None

    def is_alive(self) -> bool:
        return self._process is not None and self._process.poll() is None

    def stderr_tail(self) -> list[str]:
        with self._state_lock:
            return list(self._stderr_tail)

    def initialize(self) -> dict[str, Any]:
        with self._state_lock:
            if self._initialize_result is not None and self.is_alive():
                return self._initialize_result
        result = self.request(
            "initialize",
            {
                "protocolVersion": DEFAULT_ACP_PROTOCOL_VERSION,
                "clientCapabilities": {
                    "auth": {"terminal": False},
                    "fs": {"readTextFile": False, "writeTextFile": False},
                    "terminal": False,
                },
                "clientInfo": {
                    "name": "research-skills",
                    "version": BROKER_PROTOCOL_VERSION,
                },
            },
            timeout_seconds=self.startup_timeout_seconds,
        )
        with self._state_lock:
            self._initialize_result = result
        return result

    def request(
        self,
        method: str,
        params: dict[str, Any],
        *,
        timeout_seconds: float | None = None,
        update_buffer: list[dict[str, Any]] | None = None,
        update_session_id: str | None = None,
    ) -> dict[str, Any]:
        self._ensure_process_started()
        request_id = self._next_request_id(method)
        response_queue: queue.Queue[dict[str, Any]] = queue.Queue(maxsize=1)
        with self._state_lock:
            self._pending[request_id] = response_queue
            if update_buffer is not None:
                self._update_buffer = update_buffer
                self._update_session_id = str(update_session_id or "").strip() or None

        payload = {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params,
        }
        try:
            self._write_message(payload)
            timeout_value = timeout_seconds or self.request_timeout_seconds
            response = response_queue.get(timeout=timeout_value)
        except queue.Empty as exc:
            self.close()
            raise GeminiAcpClientError(
                f"Gemini ACP request timed out after {timeout_value:.1f}s: {method}"
            ) from exc
        finally:
            with self._state_lock:
                self._pending.pop(request_id, None)
                if update_buffer is not None and self._update_buffer is update_buffer:
                    self._update_buffer = None
                    self._update_session_id = None

        error_payload = response.get("error")
        if isinstance(error_payload, dict):
            message = str(error_payload.get("message", "unknown ACP error")).strip()
            raise GeminiAcpClientError(f"{method}: {message}")
        result = response.get("result", {})
        if not isinstance(result, dict):
            raise GeminiAcpClientError(f"{method}: invalid ACP response payload")
        return result

    def close(self) -> None:
        process = self._process
        if process is not None and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=2.0)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=2.0)
        with self._state_lock:
            self._process = None
            self._stdout_thread = None
            self._stderr_thread = None
            self._initialize_result = None
            pending = list(self._pending.values())
            self._pending.clear()
            self._update_buffer = None
            self._update_session_id = None
        for pending_queue in pending:
            try:
                pending_queue.put_nowait(
                    {
                        "error": {
                            "message": "Gemini ACP process closed before the request completed."
                        }
                    }
                )
            except queue.Full:
                pass

    def _ensure_process_started(self) -> None:
        if self.is_alive():
            return
        with self._state_lock:
            if self.is_alive():
                return
            self._initialize_result = None
            self._stderr_tail.clear()
            try:
                self._process = subprocess.Popen(
                    split_command(self.command),
                    stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    text=True,
                    bufsize=1,
                )
            except OSError as exc:
                raise GeminiAcpClientError(
                    f"failed to start Gemini ACP command '{self.command}': {exc}"
                ) from exc
            self._stdout_thread = threading.Thread(
                target=self._read_stdout_loop,
                name="gemini-acp-stdout",
                daemon=True,
            )
            self._stderr_thread = threading.Thread(
                target=self._read_stderr_loop,
                name="gemini-acp-stderr",
                daemon=True,
            )
            self._stdout_thread.start()
            self._stderr_thread.start()

    def _next_request_id(self, method: str) -> str:
        with self._state_lock:
            self._request_counter += 1
            return f"{method}-{self._request_counter}"

    def _write_message(self, payload: dict[str, Any]) -> None:
        process = self._process
        if process is None or process.stdin is None or process.poll() is not None:
            raise GeminiAcpClientError("Gemini ACP process is not running.")
        line = json.dumps(payload, ensure_ascii=False)
        with self._write_lock:
            try:
                process.stdin.write(line + "\n")
                process.stdin.flush()
            except OSError as exc:
                self.close()
                raise GeminiAcpClientError(
                    f"failed to write to Gemini ACP process: {exc}"
                ) from exc

    def _read_stdout_loop(self) -> None:
        process = self._process
        if process is None or process.stdout is None:
            return
        try:
            for raw_line in process.stdout:
                line = raw_line.strip()
                if not line:
                    continue
                try:
                    message = json.loads(line)
                except json.JSONDecodeError:
                    with self._state_lock:
                        self._stderr_tail.append(f"stdout(non-json): {line[:240]}")
                    continue
                if not isinstance(message, dict):
                    continue
                if "id" in message and ("result" in message or "error" in message):
                    request_id = str(message.get("id"))
                    pending_queue = self._pending.get(request_id)
                    if pending_queue is not None:
                        pending_queue.put(message)
                    continue
                if message.get("method") == "session/update":
                    self._handle_session_update(message.get("params"))
                    continue
                if "id" in message and "method" in message:
                    self._handle_agent_request(message)
        finally:
            self._fail_pending_requests(
                self._process_exit_message("Gemini ACP process exited while waiting for a response.")
            )

    def _read_stderr_loop(self) -> None:
        process = self._process
        if process is None or process.stderr is None:
            return
        for raw_line in process.stderr:
            line = raw_line.strip()
            if not line:
                continue
            with self._state_lock:
                self._stderr_tail.append(line[:400])

    def _handle_session_update(self, params: Any) -> None:
        if not isinstance(params, dict):
            return
        session_id = str(params.get("sessionId", "")).strip()
        normalized = _normalize_acp_update_payload(params)
        with self._state_lock:
            if (
                self._update_buffer is not None
                and self._update_session_id
                and session_id == self._update_session_id
            ):
                self._update_buffer.append(normalized)

    def _handle_agent_request(self, message: dict[str, Any]) -> None:
        request_id = message.get("id")
        if request_id is None:
            return
        method = str(message.get("method", "")).strip()
        if method == "session/request_permission":
            response = {"jsonrpc": "2.0", "id": request_id, "result": {"outcome": "cancelled"}}
        else:
            response = {
                "jsonrpc": "2.0",
                "id": request_id,
                "error": {
                    "code": -32601,
                    "message": f"ACP client capability not implemented for {method}",
                },
            }
        try:
            self._write_message(response)
        except GeminiAcpClientError:
            return

    def _fail_pending_requests(self, message: str) -> None:
        with self._state_lock:
            pending_items = list(self._pending.items())
            self._pending.clear()
            self._process = None
            self._initialize_result = None
            self._update_buffer = None
            self._update_session_id = None
        for _request_id, pending_queue in pending_items:
            try:
                pending_queue.put_nowait({"error": {"message": message}})
            except queue.Full:
                pass

    def _process_exit_message(self, default: str) -> str:
        process = self._process
        if process is None:
            return default
        returncode = process.poll()
        stderr_tail = self.stderr_tail()
        if stderr_tail:
            return f"{default} {' | '.join(stderr_tail[-3:])}"
        if returncode is not None:
            return f"{default} Exit code: {returncode}."
        return default


class GeminiACPBackend(GeminiBrokerBackend):
    name = "gemini-acp-resident"

    def __init__(
        self,
        *,
        command: str | None = None,
        default_mode: str = DEFAULT_ACP_MODE,
        default_model: str | None = None,
        startup_timeout_seconds: float = DEFAULT_BROKER_TIMEOUT_SECONDS,
        request_timeout_seconds: float = DEFAULT_ACP_REQUEST_TIMEOUT_SECONDS,
        client_factory: Any | None = None,
    ) -> None:
        self.command = str(command or os.environ.get(GEMINI_ACP_COMMAND_ENV, DEFAULT_ACP_COMMAND)).strip()
        self.default_mode = default_mode.strip() or DEFAULT_ACP_MODE
        self.default_model = default_model
        self.startup_timeout_seconds = startup_timeout_seconds
        self.request_timeout_seconds = request_timeout_seconds
        self._client_factory = client_factory
        self._lock = threading.RLock()
        self._client: _GeminiACPProcessClient | Any | None = None
        self._initialize_result: dict[str, Any] | None = None
        self._session_id: str | None = None
        self._session_cwd: str = ""
        self._current_mode: str = ""
        self._current_model: str = ""
        self._last_auth_method: str = ""

    def health(self) -> dict[str, Any]:
        command_name = split_command(self.command)[0] if self.command.strip() else "gemini"
        if command_name == "gemini":
            cli_path = shutil.which("gemini")
            if not cli_path:
                return {
                    "status": "error",
                    "summary": "gemini CLI not found in PATH.",
                    "data": {"backend": self.name, "ready": False, "command": self.command},
                }
        else:
            cli_path = shutil.which(command_name) or command_name

        try:
            initialize_result = self._ensure_initialized()
        except GeminiAcpClientError as exc:
            return {
                "status": "warning",
                "summary": f"Gemini ACP backend unavailable: {exc}",
                "provenance": self._stderr_tail(),
                "data": {
                    "backend": self.name,
                    "ready": False,
                    "command": self.command,
                    "cli_path": cli_path,
                },
            }

        ready, auth_status = self._acp_auth_candidate_status(initialize_result)
        with self._lock:
            session_id = self._session_id or ""
            current_mode = self._current_mode or self.default_mode
            current_model = self._current_model or self.default_model or ""
            auth_method = self._last_auth_method
        summary = (
            "Gemini ACP resident backend ready."
            if ready
            else auth_status
        )
        if session_id:
            summary = f"Gemini ACP resident backend ready with active session {session_id}."
        return {
            "status": "ok" if ready else "warning",
            "summary": summary,
            "provenance": [cli_path, *self._stderr_tail()[:4]],
            "data": {
                "backend": self.name,
                "ready": ready,
                "command": self.command,
                "cli_path": cli_path,
                "session_id": session_id,
                "auth_status": auth_status,
                "auth_method": auth_method,
                "current_mode": current_mode,
                "current_model": current_model,
                "auth_candidate": ready,
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
        try:
            with self._lock:
                active_session_id = self._ensure_session(
                    cwd=cwd,
                    requested_session_id=session_id,
                    runtime_options=options,
                )
                session_mode = self._resolve_session_mode(options)
                if session_mode and session_mode != self._current_mode:
                    self._request(
                        "session/set_mode",
                        {"sessionId": active_session_id, "modeId": session_mode},
                    )
                    self._current_mode = session_mode
                session_model = self._resolve_session_model(options)
                if session_model and session_model != self._current_model:
                    self._request(
                        "session/set_model",
                        {"sessionId": active_session_id, "modelId": session_model},
                    )
                    self._current_model = session_model

                updates: list[dict[str, Any]] = []
                self._request(
                    "session/prompt",
                    {
                        "sessionId": active_session_id,
                        "prompt": [{"type": "text", "text": prompt}],
                    },
                    update_buffer=updates,
                    update_session_id=active_session_id,
                )
                content = _render_acp_updates(updates)
                raw_messages = [
                    json.dumps(item, ensure_ascii=False)
                    for item in updates[-30:]
                ]
                return BridgeResponse(
                    success=True,
                    model=self._current_model or self.default_model or "gemini",
                    session_id=active_session_id,
                    content=content,
                    raw_messages=raw_messages or None,
                )
        except GeminiAcpClientError as exc:
            return BridgeResponse.from_error("gemini", str(exc))

    def reset(self) -> dict[str, Any]:
        with self._lock:
            client = self._client
            if client is not None:
                try:
                    client.close()
                except Exception:
                    pass
            self._client = None
            self._initialize_result = None
            self._session_id = None
            self._session_cwd = ""
            self._current_mode = ""
            self._current_model = ""
            self._last_auth_method = ""
        return {
            "status": "ok",
            "summary": "Gemini ACP resident backend reset.",
            "data": {"backend": self.name, "ready": True},
        }

    def _client_instance(self) -> _GeminiACPProcessClient | Any:
        client = self._client
        if client is not None and getattr(client, "is_alive", lambda: True)():
            return client
        factory = self._client_factory
        if factory is None:
            client = _GeminiACPProcessClient(
                self.command,
                startup_timeout_seconds=self.startup_timeout_seconds,
                request_timeout_seconds=self.request_timeout_seconds,
            )
        else:
            client = factory()
        self._client = client
        return client

    def _ensure_initialized(self) -> dict[str, Any]:
        client = self._client_instance()
        initialize_result = client.initialize()
        if not isinstance(initialize_result, dict):
            raise GeminiAcpClientError("Gemini ACP initialize returned an invalid payload.")
        self._initialize_result = initialize_result
        return initialize_result

    def _ensure_session(
        self,
        *,
        cwd: Path,
        requested_session_id: str | None,
        runtime_options: dict[str, Any],
    ) -> str:
        session_id = str(requested_session_id or self._session_id or "").strip()
        target_cwd = str(cwd)
        if session_id and session_id == (self._session_id or "") and self._session_cwd == target_cwd:
            return session_id

        initialize_result = self._ensure_initialized()
        try:
            session_payload = self._request(
                "session/new",
                {"cwd": target_cwd, "mcpServers": []},
            )
        except GeminiAcpClientError as exc:
            if not _looks_like_acp_auth_required(str(exc)):
                raise
            self._authenticate(initialize_result, runtime_options)
            session_payload = self._request(
                "session/new",
                {"cwd": target_cwd, "mcpServers": []},
            )

        session_id = str(session_payload.get("sessionId", "")).strip()
        if not session_id:
            raise GeminiAcpClientError("session/new did not return a sessionId.")
        self._session_id = session_id
        self._session_cwd = target_cwd
        self._current_mode = str(
            ((session_payload.get("modes") or {}) if isinstance(session_payload.get("modes"), dict) else {}).get(
                "currentModeId",
                "",
            )
        ).strip()
        self._current_model = str(
            ((session_payload.get("models") or {}) if isinstance(session_payload.get("models"), dict) else {}).get(
                "currentModelId",
                "",
            )
        ).strip()
        return session_id

    def _authenticate(
        self,
        initialize_result: dict[str, Any],
        runtime_options: dict[str, Any],
    ) -> None:
        auth_method = self._select_auth_method(initialize_result, runtime_options)
        if auth_method is None:
            raise GeminiAcpClientError(
                "Gemini ACP could not determine an auth method. Configure GEMINI_API_KEY, Vertex env auth, or keep a Google login cached under ~/.gemini."
            )
        payload: dict[str, Any] = {"methodId": str(auth_method.get("id", "")).strip()}
        meta = self._auth_meta_for_method(auth_method)
        if meta:
            payload["_meta"] = meta
        self._request("authenticate", payload)
        self._last_auth_method = str(auth_method.get("name", "")).strip()

    def _select_auth_method(
        self,
        initialize_result: dict[str, Any],
        runtime_options: dict[str, Any],
    ) -> dict[str, Any] | None:
        raw_methods = initialize_result.get("authMethods", [])
        methods = [item for item in raw_methods if isinstance(item, dict)]
        if not methods:
            return None

        preferred = str(runtime_options.get("auth_method", "")).strip().lower()
        if preferred:
            for method in methods:
                method_id = str(method.get("id", "")).strip().lower()
                method_name = str(method.get("name", "")).strip().lower()
                if preferred in {method_id, method_name}:
                    return method

        gemini_api_key = os.environ.get("GEMINI_API_KEY", "").strip()
        if gemini_api_key:
            selected = _find_auth_method(methods, include=("api key", "gemini"))
            if selected is not None:
                return selected

        use_vertex = os.environ.get("GOOGLE_GENAI_USE_VERTEXAI", "").strip().lower() in {
            "1",
            "true",
            "yes",
            "on",
        }
        if use_vertex:
            selected = _find_auth_method(methods, include=("vertex",))
            if selected is not None:
                return selected

        if gemini_cached_auth_files():
            selected = _find_auth_method(methods, include=("google",))
            if selected is not None:
                return selected

        return methods[0]

    def _auth_meta_for_method(self, auth_method: dict[str, Any]) -> dict[str, Any] | None:
        method_name = str(auth_method.get("name", "")).strip().lower()
        gemini_api_key = os.environ.get("GEMINI_API_KEY", "").strip()
        if gemini_api_key and "api key" in method_name and "vertex" not in method_name:
            return {"api-key": gemini_api_key}
        return None

    def _request(
        self,
        method: str,
        params: dict[str, Any],
        *,
        update_buffer: list[dict[str, Any]] | None = None,
        update_session_id: str | None = None,
    ) -> dict[str, Any]:
        client = self._client_instance()
        return client.request(
            method,
            params,
            timeout_seconds=self.request_timeout_seconds,
            update_buffer=update_buffer,
            update_session_id=update_session_id,
        )

    def _resolve_session_mode(self, runtime_options: dict[str, Any]) -> str:
        for key in ("approval_mode", "mode", "session_mode"):
            value = str(runtime_options.get(key, "")).strip()
            if value:
                return value
        return self.default_mode

    def _resolve_session_model(self, runtime_options: dict[str, Any]) -> str:
        for key in ("model", "session_model"):
            value = str(runtime_options.get(key, "")).strip()
            if value:
                return value
        return str(self.default_model or "").strip()

    def _acp_auth_candidate_status(
        self,
        initialize_result: dict[str, Any],
    ) -> tuple[bool, str]:
        if self._session_id:
            return True, "resident Gemini ACP session is active"
        if os.environ.get("GEMINI_API_KEY", "").strip():
            return True, "Gemini ACP will authenticate via GEMINI_API_KEY"
        use_vertex = os.environ.get("GOOGLE_GENAI_USE_VERTEXAI", "").strip().lower() in {
            "1",
            "true",
            "yes",
            "on",
        }
        if use_vertex and (
            os.environ.get("GOOGLE_API_KEY", "").strip()
            or os.environ.get("GOOGLE_APPLICATION_CREDENTIALS", "").strip()
        ):
            return True, "Gemini ACP will authenticate via Vertex AI environment settings"
        cached_files = gemini_cached_auth_files()
        if cached_files:
            auth_method = _find_auth_method(
                [item for item in initialize_result.get("authMethods", []) if isinstance(item, dict)],
                include=("google",),
            )
            if auth_method is not None:
                return (
                    True,
                    "Gemini ACP detected cached Google login and can reuse it through the resident broker path",
                )
        return (
            False,
            "Gemini ACP did not find a usable auth candidate. Configure GEMINI_API_KEY, Vertex env auth, or login with Google before starting the resident broker.",
        )

    def _stderr_tail(self) -> list[str]:
        client = self._client
        if client is None:
            return []
        try:
            return list(client.stderr_tail())[-4:]
        except Exception:
            return []


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
    return GeminiACPBackend()


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


def _looks_like_acp_auth_required(error: str | None) -> bool:
    text = str(error or "").lower()
    if not text:
        return False
    return any(
        marker in text
        for marker in (
            "authentication required",
            "authentication failed",
            "api key is missing",
            "login",
            "oauth",
            "not configured",
        )
    )


def _find_auth_method(
    methods: list[dict[str, Any]],
    *,
    include: tuple[str, ...],
) -> dict[str, Any] | None:
    for method in methods:
        haystack = " ".join(
            str(method.get(field, "")).strip().lower()
            for field in ("id", "name", "description")
        )
        if all(token in haystack for token in include):
            return method
    return None


def _render_acp_updates(updates: list[dict[str, Any]]) -> str:
    chunks: list[str] = []
    for item in updates:
        normalized = _normalize_acp_update_payload(item)
        if str(normalized.get("sessionUpdate", "")).strip() != "agent_message_chunk":
            continue
        content = normalized.get("content", {})
        if not isinstance(content, dict):
            continue
        if str(content.get("type", "")).strip() != "text":
            continue
        text = str(content.get("text", ""))
        if text:
            chunks.append(text)
    return "".join(chunks).strip()


def _normalize_acp_update_payload(payload: dict[str, Any]) -> dict[str, Any]:
    normalized = dict(payload)
    nested_update = normalized.get("update")
    if isinstance(nested_update, dict):
        merged = dict(nested_update)
        if "sessionId" not in merged and "sessionId" in normalized:
            merged["sessionId"] = normalized["sessionId"]
        return merged
    return normalized
