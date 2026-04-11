from __future__ import annotations

import os
import sys
import types
import unittest
from pathlib import Path
from unittest import mock

try:
    import yaml as _yaml  # noqa: F401
except ModuleNotFoundError:
    yaml_stub = types.SimpleNamespace(
        safe_load=lambda *_args, **_kwargs: {},
        safe_dump=lambda *_args, **_kwargs: "",
        YAMLError=ValueError,
    )
    sys.modules["yaml"] = yaml_stub

from bridges.base_bridge import BridgeResponse
from bridges.orchestrator import ModelOrchestrator
from bridges.providers.research_collab import (
    BROKER_BACKEND_CMD_ENV,
    GEMINI_TRANSPORT_ENV,
    GeminiACPBackend,
    GeminiAcpClientError,
    GeminiBrokerBackend,
    GeminiSessionBroker,
    build_default_broker_backend,
    bridge_response_from_broker_payload,
    resolve_gemini_transport,
)


REPO_ROOT = Path(__file__).resolve().parents[1]


class _SuccessBackend(GeminiBrokerBackend):
    name = "test-success"

    def __init__(self) -> None:
        self.calls: list[dict[str, object]] = []

    def health(self) -> dict[str, object]:
        return {
            "status": "ok",
            "summary": "test backend ready",
            "data": {"backend": self.name, "ready": True},
        }

    def prompt(
        self,
        *,
        prompt: str,
        cwd: Path,
        session_id: str | None = None,
        runtime_options: dict[str, object] | None = None,
    ) -> BridgeResponse:
        self.calls.append(
            {
                "prompt": prompt,
                "cwd": str(cwd),
                "session_id": session_id or "",
                "runtime_options": dict(runtime_options or {}),
            }
        )
        return BridgeResponse(
            success=True,
            model="gemini",
            session_id=f"session-{len(self.calls)}",
            content=f"broker:{prompt}",
        )

    def reset(self) -> dict[str, object]:
        return {
            "status": "ok",
            "summary": "test backend reset",
            "data": {"backend": self.name, "ready": True},
        }


class _AuthLossBackend(GeminiBrokerBackend):
    name = "test-auth-loss"

    def health(self) -> dict[str, object]:
        return {
            "status": "ok",
            "summary": "auth-loss backend ready",
            "data": {"backend": self.name, "ready": True},
        }

    def prompt(
        self,
        *,
        prompt: str,
        cwd: Path,
        session_id: str | None = None,
        runtime_options: dict[str, object] | None = None,
    ) -> BridgeResponse:
        return BridgeResponse.from_error(
            "gemini",
            "Opening authentication page in your browser.",
        )


class _FailDirectGemini:
    def execute(self, *_args, **_kwargs) -> BridgeResponse:
        raise AssertionError("direct Gemini bridge should not run when broker succeeds")


class _BrokerRoutingOrchestrator(ModelOrchestrator):
    def __init__(self) -> None:
        super().__init__(standards_dir=REPO_ROOT / "standards")
        self.gemini = _FailDirectGemini()

    def _runtime_preflight_error(
        self,
        agent_name: str,
        cwd: Path,
        runtime_options: dict[str, object] | None = None,
    ) -> str | None:
        return None


class _FakeACPClient:
    def __init__(self, *, fail_new_session_once: bool = False) -> None:
        self.fail_new_session_once = fail_new_session_once
        self.failed_new_session = False
        self.closed = False
        self.calls: list[tuple[str, dict[str, object]]] = []
        self.initialize_result = {
            "protocolVersion": 1,
            "authMethods": [
                {
                    "id": "google-login",
                    "name": "Log in with Google",
                    "description": "Use cached Google login",
                },
                {
                    "id": "gemini-api-key",
                    "name": "Gemini API key",
                    "description": "Use an API key with Gemini Developer API",
                },
                {
                    "id": "vertex-ai",
                    "name": "Vertex AI",
                    "description": "Use Vertex AI",
                },
            ],
        }

    def is_alive(self) -> bool:
        return not self.closed

    def stderr_tail(self) -> list[str]:
        return []

    def initialize(self) -> dict[str, object]:
        return self.initialize_result

    def close(self) -> None:
        self.closed = True

    def request(
        self,
        method: str,
        params: dict[str, object],
        *,
        timeout_seconds: float | None = None,
        update_buffer: list[dict[str, object]] | None = None,
        update_session_id: str | None = None,
    ) -> dict[str, object]:
        self.calls.append((method, dict(params)))
        if method == "session/new":
            if self.fail_new_session_once and not self.failed_new_session:
                self.failed_new_session = True
                raise GeminiAcpClientError("session/new: Authentication required.")
            return {
                "sessionId": "acp-session-1",
                "modes": {"currentModeId": "default"},
                "models": {"currentModelId": "gemini-2.5-pro"},
            }
        if method == "authenticate":
            return {}
        if method == "session/set_mode":
            return {}
        if method == "session/set_model":
            return {}
        if method == "session/prompt":
            if update_buffer is not None:
                update_buffer.append(
                    {
                        "sessionId": update_session_id or "acp-session-1",
                        "sessionUpdate": "agent_message_chunk",
                        "content": {"type": "text", "text": "hello from acp"},
                    }
                )
            return {"stopReason": "end_turn"}
        return {}


class ResearchCollabTests(unittest.TestCase):
    def test_resolve_gemini_transport_prefers_runtime_options_over_env(self) -> None:
        with mock.patch.dict(os.environ, {GEMINI_TRANSPORT_ENV: "broker"}, clear=False):
            transport, source = resolve_gemini_transport({"transport": "direct"})

        self.assertEqual(transport, "direct")
        self.assertEqual(source, "runtime_options.transport")

    def test_broker_round_trip_tracks_session_and_reset(self) -> None:
        backend = _SuccessBackend()
        broker = GeminiSessionBroker(backend=backend, token="secret")

        health = broker.health()
        self.assertEqual(health["status"], "ok")
        self.assertTrue(health["data"]["ready"])
        self.assertEqual(health["data"]["session_id"], "")

        prompt_payload = broker.prompt({"prompt": "hello", "cwd": str(REPO_ROOT)})
        bridge_response = bridge_response_from_broker_payload(prompt_payload)
        self.assertTrue(bridge_response.success)
        self.assertEqual(bridge_response.content, "broker:hello")
        self.assertEqual(bridge_response.session_id, "session-1")

        health_after_prompt = broker.health()
        self.assertEqual(health_after_prompt["data"]["session_id"], "session-1")
        self.assertEqual(health_after_prompt["data"]["prompt_count"], 1)

        reset_payload = broker.reset({"reason": "test reset"})
        self.assertEqual(reset_payload["status"], "ok")
        self.assertIn("test reset", reset_payload["summary"])

        health_after_reset = broker.health()
        self.assertEqual(health_after_reset["data"]["session_id"], "")
        self.assertFalse(health_after_reset["data"]["auth_lost"])

    def test_broker_marks_auth_lost_after_browser_auth_failure(self) -> None:
        broker = GeminiSessionBroker(backend=_AuthLossBackend())

        prompt_payload = broker.prompt({"prompt": "hello", "cwd": str(REPO_ROOT)})
        bridge_response = bridge_response_from_broker_payload(prompt_payload)
        self.assertFalse(bridge_response.success)
        self.assertIn("authentication page", bridge_response.error or "")

        health = broker.health()
        self.assertEqual(health["status"], "warning")
        self.assertTrue(health["data"]["auth_lost"])
        self.assertFalse(health["data"]["ready"])

    def test_orchestrator_prefers_broker_for_gemini(self) -> None:
        broker = GeminiSessionBroker(backend=_SuccessBackend())
        orchestrator = _BrokerRoutingOrchestrator()

        fake_client = types.SimpleNamespace(
            prompt=lambda **kwargs: broker.prompt(
                {
                    "prompt": kwargs["prompt"],
                    "cwd": str(kwargs["cwd"]),
                    "runtime_options": kwargs.get("runtime_options", {}),
                }
            )
        )
        with mock.patch(
            "bridges.orchestrator.broker_client_from_env",
            return_value=fake_client,
        ):
            response = orchestrator._execute_runtime_agent(
                "gemini",
                "broker route test",
                REPO_ROOT,
            )

        self.assertTrue(response.success)
        self.assertEqual(response.content, "broker:broker route test")

    def test_preflight_broker_transport_accepts_healthy_broker_without_direct_auth(self) -> None:
        orchestrator = ModelOrchestrator(standards_dir=REPO_ROOT / "standards")

        with mock.patch.dict(os.environ, {GEMINI_TRANSPORT_ENV: "broker"}, clear=False):
            with mock.patch.object(
                orchestrator,
                "_gemini_broker_status",
                return_value={"configured": True, "ok": True, "detail": "broker ready", "data": {}},
            ):
                with mock.patch.object(
                    orchestrator,
                    "_gemini_noninteractive_auth_status",
                    return_value=(False, "direct auth missing"),
                ):
                    error = orchestrator._runtime_preflight_error("gemini", REPO_ROOT, {})

        self.assertIsNone(error)

    def test_preflight_direct_transport_ignores_broker_and_requires_direct_auth(self) -> None:
        orchestrator = ModelOrchestrator(standards_dir=REPO_ROOT / "standards")

        with mock.patch.dict(os.environ, {GEMINI_TRANSPORT_ENV: "direct"}, clear=False):
            with mock.patch.object(
                orchestrator,
                "_gemini_broker_status",
                return_value={"configured": True, "ok": True, "detail": "broker ready", "data": {}},
            ):
                with mock.patch.object(
                    orchestrator,
                    "_gemini_noninteractive_auth_status",
                    return_value=(False, "direct auth missing"),
                ):
                    with mock.patch("bridges.orchestrator.shutil.which", return_value="/tmp/gemini"):
                        error = orchestrator._runtime_preflight_error("gemini", REPO_ROOT, {})

        self.assertEqual(error, "direct auth missing")

    def test_doctor_marks_direct_auth_ok_when_auto_transport_resolves_to_broker(self) -> None:
        orchestrator = ModelOrchestrator(standards_dir=REPO_ROOT / "standards")

        with mock.patch.dict(os.environ, {}, clear=False):
            with mock.patch("bridges.orchestrator.shutil.which", side_effect=lambda name: f"/tmp/{name}"):
                with mock.patch.object(
                    orchestrator,
                    "_check_command_available",
                    return_value=(True, "/tmp/mock-command"),
                ):
                    with mock.patch.object(
                        orchestrator,
                        "_gemini_broker_status",
                        return_value={"configured": True, "ok": True, "detail": "broker ready", "data": {}},
                    ):
                        with mock.patch.object(
                            orchestrator,
                            "_gemini_noninteractive_auth_status",
                            return_value=(False, "direct auth missing"),
                        ):
                            result = orchestrator.doctor(REPO_ROOT)

        self.assertIn("[OK] Gemini transport: auto (source: default)", result.merged_analysis)
        self.assertIn("[OK] Gemini broker: broker ready", result.merged_analysis)
        self.assertIn(
            "[OK] Gemini direct auth: not required because auto transport resolves to broker",
            result.merged_analysis,
        )

    def test_build_default_broker_backend_prefers_acp_resident_without_override(self) -> None:
        with mock.patch.dict(os.environ, {BROKER_BACKEND_CMD_ENV: ""}, clear=False):
            backend = build_default_broker_backend()

        self.assertIsInstance(backend, GeminiACPBackend)

    def test_acp_backend_health_accepts_cached_google_login_candidate(self) -> None:
        fake_client = _FakeACPClient()
        backend = GeminiACPBackend(client_factory=lambda: fake_client)

        with mock.patch("bridges.providers.research_collab.shutil.which", return_value="/tmp/gemini"):
            with mock.patch(
                "bridges.providers.research_collab.gemini_cached_auth_files",
                return_value=[Path("/tmp/oauth_creds.json")],
            ):
                health = backend.health()

        self.assertEqual(health["status"], "ok")
        self.assertTrue(health["data"]["ready"])
        self.assertIn("cached Google login", health["data"]["auth_status"])

    def test_acp_backend_prompt_authenticates_with_google_login_when_cached_oauth_exists(self) -> None:
        fake_client = _FakeACPClient(fail_new_session_once=True)
        backend = GeminiACPBackend(client_factory=lambda: fake_client)

        with mock.patch.dict(os.environ, {"GEMINI_API_KEY": ""}, clear=False):
            with mock.patch(
                "bridges.providers.research_collab.gemini_cached_auth_files",
                return_value=[Path("/tmp/oauth_creds.json")],
            ):
                response = backend.prompt(prompt="hello", cwd=REPO_ROOT)

        self.assertTrue(response.success)
        self.assertEqual(response.content, "hello from acp")
        authenticate_calls = [payload for method, payload in fake_client.calls if method == "authenticate"]
        self.assertEqual(len(authenticate_calls), 1)
        self.assertEqual(authenticate_calls[0]["methodId"], "google-login")

    def test_acp_backend_prompt_authenticates_with_gemini_api_key_when_available(self) -> None:
        fake_client = _FakeACPClient(fail_new_session_once=True)
        backend = GeminiACPBackend(client_factory=lambda: fake_client)

        with mock.patch.dict(os.environ, {"GEMINI_API_KEY": "test-key"}, clear=False):
            with mock.patch(
                "bridges.providers.research_collab.gemini_cached_auth_files",
                return_value=[],
            ):
                response = backend.prompt(prompt="hello", cwd=REPO_ROOT)

        self.assertTrue(response.success)
        authenticate_calls = [payload for method, payload in fake_client.calls if method == "authenticate"]
        self.assertEqual(len(authenticate_calls), 1)
        self.assertEqual(authenticate_calls[0]["methodId"], "gemini-api-key")
        self.assertEqual(authenticate_calls[0]["_meta"]["api-key"], "test-key")

    def test_acp_backend_extracts_text_from_nested_update_payload(self) -> None:
        class _NestedUpdateClient(_FakeACPClient):
            def request(
                self,
                method: str,
                params: dict[str, object],
                *,
                timeout_seconds: float | None = None,
                update_buffer: list[dict[str, object]] | None = None,
                update_session_id: str | None = None,
            ) -> dict[str, object]:
                self.calls.append((method, dict(params)))
                if method == "session/new":
                    return {
                        "sessionId": "acp-session-1",
                        "modes": {"currentModeId": "default"},
                        "models": {"currentModelId": "gemini-2.5-pro"},
                    }
                if method == "session/prompt":
                    if update_buffer is not None:
                        update_buffer.append(
                            {
                                "sessionId": update_session_id or "acp-session-1",
                                "update": {
                                    "sessionUpdate": "agent_message_chunk",
                                    "content": {"type": "text", "text": "nested hello"},
                                },
                            }
                        )
                    return {"stopReason": "end_turn"}
                return {}

        backend = GeminiACPBackend(client_factory=lambda: _NestedUpdateClient())
        response = backend.prompt(prompt="hello", cwd=REPO_ROOT, runtime_options={"approval_mode": "default"})

        self.assertTrue(response.success)
        self.assertEqual(response.content, "nested hello")
