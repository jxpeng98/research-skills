from __future__ import annotations

import json
import os
import tempfile
import unittest
from unittest import mock
from pathlib import Path
from typing import Any

from bridges.base_bridge import BridgeResponse
from bridges import i18n as i18n_module
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
    def setUp(self) -> None:
        super().setUp()
        self._prev_cli_lang = os.environ.get("RESEARCH_CLI_LANG")
        os.environ["RESEARCH_CLI_LANG"] = "zh-CN"
        i18n_module._current_lang = None

    def tearDown(self) -> None:
        if self._prev_cli_lang is None:
            os.environ.pop("RESEARCH_CLI_LANG", None)
        else:
            os.environ["RESEARCH_CLI_LANG"] = self._prev_cli_lang
        i18n_module._current_lang = None
        super().tearDown()

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
        self.assertIn("## 并发执行分析", result.merged_analysis)
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

        self.assertIn("## 并发执行分析 (双重/Dual)", result.merged_analysis)
        self.assertIn("- 失败的 Agent: gemini", result.merged_analysis)
        self.assertIn("## 综合归纳 (Synthesis)", result.merged_analysis)

    def test_parallel_unknown_profile_returns_structured_error(self) -> None:
        from bridges.errors import ConfigError
        orchestrator = MockOrchestrator()
        with self.assertRaises(ConfigError):
            result = orchestrator.execute(
                mode=CollaborationMode.PARALLEL,
                cwd=REPO_ROOT,
                prompt="bad profile",
                profile="profile-not-exist",
            )

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
        self.assertIn("## 起草阶段草稿 (", result.merged_analysis)
        self.assertIn("## 复核阶段审查 (", result.merged_analysis)
        self.assertIn("## 运行预案 (Agent Profiles)", result.merged_analysis)
        self.assertGreaterEqual(len(orchestrator.runtime_calls), 2)
        directives = [call["profile_directive"] for call in orchestrator.runtime_calls]
        self.assertTrue(any("(stage: draft)" in directive for directive in directives))
        self.assertTrue(any("(stage: review)" in directive for directive in directives))

    def test_task_run_unknown_profile_returns_structured_error(self) -> None:
        from bridges.errors import ConfigError
        orchestrator = MockOrchestrator()
        with self.assertRaises(ConfigError):
            result = orchestrator.task_run(
                task_id="F3",
                paper_type="empirical",
                topic="ai-in-education",
                cwd=REPO_ROOT,
                draft_profile="not-exist",
            )

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
    def test_parse_review_verdict_pass(self) -> None:
        orchestrator = MockOrchestrator()
        verdict, conf = orchestrator._parse_review_verdict("Verdict: PASS\nConfidence: 0.95")
        self.assertEqual(verdict, "PASS")
        self.assertEqual(conf, 0.95)

    def test_parse_review_verdict_block(self) -> None:
        orchestrator = MockOrchestrator()
        verdict, conf = orchestrator._parse_review_verdict("Overall Verdict: BLOCK\nConfidence (0-1): 0.5")
        self.assertEqual(verdict, "BLOCK")
        self.assertEqual(conf, 0.5)

    def test_parse_review_verdict_convergence(self) -> None:
        orchestrator = MockOrchestrator()
        # High confidence but BLOCK should auto-converge to PASS
        verdict, conf = orchestrator._parse_review_verdict("- Verdict: BLOCK\nConfidence: 0.9")
        self.assertEqual(verdict, "PASS")
        self.assertEqual(conf, 0.9)

    def test_critique_question_injection(self) -> None:
        orchestrator = MockOrchestrator()
        prompt = orchestrator._build_task_review_prompt(
            task_packet={"task_id": "F3"},
            mcp_evidence=[],
            skill_cards=[],
            draft_output="draft",
        )
        self.assertIn("Stage-specific critique questions", prompt)
        # F-series task is manuscript stage
        self.assertIn("Do the claims in the Discussion section", prompt)

    def test_task_run_revision_loop_converges(self) -> None:
        # Create a mock orchestrator that fails the first review but passes the second
        class ConvergingOrchestrator(MockOrchestrator):
            def __init__(self) -> None:
                super().__init__()
                self.review_count = 0

            def _execute_runtime_agent(
                self,
                agent_name: str,
                prompt: str,
                cwd: Path,
                runtime_options: dict[str, Any] | None = None,
                profile_directive: str | None = None,
            ) -> BridgeResponse:
                # If it's the review agent, output BLOCK then PASS
                if "Review the draft" in prompt:
                    self.review_count += 1
                    if self.review_count == 1:
                        content = "Verdict: BLOCK\nConfidence: 0.5\nCritical Issues: Needed more depth"
                    else:
                        content = "Verdict: PASS\nConfidence: 0.9\nLGTM."
                    
                    self.runtime_calls.append({
                        "agent": agent_name,
                        "runtime_options": dict(runtime_options or {}),
                        "profile_directive": profile_directive or "",
                        "cwd": str(cwd),
                    })
                    return BridgeResponse(success=True, model=agent_name, content=content)
                
                return super()._execute_runtime_agent(
                    agent_name, prompt, cwd, runtime_options, profile_directive
                )

        orchestrator = ConvergingOrchestrator()
        result = orchestrator.task_run(
            task_id="F3",
            paper_type="empirical",
            topic="test-topic",
            cwd=REPO_ROOT,
        )
        # Check that revision history tracked both rounds
        self.assertIn("Round 0: BLOCK", result.merged_analysis)
        self.assertIn("Round 1: PASS", result.merged_analysis)
        # Codex (draft) -> Claude (review 1) -> Codex (revise) -> Claude (review 2) -> Gemini (triad)
        agents_called = [c["agent"] for c in orchestrator.runtime_calls]
        self.assertEqual(agents_called.count("claude"), 2)
        self.assertEqual(agents_called.count("codex"), 2)

    @unittest.mock.patch("sys.stdin.isatty", return_value=False)
    def test_interactive_mode_skips_when_no_tty(self, mock_isatty) -> None:
        """Ensure interactive flag is ignored if not running in a true TTY."""
        orchestrator = MockOrchestrator()
        # MockOrchestrator doesn't accept interactive in init natively, so we set it
        orchestrator.interactive = True 
        
        result = orchestrator.execute(
            mode=CollaborationMode.SINGLE,
            cwd=REPO_ROOT,
            prompt="test interactive bypass",
            single_model="codex",
        )
        # Should execute normally without blocking for input
        self.assertTrue(result.codex_response.success)
        self.assertEqual(len(orchestrator.runtime_calls), 1)

    @unittest.mock.patch("sys.stdin.isatty", return_value=True)
    @unittest.mock.patch("builtins.input", side_effect=["y"])
    def test_interactive_mode_proceeds_on_yes(self, mock_input, mock_isatty) -> None:
        """Ensure interactive mode runs the agent when user types Y."""
        orchestrator = MockOrchestrator()
        orchestrator.interactive = True 
        
        result = orchestrator.execute(
            mode=CollaborationMode.SINGLE,
            cwd=REPO_ROOT,
            prompt="test interactive yes",
            single_model="codex",
        )
        
        self.assertTrue(result.codex_response.success)
        self.assertEqual(len(orchestrator.runtime_calls), 1)
        mock_input.assert_called_once()

    @unittest.mock.patch("sys.stdin.isatty", return_value=True)
    @unittest.mock.patch("builtins.input", side_effect=["n"])
    def test_interactive_mode_skips_on_no(self, mock_input, mock_isatty) -> None:
        """Ensure interactive mode aborts execution when user types n."""
        # Need to use the real base execute so it hits the _execute_runtime_agent behavior
        orchestrator = MockOrchestrator()
        # Restore the original _execute_runtime_agent logic to test the skip
        orchestrator._execute_runtime_agent = ModelOrchestrator._execute_runtime_agent.__get__(orchestrator, ModelOrchestrator)
        orchestrator.interactive = True 
        
        # execution will fail because real Claude/Codex APIs aren't mocked here, 
        # but the Interactive skip happens BEFORE the API call
        result = orchestrator.execute(
            mode=CollaborationMode.SINGLE,
            cwd=REPO_ROOT,
            prompt="test interactive no",
            single_model="codex",
        )
        
        self.assertFalse(result.codex_response.success)
        self.assertIn("Skipped by user", result.codex_response.error)
        mock_input.assert_called_once()


if __name__ == "__main__":
    unittest.main(verbosity=2)
