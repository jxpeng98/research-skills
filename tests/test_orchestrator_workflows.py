from __future__ import annotations

import json
import os
import tempfile
import unittest
from unittest import mock
from pathlib import Path
from typing import Any

from bridges.base_bridge import BridgeResponse, CollaborationResult
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
                "prompt": prompt,
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

    def test_task_plan_exposes_functional_and_runtime_routing(self) -> None:
        orchestrator = MockOrchestrator()
        result = orchestrator.task_plan(
            task_id="F3",
            paper_type="empirical",
            topic="ai-in-education",
            cwd=REPO_ROOT,
        )

        self.assertIn("### Functional routing", result.merged_analysis)
        self.assertIn("`writing-agent` [stage-default]", result.merged_analysis)
        self.assertIn("### Runtime routing plan", result.merged_analysis)
        self.assertEqual(result.data["functional_owner"], "writing-agent")
        self.assertEqual(result.data["runtime_plan"]["primary_agent"], "claude")
        self.assertTrue(result.data["functional_owner_chain"])

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

    def test_task_run_emits_functional_routing_trace(self) -> None:
        orchestrator = MockOrchestrator()
        result = orchestrator.task_run(
            task_id="F3",
            paper_type="empirical",
            topic="ai-in-education",
            cwd=REPO_ROOT,
            context="functional routing test",
        )

        self.assertIn("Functional owner resolved for F3: writing-agent", result.merged_analysis)
        self.assertIn("Functional handoff chain:", result.merged_analysis)
        self.assertIn("\"functional_owner\": \"writing-agent\"", result.merged_analysis)
        self.assertIn("\"runtime_plan\": {", result.merged_analysis)

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

    def test_task_run_focus_outputs_limits_active_outputs(self) -> None:
        orchestrator = MockOrchestrator()
        result = orchestrator.task_run(
            task_id="F3",
            paper_type="empirical",
            topic="ai-in-education",
            cwd=REPO_ROOT,
            focus_outputs=["manuscript/manuscript.md", "manuscript/results_interpretation.md"],
            output_budget=1,
        )

        self.assertIn("\"artifact_policy\": \"focused\"", result.merged_analysis)
        self.assertIn("\"required_outputs\": [\n    \"manuscript/manuscript.md\"\n  ]", result.merged_analysis)
        self.assertIn("\"deferred_outputs\": [", result.merged_analysis)
        self.assertIn("manuscript/results_interpretation.md", result.merged_analysis)
        self.assertIn("Output control: policy=focused, active_outputs=1/3.", result.merged_analysis)

    def test_build_task_prompts_include_deep_research_constraints(self) -> None:
        orchestrator = MockOrchestrator()
        task_packet = {
            "task_id": "B1",
            "required_outputs": ["protocol.md"],
            "deferred_outputs": ["search_results.csv"],
            "research_depth": "deep",
            "evidence_expansion_rounds": 2,
        }

        draft_prompt = orchestrator._build_task_draft_prompt(
            task_packet=task_packet,
            mcp_evidence=[],
            skill_cards=[],
            extra_context=None,
        )
        review_prompt = orchestrator._build_task_review_prompt(
            task_packet=task_packet,
            mcp_evidence=[],
            skill_cards=[],
            draft_output="draft",
        )

        self.assertIn("Deep research mode is active", draft_prompt)
        self.assertIn("Evidence Expansion Log", draft_prompt)
        self.assertIn("Deferred Outputs", draft_prompt)
        self.assertIn("4-6 highly specific, critical questions", review_prompt)
        self.assertIn("Evidence Depth Assessment", review_prompt)

    def test_stage_i_prompt_templates_are_structured(self) -> None:
        orchestrator = MockOrchestrator()

        i5_prompt = orchestrator._build_task_draft_prompt(
            task_packet={"task_id": "I5", "required_outputs": ["code/code_specification.md"]},
            mcp_evidence=[],
            skill_cards=[],
            extra_context=None,
        )
        i6_prompt = orchestrator._build_task_draft_prompt(
            task_packet={"task_id": "I6", "required_outputs": ["code/plan.md"]},
            mcp_evidence=[],
            skill_cards=[],
            extra_context=None,
        )
        i7_prompt = orchestrator._build_task_draft_prompt(
            task_packet={"task_id": "I7", "required_outputs": ["code/performance_profile.md"]},
            mcp_evidence=[],
            skill_cards=[],
            extra_context=None,
        )
        i4_prompt = orchestrator._build_task_draft_prompt(
            task_packet={"task_id": "I4", "required_outputs": ["code/reproducibility_audit.md"]},
            mcp_evidence=[],
            skill_cards=[],
            extra_context=None,
        )
        i8_review_prompt = orchestrator._build_task_review_prompt(
            task_packet={"task_id": "I8", "required_outputs": ["code/code_review.md"]},
            mcp_evidence=[],
            skill_cards=[],
            draft_output="draft",
        )
        i7_review_prompt = orchestrator._build_task_review_prompt(
            task_packet={"task_id": "I7", "required_outputs": ["code/performance_profile.md"]},
            mcp_evidence=[],
            skill_cards=[],
            draft_output="draft",
        )
        i4_review_prompt = orchestrator._build_task_review_prompt(
            task_packet={"task_id": "I4", "required_outputs": ["code/reproducibility_audit.md"]},
            mcp_evidence=[],
            skill_cards=[],
            draft_output="draft",
        )

        self.assertIn("## Spec Contract Block", i5_prompt)
        self.assertIn("must start with YAML frontmatter", i5_prompt)
        self.assertIn("fenced `json` block", i5_prompt)
        self.assertIn("## Validation Matrix", i5_prompt)
        self.assertIn("## Plan Contract Block", i6_prompt)
        self.assertIn("must start with YAML frontmatter", i6_prompt)
        self.assertIn("## Rollback / Recovery", i6_prompt)
        self.assertIn("## Execution Contract Block", i7_prompt)
        self.assertIn("fenced `json` execution contract block", i7_review_prompt)
        self.assertIn("## Artifact Inventory", i7_prompt)
        self.assertIn("## Audit Contract Block", i4_prompt)
        self.assertIn("must start with YAML frontmatter", i4_prompt)
        self.assertIn("## Audit Verdict", i4_prompt)
        self.assertIn("Structured deliverable review requirements:", i8_review_prompt)
        self.assertIn("fenced `json` review contract block", i8_review_prompt)
        self.assertIn("severity-ranked", i8_review_prompt)
        self.assertIn("validation evidence is only asserted", i7_review_prompt)
        self.assertIn("seed handling", i4_review_prompt)

    def test_task_run_deep_research_raises_revision_floor(self) -> None:
        class DeepRevisionOrchestrator(MockOrchestrator):
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
                if "Review the draft" in prompt:
                    self.review_count += 1
                    content = "Verdict: BLOCK\nConfidence: 0.4\nCritical Issues: Need more evidence depth"
                    if self.review_count >= 4:
                        content = "Verdict: PASS\nConfidence: 0.9\nCritical Issues: resolved"
                    self.runtime_calls.append({
                        "agent": agent_name,
                        "prompt": prompt,
                        "runtime_options": dict(runtime_options or {}),
                        "profile_directive": profile_directive or "",
                        "cwd": str(cwd),
                    })
                    return BridgeResponse(success=True, model=agent_name, content=content)
                return super()._execute_runtime_agent(
                    agent_name, prompt, cwd, runtime_options, profile_directive
                )

        orchestrator = DeepRevisionOrchestrator()
        result = orchestrator.task_run(
            task_id="F3",
            paper_type="empirical",
            topic="deep-revision",
            cwd=REPO_ROOT,
            research_depth="deep",
            max_revision_rounds=1,
        )

        self.assertEqual(orchestrator.review_count, 4)
        self.assertIn("Round 3: PASS", result.merged_analysis)
        self.assertIn("Research depth: deep (evidence_expansion_rounds=2).", result.merged_analysis)

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

    def test_task_run_injects_domain_profile_for_code_tasks(self) -> None:
        orchestrator = MockOrchestrator()
        result = orchestrator.task_run(
            task_id="I1",
            paper_type="methods",
            topic="llm-bias",
            cwd=REPO_ROOT,
            domain="cs",
        )

        self.assertIn('"domain": "cs-ai"', result.merged_analysis)
        self.assertIn("Domain profile: requested=cs, resolved=cs-ai, status=loaded.", result.merged_analysis)
        draft_prompts = [
            call["prompt"]
            for call in orchestrator.runtime_calls
            if "You are executing one canonical research workflow task." in call["prompt"]
        ]
        self.assertTrue(draft_prompts)
        draft_prompt = draft_prompts[0]
        self.assertIn("Domain profile guidance:", draft_prompt)
        self.assertIn("Computer Science & Artificial Intelligence", draft_prompt)
        self.assertIn("Academic code lane rules:", draft_prompt)
        self.assertIn("Treat this as academic research code", draft_prompt)

    def test_task_run_extracts_stage_i_structured_output(self) -> None:
        class StructuredPlanOrchestrator(MockOrchestrator):
            def _execute_runtime_agent(
                self,
                agent_name: str,
                prompt: str,
                cwd: Path,
                runtime_options: dict[str, Any] | None = None,
                profile_directive: str | None = None,
            ) -> BridgeResponse:
                if "You are executing one canonical research workflow task." in prompt and '"task_id": "I6"' in prompt:
                    content = """---
task_id: I6
template_type: code_plan
topic: llm-bias
primary_artifact: code/plan.md
---

# Execution Plan

## Plan Contract Block
```json
{
  "task_id": "I6",
  "topic": "llm-bias",
  "spec_source": "code/code_specification.md",
  "plan_artifact": "code/plan.md",
  "steps": [
    {
      "step_id": "S1",
      "depends_on": [],
      "owner": "codex",
      "command": "python run.py",
      "outputs": ["analysis/results.csv"],
      "checkpoint": "results.csv exists",
      "rollback": "git restore analysis/results.csv"
    }
  ]
}
```

## Scope Lock
- locked
"""
                    self.runtime_calls.append(
                        {
                            "agent": agent_name,
                            "prompt": prompt,
                            "runtime_options": dict(runtime_options or {}),
                            "profile_directive": profile_directive or "",
                            "cwd": str(cwd),
                        }
                    )
                    return BridgeResponse(success=True, model=agent_name, content=content)
                if "Review the draft" in prompt:
                    self.runtime_calls.append(
                        {
                            "agent": agent_name,
                            "prompt": prompt,
                            "runtime_options": dict(runtime_options or {}),
                            "profile_directive": profile_directive or "",
                            "cwd": str(cwd),
                        }
                    )
                    return BridgeResponse(
                        success=True,
                        model=agent_name,
                        content="Verdict: PASS\nConfidence: 0.91\nCritical Issues: resolved",
                    )
                return super()._execute_runtime_agent(
                    agent_name,
                    prompt,
                    cwd,
                    runtime_options,
                    profile_directive,
                )

        orchestrator = StructuredPlanOrchestrator()
        result = orchestrator.task_run(
            task_id="I6",
            paper_type="methods",
            topic="llm-bias",
            cwd=REPO_ROOT,
        )

        self.assertIn("## Structured Artifact Summary", result.merged_analysis)
        self.assertIn("- valid: yes", result.merged_analysis)
        structured = result.data["structured_output"]
        self.assertTrue(structured["valid"])
        self.assertEqual(structured["frontmatter"]["template_type"], "code_plan")
        self.assertEqual(structured["contract"]["steps"][0]["step_id"], "S1")
        self.assertEqual(structured["actionable_targets"], ["S1"])

    def test_task_run_targeted_follow_up_reads_existing_stage_artifact(self) -> None:
        existing_artifact = """---
task_id: I6
template_type: code_plan
topic: llm-bias
primary_artifact: code/plan.md
---

# Execution Plan

## Plan Contract Block
```json
{
  "task_id": "I6",
  "topic": "llm-bias",
  "spec_source": "code/code_specification.md",
  "plan_artifact": "code/plan.md",
  "steps": [
    {
      "step_id": "S1",
      "depends_on": [],
      "owner": "codex",
      "command": "python run_stage_one.py",
      "outputs": ["analysis/results_a.csv"],
      "checkpoint": "results_a.csv exists",
      "rollback": "git restore analysis/results_a.csv"
    },
    {
      "step_id": "S2",
      "depends_on": ["S1"],
      "owner": "claude",
      "command": "python run_stage_two.py",
      "outputs": ["analysis/results_b.csv"],
      "checkpoint": "results_b.csv exists",
      "rollback": "git restore analysis/results_b.csv"
    }
  ]
}
```

## Scope Lock
- locked
"""

        revised_artifact = """---
task_id: I6
template_type: code_plan
topic: llm-bias
primary_artifact: code/plan.md
---

# Execution Plan

## Plan Contract Block
```json
{
  "task_id": "I6",
  "topic": "llm-bias",
  "spec_source": "code/code_specification.md",
  "plan_artifact": "code/plan.md",
  "steps": [
    {
      "step_id": "S1",
      "depends_on": [],
      "owner": "codex",
      "command": "python run_stage_one.py --rerun",
      "outputs": ["analysis/results_a.csv"],
      "checkpoint": "results_a.csv refreshed",
      "rollback": "git restore analysis/results_a.csv"
    }
  ]
}
```

## Scope Lock
- locked
"""

        class TargetedPlanOrchestrator(MockOrchestrator):
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
                        "prompt": prompt,
                        "runtime_options": dict(runtime_options or {}),
                        "profile_directive": profile_directive or "",
                        "cwd": str(cwd),
                    }
                )
                if "You are executing one canonical research workflow task." in prompt:
                    return BridgeResponse(success=True, model=agent_name, content=revised_artifact)
                if "Review the draft" in prompt:
                    return BridgeResponse(
                        success=True,
                        model=agent_name,
                        content="Verdict: PASS\nConfidence: 0.92\nCritical Issues: none",
                    )
                return BridgeResponse(success=True, model=agent_name, content=f"{agent_name} ok")

        with tempfile.TemporaryDirectory() as tmp_dir:
            workdir = Path(tmp_dir)
            artifact_path = workdir / "RESEARCH" / "llm-bias" / "code" / "plan.md"
            artifact_path.parent.mkdir(parents=True, exist_ok=True)
            artifact_path.write_text(existing_artifact, encoding="utf-8")

            orchestrator = TargetedPlanOrchestrator()
            result = orchestrator.task_run(
                task_id="I6",
                paper_type="methods",
                topic="llm-bias",
                cwd=workdir,
                only_targets=["S1"],
            )

        draft_prompts = [
            call["prompt"]
            for call in orchestrator.runtime_calls
            if "You are executing one canonical research workflow task." in call["prompt"]
        ]
        self.assertTrue(draft_prompts)
        self.assertIn("Targeted follow-up mode is active.", draft_prompts[0])
        self.assertIn("- selected_actionable_targets: S1", draft_prompts[0])
        self.assertIn('"step_id": "S1"', draft_prompts[0])
        targeted = result.data["targeted_follow_up"]
        self.assertTrue(targeted["enabled"])
        self.assertEqual(targeted["selected_targets"], ["S1"])
        self.assertEqual(targeted["source_artifact"], "code/plan.md")

    def test_code_build_focus_maps_to_stage_i_task(self) -> None:
        class CapturingCodeBuildOrchestrator(MockOrchestrator):
            def __init__(self) -> None:
                super().__init__()
                self.task_run_calls: list[dict[str, Any]] = []

            def task_run(self, **kwargs: Any) -> CollaborationResult:  # type: ignore[override]
                self.task_run_calls.append(dict(kwargs))
                return CollaborationResult(
                    mode="task-run",
                    task_description="captured",
                    merged_analysis='{"stage":"ok"}',
                    confidence=0.88,
                    data={"task_id": kwargs["task_id"]},
                )

        orchestrator = CapturingCodeBuildOrchestrator()
        result = orchestrator.code_build(
            method="Staggered DID",
            cwd=REPO_ROOT,
            domain="econ",
            topic="policy-effects",
            focus="code_specification",
            paper_type="methods",
        )

        self.assertEqual(len(orchestrator.task_run_calls), 1)
        call = orchestrator.task_run_calls[0]
        self.assertEqual(call["task_id"], "I5")
        self.assertEqual(call["domain"], "econ")
        self.assertEqual(call["profile"], "focused-delivery")
        self.assertEqual(call["review_profile"], "strict-review")
        self.assertEqual(result.mode, "code-build")
        self.assertIn("- mapped_task_id: I5", result.merged_analysis)

    def test_code_build_full_runs_strict_sequence(self) -> None:
        class FullSequenceOrchestrator(MockOrchestrator):
            def __init__(self) -> None:
                super().__init__()
                self.task_run_calls: list[dict[str, Any]] = []

            def task_run(self, **kwargs: Any) -> CollaborationResult:  # type: ignore[override]
                self.task_run_calls.append(dict(kwargs))
                task_id = kwargs["task_id"]
                actionable = {
                    "I5": ["decision-1"],
                    "I6": ["S1"],
                    "I7": ["S1"],
                    "I8": ["P1-01"],
                }
                return CollaborationResult(
                    mode="task-run",
                    task_description=task_id,
                    merged_analysis=f"{task_id} ok",
                    confidence=0.8,
                    data={
                        "task_id": task_id,
                        "structured_output": {
                            "task_id": task_id,
                            "valid": True,
                            "actionable_targets": actionable.get(task_id, []),
                        },
                    },
                )

        orchestrator = FullSequenceOrchestrator()
        result = orchestrator.code_build(
            method="Transformer Fine-Tuning",
            cwd=REPO_ROOT,
            domain="cs",
            topic="llm-bias",
            focus="full",
            paper_type="methods",
            triad=True,
        )

        self.assertEqual(
            [call["task_id"] for call in orchestrator.task_run_calls],
            ["I5", "I6", "I7", "I8"],
        )
        self.assertEqual(orchestrator.task_run_calls[-1]["triad"], True)
        self.assertEqual(orchestrator.task_run_calls[0]["triad"], False)
        self.assertEqual(result.mode, "code-build")
        self.assertIn("- strict_stage_sequence: I5 -> I6 -> I7 -> I8", result.merged_analysis)
        self.assertIn("I5", result.data["structured_stage_outputs"])
        self.assertEqual(result.data["actionable_targets"]["I5"], ["decision-1"])

    def test_code_build_full_only_targets_routes_selected_stages(self) -> None:
        class TargetedSequenceOrchestrator(MockOrchestrator):
            def __init__(self) -> None:
                super().__init__()
                self.task_run_calls: list[dict[str, Any]] = []

            def task_run(self, **kwargs: Any) -> CollaborationResult:  # type: ignore[override]
                self.task_run_calls.append(dict(kwargs))
                task_id = kwargs["task_id"]
                selected_targets = [
                    str(item)
                    for item in kwargs.get("only_targets", []) or []
                    if str(item).strip()
                ]
                return CollaborationResult(
                    mode="task-run",
                    task_description=task_id,
                    merged_analysis=f"{task_id} targeted",
                    confidence=0.86,
                    data={
                        "task_id": task_id,
                        "structured_output": {
                            "task_id": task_id,
                            "valid": True,
                            "actionable_targets": selected_targets,
                        },
                    },
                )

        orchestrator = TargetedSequenceOrchestrator()
        result = orchestrator.code_build(
            method="Transformer Fine-Tuning",
            cwd=REPO_ROOT,
            domain="cs",
            topic="llm-bias",
            focus="full",
            paper_type="methods",
            only_targets=["I5:decision-1", "I8:P1-01"],
        )

        self.assertEqual(
            [call["task_id"] for call in orchestrator.task_run_calls],
            ["I5", "I8"],
        )
        self.assertEqual(orchestrator.task_run_calls[0]["only_targets"], ["decision-1"])
        self.assertEqual(orchestrator.task_run_calls[1]["only_targets"], ["P1-01"])
        self.assertEqual(
            result.data["selected_target_map"],
            {"I5": ["decision-1"], "I8": ["P1-01"]},
        )
        self.assertIn("- selected_target_map: I5=decision-1; I8=P1-01", result.merged_analysis)

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
                        "prompt": prompt,
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
        orchestrator._execute_runtime_agent = ModelOrchestrator._execute_runtime_agent.__get__(
            orchestrator,
            ModelOrchestrator,
        )
        orchestrator.interactive = True 

        with mock.patch.object(
            orchestrator.codex,
            "execute",
            return_value=BridgeResponse(success=True, model="codex", content="ok"),
        ) as mock_execute:
            result = orchestrator.execute(
                mode=CollaborationMode.SINGLE,
                cwd=REPO_ROOT,
                prompt="test interactive bypass",
                single_model="codex",
            )
        self.assertTrue(result.codex_response.success)
        mock_execute.assert_called_once()

    @unittest.mock.patch("sys.stdin.isatty", return_value=True)
    @unittest.mock.patch("builtins.input", side_effect=["y"])
    def test_interactive_mode_proceeds_on_yes(self, mock_input, mock_isatty) -> None:
        """Ensure interactive mode runs the agent when user types Y."""
        orchestrator = MockOrchestrator()
        orchestrator._execute_runtime_agent = ModelOrchestrator._execute_runtime_agent.__get__(
            orchestrator,
            ModelOrchestrator,
        )
        orchestrator.interactive = True 

        with mock.patch.object(
            orchestrator.codex,
            "execute",
            return_value=BridgeResponse(success=True, model="codex", content="ok"),
        ) as mock_execute:
            result = orchestrator.execute(
                mode=CollaborationMode.SINGLE,
                cwd=REPO_ROOT,
                prompt="test interactive yes",
                single_model="codex",
            )

        self.assertTrue(result.codex_response.success)
        mock_execute.assert_called_once()
        mock_input.assert_called_once()

    @unittest.mock.patch("sys.stdin.isatty", return_value=True)
    @unittest.mock.patch("builtins.input", side_effect=["n"])
    def test_interactive_mode_skips_on_no(self, mock_input, mock_isatty) -> None:
        """Ensure interactive mode aborts execution when user types n."""
        orchestrator = MockOrchestrator()
        orchestrator._execute_runtime_agent = ModelOrchestrator._execute_runtime_agent.__get__(orchestrator, ModelOrchestrator)
        orchestrator.interactive = True 

        with mock.patch.object(orchestrator.codex, "execute") as mock_execute:
            result = orchestrator.execute(
                mode=CollaborationMode.SINGLE,
                cwd=REPO_ROOT,
                prompt="test interactive no",
                single_model="codex",
            )

        self.assertFalse(result.codex_response.success)
        self.assertIn("Skipped by user", result.codex_response.error)
        mock_execute.assert_not_called()
        mock_input.assert_called_once()

    # ── Team-Run Tests ───────────────────────────────────────────────────

    def test_team_run_loads_config(self) -> None:
        """Verify _load_team_run_config parses B1 and H3 from capability map."""
        orchestrator = MockOrchestrator()

        b1_config = orchestrator._load_team_run_config("B1")
        self.assertEqual(b1_config["task_id"], "B1")
        self.assertEqual(b1_config["execution_mode"], "fanout_merge")
        self.assertEqual(b1_config["partition_strategy"], "by_paper_batch")
        self.assertEqual(b1_config["max_parallel_units"], 5)
        self.assertIn("codex", b1_config["worker_pool"])
        self.assertEqual(b1_config["barrier_rules"]["on_failure"], "degrade")
        self.assertAlmostEqual(b1_config["barrier_rules"]["min_success_ratio"], 0.6)
        self.assertTrue(len(b1_config["shard_outputs"]) > 0)
        self.assertTrue(len(b1_config["canonical_outputs"]) > 0)

        h3_config = orchestrator._load_team_run_config("H3")
        self.assertEqual(h3_config["task_id"], "H3")
        self.assertEqual(h3_config["partition_strategy"], "by_reviewer_persona")
        self.assertEqual(h3_config["max_parallel_units"], 3)
        self.assertEqual(h3_config["barrier_rules"]["on_failure"], "block")
        self.assertAlmostEqual(h3_config["barrier_rules"]["min_success_ratio"], 1.0)
        self.assertTrue(len(h3_config["personas"]) == 3)
        persona_ids = [p["id"] for p in h3_config["personas"]]
        self.assertIn("methodologist", persona_ids)
        self.assertIn("domain_expert", persona_ids)
        self.assertIn("reviewer_2", persona_ids)

    def test_team_run_b1_mock_e2e(self) -> None:
        """B1 team-run: planner generates units, workers fan out, merge + review."""
        orchestrator = MockOrchestrator()

        # Override _execute_runtime_agent to return planner-style JSON for first call
        call_count = {"n": 0}
        original_execute = orchestrator._execute_runtime_agent

        def mock_execute(agent_name, prompt, cwd, runtime_options=None, profile_directive=None):
            call_count["n"] += 1
            if call_count["n"] == 1:
                # Planner call — return JSON work units
                return BridgeResponse(
                    success=True,
                    model=agent_name,
                    content='[{"unit_id": "batch_1", "description": "AI pedagogy", "scope": "papers on AI teaching"},'
                            '{"unit_id": "batch_2", "description": "AI assessment", "scope": "papers on AI evaluation"}]',
                )
            # All other calls (workers, merge, review)
            return original_execute(agent_name, prompt, cwd, runtime_options, profile_directive)

        orchestrator._execute_runtime_agent = mock_execute

        result = orchestrator.team_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="ai-in-education",
            cwd=REPO_ROOT,
        )

        self.assertEqual(result.mode, "team-run")
        self.assertGreater(result.confidence, 0.0)
        self.assertIn("Team-Run: B1", result.merged_analysis)
        self.assertIn("batch_1", result.merged_analysis)
        self.assertIn("batch_2", result.merged_analysis)
        # Planner + 2 workers + 1 merge + 1 review = 5 calls
        self.assertEqual(call_count["n"], 5)

    def test_team_run_h3_persona_static_units(self) -> None:
        """H3 uses static persona-based partitioning — no planner agent call."""
        orchestrator = MockOrchestrator()

        result = orchestrator.team_run(
            task_id="H3",
            paper_type="empirical",
            topic="digital-literacy",
            cwd=REPO_ROOT,
        )

        self.assertEqual(result.mode, "team-run")
        self.assertGreater(result.confidence, 0.0)
        self.assertIn("Team-Run: H3", result.merged_analysis)
        self.assertIn("methodologist", result.merged_analysis)
        self.assertIn("domain_expert", result.merged_analysis)
        self.assertIn("reviewer_2", result.merged_analysis)
        # 3 workers + 1 merge + 1 review = 5 calls (no planner)
        self.assertEqual(len(orchestrator.runtime_calls), 5)

    def test_team_run_single_worker_failure_degrades(self) -> None:
        """B1 degrade policy: 2/3 workers succeed → merge proceeds."""
        orchestrator = MockOrchestrator()

        call_count = {"n": 0}

        def mock_execute(agent_name, prompt, cwd, runtime_options=None, profile_directive=None):
            call_count["n"] += 1
            if call_count["n"] == 1:
                # Planner
                return BridgeResponse(
                    success=True, model=agent_name,
                    content='[{"unit_id": "b1", "description": "a", "scope": "s1"},'
                            '{"unit_id": "b2", "description": "b", "scope": "s2"},'
                            '{"unit_id": "b3", "description": "c", "scope": "s3"}]',
                )
            if call_count["n"] == 3:
                # Third call (second worker) fails
                return BridgeResponse(success=False, model=agent_name, error="timeout")
            return BridgeResponse(success=True, model=agent_name, content=f"output-{call_count['n']}")

        orchestrator._execute_runtime_agent = mock_execute

        result = orchestrator.team_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="test-degrade",
            cwd=REPO_ROOT,
        )

        self.assertIn("degraded", result.merged_analysis.lower())
        self.assertGreater(result.confidence, 0.0)

    def test_team_run_all_workers_fail_blocks(self) -> None:
        """When all workers fail, confidence should be 0."""
        orchestrator = MockOrchestrator()

        call_count = {"n": 0}

        def mock_execute(agent_name, prompt, cwd, runtime_options=None, profile_directive=None):
            call_count["n"] += 1
            if call_count["n"] == 1:
                return BridgeResponse(
                    success=True, model=agent_name,
                    content='[{"unit_id": "b1", "description": "a", "scope": "s1"}]',
                )
            return BridgeResponse(success=False, model=agent_name, error="all-fail")

        orchestrator._execute_runtime_agent = mock_execute

        result = orchestrator.team_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="test-block",
            cwd=REPO_ROOT,
        )

        self.assertEqual(result.confidence, 0.0)
        self.assertIn("blocked", result.merged_analysis.lower())

    def test_team_run_does_not_affect_task_run(self) -> None:
        """Existing task-run behavior must remain unchanged after team-run addition."""
        orchestrator = MockOrchestrator()

        result = orchestrator.task_run(
            task_id="F3",
            paper_type="empirical",
            topic="regression-test",
            cwd=REPO_ROOT,
        )

        self.assertEqual(result.mode, "task-run")
        self.assertGreater(result.confidence, 0.0)
        self.assertTrue(len(orchestrator.runtime_calls) >= 2)


if __name__ == "__main__":
    unittest.main(verbosity=2)
