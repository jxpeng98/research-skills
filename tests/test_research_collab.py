from __future__ import annotations

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
    GeminiBrokerBackend,
    GeminiSessionBroker,
    bridge_response_from_broker_payload,
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


class ResearchCollabTests(unittest.TestCase):
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
