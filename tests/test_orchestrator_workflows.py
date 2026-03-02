from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from typing import Any

from bridges.base_bridge import BridgeResponse
from bridges.mcp_connectors import MCPEvidence
from bridges.orchestrator import CollaborationMode, ModelOrchestrator


REPO_ROOT = Path(__file__).resolve().parents[1]


class MockOrchestrator(ModelOrchestrator):
    def __init__(self) -> None:
        super().__init__(standards_dir=REPO_ROOT / "standards")
        self.runtime_calls: list[dict[str, Any]] = []

    def _execute_runtime_agent(
        self,
        agent_name: str,
        prompt: str,
        cwd: Path,
        runtime_options: dict[str, Any] | None = None,
        profile_directive: str | None = None,
    ) -> BridgeResponse:
        self.runtime_calls.append(
            {
                "agent": agent_name,
                "runtime_options": dict(runtime_options or {}),
                "profile_directive": profile_directive or "",
                "cwd": str(cwd),
            }
        )
        return BridgeResponse(
            success=True,
            model=agent_name,
            content=f"{agent_name} mock output",
        )

    def _collect_mcp_evidence(
        self,
        task_packet: dict[str, Any],
        cwd: Path,
        strict: bool = False,
    ) -> tuple[list[MCPEvidence], list[str]]:
        return [
            MCPEvidence(
                provider="filesystem",
                status="ok",
                summary="mock evidence",
            )
        ], []


class OrchestratorWorkflowTests(unittest.TestCase):
    def _write_profile_file(self, payload: dict[str, Any]) -> Path:
        handle = tempfile.NamedTemporaryFile(
            mode="w",
            suffix=".json",
            prefix="research-skills-profile-",
            delete=False,
            encoding="utf-8",
        )
        with handle:
            json.dump(payload, handle, ensure_ascii=False)
        profile_path = Path(handle.name)
        self.addCleanup(profile_path.unlink, missing_ok=True)
        return profile_path

    def test_parallel_runs_with_mock_runtime(self) -> None:
        orchestrator = MockOrchestrator()
        result = orchestrator.execute(
            mode=CollaborationMode.PARALLEL,
            cwd=REPO_ROOT,
            prompt="parallel test",
            parallel_summarizer="claude",
        )

        self.assertEqual(result.mode, "parallel")
        self.assertIn("## Parallel Execution", result.merged_analysis)
        self.assertEqual(len(orchestrator.runtime_calls), 4)
        called_agents = [call["agent"] for call in orchestrator.runtime_calls]
        self.assertIn("codex", called_agents)
        self.assertIn("claude", called_agents)
        self.assertIn("gemini", called_agents)

    def test_parallel_profile_file_parsing_applies_runtime_options(self) -> None:
        orchestrator = MockOrchestrator()
        profile_path = self._write_profile_file(
            {
                "profiles": {
                    "qa-custom": {
                        "persona": "Custom persona",
                        "analysis_style": "Custom analysis style",
                        "runtime_options": {
                            "codex": {"timeout_seconds": 17}
                        },
                    }
                }
            }
        )
        result = orchestrator.execute(
            mode=CollaborationMode.PARALLEL,
            cwd=REPO_ROOT,
            prompt="profile parse test",
            profile_file=profile_path,
            profile="qa-custom",
            summarizer_profile="qa-custom",
            parallel_summarizer="claude",
        )

        self.assertIn("- Base profile: qa-custom", result.merged_analysis)
        self.assertIn("- Summarizer profile: qa-custom", result.merged_analysis)
        self.assertTrue(
            any(
                call["agent"] == "codex"
                and call["runtime_options"].get("timeout_seconds") == 17
                for call in orchestrator.runtime_calls
            )
        )
        self.assertTrue(
            any("Agent Profile: qa-custom" in call["profile_directive"] for call in orchestrator.runtime_calls)
        )

    def test_parallel_degrades_to_dual_when_one_agent_fails(self) -> None:
        class PartialFailOrchestrator(MockOrchestrator):
            def _execute_runtime_agent(
                self,
                agent_name: str,
                prompt: str,
                cwd: Path,
                runtime_options: dict[str, Any] | None = None,
                profile_directive: str | None = None,
            ) -> BridgeResponse:
                if agent_name == "gemini":
                    return BridgeResponse.from_error("gemini", "forced failure")
                return super()._execute_runtime_agent(
                    agent_name,
                    prompt,
                    cwd,
                    runtime_options,
                    profile_directive,
                )

        orchestrator = PartialFailOrchestrator()
        result = orchestrator.execute(
            mode=CollaborationMode.PARALLEL,
            cwd=REPO_ROOT,
            prompt="degrade test",
            parallel_summarizer="claude",
        )

        self.assertIn("## Parallel Execution (dual)", result.merged_analysis)
        self.assertIn("- Failed agents: gemini", result.merged_analysis)
        self.assertIn("## Synthesis", result.merged_analysis)

    def test_parallel_unknown_profile_returns_structured_error(self) -> None:
        orchestrator = MockOrchestrator()
        result = orchestrator.execute(
            mode=CollaborationMode.PARALLEL,
            cwd=REPO_ROOT,
            prompt="bad profile",
            profile="profile-not-exist",
        )

        self.assertEqual(result.mode, "parallel")
        self.assertIn("Unknown agent profile", result.merged_analysis)
        self.assertEqual(result.confidence, 0.0)
        self.assertEqual(len(orchestrator.runtime_calls), 0)

    def test_task_run_executes_with_draft_and_review(self) -> None:
        orchestrator = MockOrchestrator()
        result = orchestrator.task_run(
            task_id="F3",
            paper_type="empirical",
            topic="ai-in-education",
            cwd=REPO_ROOT,
            context="task-run test",
        )

        self.assertEqual(result.mode, "task-run")
        self.assertIn("## Draft (", result.merged_analysis)
        self.assertIn("## Review (", result.merged_analysis)
        self.assertIn("## Agent Profiles", result.merged_analysis)
        self.assertGreaterEqual(len(orchestrator.runtime_calls), 2)
        directives = [call["profile_directive"] for call in orchestrator.runtime_calls]
        self.assertTrue(any("(stage: draft)" in directive for directive in directives))
        self.assertTrue(any("(stage: review)" in directive for directive in directives))

    def test_task_run_unknown_profile_returns_structured_error(self) -> None:
        orchestrator = MockOrchestrator()
        result = orchestrator.task_run(
            task_id="F3",
            paper_type="empirical",
            topic="ai-in-education",
            cwd=REPO_ROOT,
            draft_profile="not-exist",
        )

        self.assertEqual(result.mode, "task-run")
        self.assertIn("Unknown agent profile", result.merged_analysis)
        self.assertEqual(result.confidence, 0.0)
        self.assertEqual(len(orchestrator.runtime_calls), 0)

    def test_task_run_stage_profile_overrides_apply(self) -> None:
        orchestrator = MockOrchestrator()
        profile_path = self._write_profile_file(
            {
                "profiles": {
                    "p-draft": {
                        "runtime_options": {
                            "codex": {"timeout_seconds": 11},
                            "claude": {"timeout_seconds": 11},
                            "gemini": {"timeout_seconds": 11},
                        },
                        "draft_style": "Draft quickly",
                    },
                    "p-review": {
                        "runtime_options": {
                            "codex": {"timeout_seconds": 22},
                            "claude": {"timeout_seconds": 22},
                            "gemini": {"timeout_seconds": 22},
                        },
                        "review_style": "Review strictly",
                    },
                    "p-triad": {
                        "runtime_options": {
                            "codex": {"timeout_seconds": 33},
                            "claude": {"timeout_seconds": 33},
                            "gemini": {"timeout_seconds": 33},
                        },
                        "triad_style": "Triad arbitration",
                    },
                }
            }
        )
        result = orchestrator.task_run(
            task_id="F3",
            paper_type="empirical",
            topic="ai-in-education",
            cwd=REPO_ROOT,
            profile_file=profile_path,
            draft_profile="p-draft",
            review_profile="p-review",
            triad_profile="p-triad",
        )

        self.assertIn("- draft: p-draft", result.merged_analysis)
        self.assertIn("- review: p-review", result.merged_analysis)
        self.assertIn("- triad: p-triad", result.merged_analysis)
        self.assertTrue(
            any(
                call["runtime_options"].get("timeout_seconds") in {11, 22, 33}
                for call in orchestrator.runtime_calls
            )
        )

    def test_build_profile_directive_bilingual(self) -> None:
        """Verify that output_language triggers additional constraint injection."""
        orchestrator = MockOrchestrator()
        
        # Test 1: Profile without output_language
        directive_normal = orchestrator._build_profile_directive(
            "default",
            {"persona": "Tester", "analysis_style": "Normal style"},
            "analysis"
        )
        self.assertNotIn("Output Language Directive", directive_normal)

        # Test 2: Profile with output_language
        directive_bilingual = orchestrator._build_profile_directive(
            "bilingual",
            {"persona": "Tester", "output_language": "zh-CN"},
            "analysis"
        )
        self.assertIn("Output Language Directive: You MUST output the final response in zh-CN", directive_bilingual)
        
        # Test 3: Profile with ONLY output_language and no persona/style
        directive_only_lang = orchestrator._build_profile_directive(
            "only_lang",
            {"output_language": "fr-FR"},
            "analysis"
        )
        self.assertIn("Output Language Directive: You MUST output the final response in fr-FR", directive_only_lang)
        self.assertIn("Agent Profile: only_lang", directive_only_lang)


if __name__ == "__main__":
    unittest.main(verbosity=2)
