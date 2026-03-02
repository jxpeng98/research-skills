#!/usr/bin/env python3
"""
Multi-Model Orchestrator for research-skills.
Single entry point for coordinating Codex, Claude, and Gemini collaboration.

Usage:
    python orchestrator.py parallel --prompt "..." --cwd "/path" --summarizer claude
    python orchestrator.py chain --prompt "..." --cwd "/path" --generator claude
    python orchestrator.py role --cwd "/path" --codex-task "..." --claude-task "..." --gemini-task "..."
    python orchestrator.py single --model claude --prompt "..." --cwd "/path"
    python orchestrator.py task-run --task-id F3 --paper-type empirical --topic ai-in-education --cwd "/path" --mcp-strict --skills-strict --triad
    python orchestrator.py doctor --cwd "/path"

Python 3.10+ required.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import shutil
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import asdict
from enum import Enum
from pathlib import Path
from typing import Any

from .base_bridge import BridgeResponse, CollaborationResult, configure_stdio
from .claude_bridge import ClaudeBridge
from .codex_bridge import CodexBridge
from .gemini_bridge import GeminiBridge
from .mcp_connectors import MCPEvidence, MCPConnector
from .i18n import get_text
from .errors import (
    ResearchError,
    ConfigError,
    ERR_CFG_INVALID_PROFILE,
    ERR_CFG_MISSING_STANDARD,
    ERR_CFG_INVALID_TASK,
    ERR_EXE_PARALLEL_FAIL,
    ERR_MCP_PROVIDER_MISSING
)


class CollaborationMode(Enum):
    """Collaboration mode types."""
    PARALLEL = "parallel"      # Both models analyze simultaneously
    CHAIN = "chain"            # One generates, other verifies
    ROLE_BASED = "role"        # Task division by specialty
    SINGLE = "single"          # Single model execution


class AcademicTaskType(Enum):
    """Academic research task types for intelligent routing."""
    ALGORITHM_IMPL = "algorithm"       # Implement algorithm from paper
    CODE_REPRODUCE = "reproduce"       # Reproduce paper results
    CODE_ANALYSIS = "analyze"          # Analyze existing codebase
    DATA_PIPELINE = "data"             # Data processing pipeline
    EXPERIMENT = "experiment"          # Experiment design/execution


class ModelOrchestrator:
    """
    Orchestrates multi-model collaboration for academic research tasks.
    
    Supports three collaboration patterns:
    1. Parallel: Both models analyze, merge high-confidence conclusions
    2. Chain: One generates, another verifies (iterative refinement)
    3. Role-based: Task division by model specialty
    """
    
    # Model specialties for intelligent routing
    CODEX_STRENGTHS = [
        "code generation",
        "algorithm implementation",
        "bug fixing",
        "refactoring",
        "performance optimization",
    ]
    
    GEMINI_STRENGTHS = [
        "code explanation",
        "architecture analysis",
        "documentation",
        "design review",
        "research context",
    ]
    CLAUDE_STRENGTHS = [
        "long-form reasoning",
        "manuscript drafting",
        "argument structure",
        "quality critique",
        "cross-section consistency",
    ]
    DEFAULT_AGENT_PROFILES: dict[str, dict[str, Any]] = {
        "default": {
            "persona": "Balanced academic collaborator emphasizing traceability and rigor.",
            "analysis_style": "State assumptions explicitly and separate evidence from inference.",
            "draft_style": "Use concise academic prose with explicit claim-evidence links.",
            "review_style": "Prioritize high-impact risks first, then completeness and formatting.",
            "summary_style": "Synthesize consensus, disagreements, and next actions.",
            "runtime_options": {
                "codex": {"sandbox": "read-only", "non_interactive": True},
                "claude": {"permission_mode": "default", "non_interactive": True},
                "gemini": {"sandbox": False, "non_interactive": True},
            },
        },
        "strict-review": {
            "persona": "Critical reviewer with low tolerance for unsupported claims.",
            "review_style": "Block on unresolved validity, reproducibility, or reporting gaps.",
            "summary_style": "Focus on blockers, evidence gaps, and remediation order.",
            "runtime_options": {
                "codex": {"sandbox": "read-only", "non_interactive": True},
                "claude": {"permission_mode": "default", "non_interactive": True},
                "gemini": {"sandbox": True, "non_interactive": True},
            },
        },
        "rapid-draft": {
            "persona": "Fast drafting assistant optimizing iteration speed.",
            "draft_style": "Produce structured placeholders quickly and list missing inputs.",
            "summary_style": "Summarize fastest path to a complete next revision.",
            "runtime_options": {
                "codex": {"sandbox": "workspace-write", "non_interactive": True},
                "claude": {"permission_mode": "default", "non_interactive": True},
                "gemini": {"sandbox": False, "non_interactive": True},
            },
        },
    }
    RUNTIME_AGENTS = {"codex", "claude", "gemini"}
    DEFAULT_STANDARDS_DIR = Path(__file__).resolve().parents[1] / "standards"
    
    def __init__(
        self,
        codex_sandbox: str = "read-only",
        claude_permission_mode: str | None = None,
        gemini_sandbox: bool = False,
        standards_dir: Path | None = None,
        mcp_timeout_seconds: int = 20,
    ):
        """Initialize orchestrator with bridges."""
        self.codex = CodexBridge(sandbox=codex_sandbox)
        self.claude = ClaudeBridge(permission_mode=claude_permission_mode)
        self.gemini = GeminiBridge(sandbox=gemini_sandbox)
        self.standards_dir = standards_dir or self.DEFAULT_STANDARDS_DIR
        self.mcp_connector = MCPConnector(timeout_seconds=mcp_timeout_seconds)

    def _deep_merge_dict(self, base: dict[str, Any], override: dict[str, Any]) -> dict[str, Any]:
        merged: dict[str, Any] = dict(base)
        for key, value in override.items():
            base_value = merged.get(key)
            if isinstance(base_value, dict) and isinstance(value, dict):
                merged[key] = self._deep_merge_dict(base_value, value)
            else:
                merged[key] = value
        return merged

    def _load_profile_bundle(
        self,
        profile_file: Path | None,
    ) -> tuple[dict[str, dict[str, Any]], dict[str, dict[str, str]]]:
        profiles = json.loads(json.dumps(self.DEFAULT_AGENT_PROFILES))
        task_overrides: dict[str, dict[str, str]] = {}
        if profile_file is None:
            return profiles, task_overrides
        target = profile_file
        if not target.is_absolute():
            target = Path.cwd() / target
        if not target.exists():
            raise ValueError(f"Profile file not found: {target}")
        try:
            payload = json.loads(target.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as exc:
            raise ValueError(f"Failed to load profile file {target}: {exc}") from exc
        if not isinstance(payload, dict):
            raise ValueError("Profile file root must be a JSON object.")

        payload_profiles = payload.get("profiles", {})
        if not isinstance(payload_profiles, dict):
            raise ValueError("Profile file field 'profiles' must be a JSON object.")
        for name, config in payload_profiles.items():
            if not isinstance(name, str) or not name.strip():
                raise ValueError("Profile name must be a non-empty string.")
            if not isinstance(config, dict):
                raise ValueError(f"Profile '{name}' must be a JSON object.")
            existing = profiles.get(name, {})
            profiles[name] = self._deep_merge_dict(existing, config)

        payload_overrides = payload.get("task_overrides", {})
        if payload_overrides:
            if not isinstance(payload_overrides, dict):
                raise ValueError("Profile file field 'task_overrides' must be a JSON object.")
            for task_id, mapping in payload_overrides.items():
                if not isinstance(mapping, dict):
                    raise ValueError(f"Task override '{task_id}' must be a JSON object.")
                normalized_map: dict[str, str] = {}
                for key in ("profile", "draft_profile", "review_profile", "triad_profile"):
                    value = mapping.get(key)
                    if value is None:
                        continue
                    if not isinstance(value, str) or not value.strip():
                        raise ValueError(
                            f"Task override {task_id}.{key} must be a non-empty string."
                        )
                    normalized_map[key] = value.strip()
                task_overrides[str(task_id).strip().upper()] = normalized_map
        return profiles, task_overrides

    def _resolve_profile_config(
        self,
        profile_name: str,
        profile_registry: dict[str, dict[str, Any]],
    ) -> dict[str, Any]:
        name = profile_name.strip()
        config = profile_registry.get(name)
        if config is None:
            err_msg = get_text("unknown_profile", profile=name)
            raise ConfigError(
                ERR_CFG_INVALID_PROFILE,
                detail=f"{err_msg}. Available: {', '.join(sorted(profile_registry.keys()))}"
            )
        if not isinstance(config, dict):
            raise ConfigError(ERR_CFG_INVALID_PROFILE, detail=f"Invalid profile config format: {name}")
        return config

    def _resolve_task_profile_names(
        self,
        task_id: str,
        task_overrides: dict[str, dict[str, str]],
        base_profile: str,
        draft_profile: str | None,
        review_profile: str | None,
        triad_profile: str | None,
    ) -> dict[str, str]:
        override = task_overrides.get(task_id.upper(), {})
        selected_base = (override.get("profile") or base_profile or "default").strip()
        selected_draft = (draft_profile or override.get("draft_profile") or selected_base).strip()
        selected_review = (review_profile or override.get("review_profile") or selected_base).strip()
        selected_triad = (triad_profile or override.get("triad_profile") or selected_base).strip()
        return {
            "base": selected_base,
            "draft": selected_draft,
            "review": selected_review,
            "triad": selected_triad,
        }

    def _profile_runtime_options(
        self,
        profile_cfg: dict[str, Any],
        agent_name: str,
    ) -> dict[str, Any]:
        runtime = profile_cfg.get("runtime_options", {})
        if not isinstance(runtime, dict):
            return {}
        agent_opts = runtime.get(agent_name, {})
        if not isinstance(agent_opts, dict):
            return {}
        return dict(agent_opts)

    def _build_profile_directive(
        self,
        profile_name: str,
        profile_cfg: dict[str, Any],
        stage: str,
    ) -> str:
        style_map = {
            "analysis": "analysis_style",
            "draft": "draft_style",
            "review": "review_style",
            "summary": "summary_style",
            "triad": "triad_style",
        }
        persona = str(profile_cfg.get("persona", "")).strip()
        style_key = style_map.get(stage, "analysis_style")
        stage_style = str(profile_cfg.get(style_key, "")).strip()
        if not stage_style and stage == "triad":
            stage_style = str(profile_cfg.get("summary_style", "")).strip()
        if not persona and not stage_style:
            lines = []
        else:
            lines = [f"Agent Profile: {profile_name} (stage: {stage})"]
            if persona:
                lines.append(f"- Persona: {persona}")
            if stage_style:
                lines.append(f"- Style: {stage_style}")
        
        output_language = str(profile_cfg.get("output_language", "")).strip()
        if output_language:
            if not lines:
                lines = [f"Agent Profile: {profile_name} (stage: {stage})"]
            lines.append(
                f"- Output Language Directive: You MUST output the final response in {output_language}, "
                "but strictly adhere to the English constraints, structures, and schemas provided."
            )
        
        if not lines:
            return ""
        return "\n".join(lines)
    
    def execute(
        self,
        mode: CollaborationMode,
        cwd: Path,
        prompt: str | None = None,
        codex_task: str | None = None,
        claude_task: str | None = None,
        gemini_task: str | None = None,
        generator: str = "codex",
        single_model: str = "codex",
        parallel_summarizer: str = "claude",
        profile_file: Path | None = None,
        profile: str = "default",
        summarizer_profile: str | None = None,
        session_id: str | None = None,
    ) -> CollaborationResult:
        """
        Execute collaboration based on mode.
        
        Args:
            mode: Collaboration mode
            cwd: Working directory
            prompt: Main prompt (for parallel/chain/single modes)
            codex_task: Codex-specific task (for role mode)
            claude_task: Claude-specific task (for role mode)
            gemini_task: Gemini-specific task (for role mode)
            generator: Which model generates in chain mode
            single_model: Which model to use in single mode
            parallel_summarizer: Which model performs post-parallel synthesis
            profile_file: Optional JSON profile bundle path
            profile: Base profile for this run
            summarizer_profile: Profile used by parallel summarizer
            session_id: Resume existing session
        """
        if mode == CollaborationMode.PARALLEL:
            try:
                profile_registry, _ = self._load_profile_bundle(profile_file)
                return self._parallel_analyze(
                    prompt or "",
                    cwd,
                    summarizer=parallel_summarizer,
                    profile_registry=profile_registry,
                    base_profile_name=profile,
                    summarizer_profile_name=summarizer_profile or profile,
                )
            except ValueError as exc:
                error_resp = BridgeResponse.from_error("orchestrator", str(exc))
                return CollaborationResult(
                    mode="parallel",
                    task_description=(prompt or "")[:200],
                    merged_analysis=str(exc),
                    confidence=0.0,
                    recommendations=[],
                    codex_response=error_resp if "codex" in str(exc).lower() else None,
                    claude_response=error_resp if "claude" in str(exc).lower() else None,
                    gemini_response=error_resp if "gemini" in str(exc).lower() else None,
                )
        elif mode == CollaborationMode.CHAIN:
            return self._chain_verify(prompt or "", cwd, generator)
        elif mode == CollaborationMode.ROLE_BASED:
            return self._role_based(cwd, codex_task, claude_task, gemini_task)
        elif mode == CollaborationMode.SINGLE:
            return self._single_execute(
                prompt or "", cwd, single_model, session_id
            )
        else:
            raise ValueError(f"Unknown collaboration mode: {mode}")
    
    def _parallel_analyze(
        self,
        prompt: str,
        cwd: Path,
        summarizer: str = "claude",
        profile_registry: dict[str, dict[str, Any]] | None = None,
        base_profile_name: str = "default",
        summarizer_profile_name: str = "default",
    ) -> CollaborationResult:
        """
        Parallel mode: 3-agent concurrent analysis (codex/claude/gemini),
        then one summarizer agent performs cross-model synthesis.

        If triad is unavailable, automatically degrade to dual/single.
        """
        requested_agents = ["codex", "claude", "gemini"]
        responses: dict[str, BridgeResponse] = {}
        registry = profile_registry or self.DEFAULT_AGENT_PROFILES
        base_profile_cfg = self._resolve_profile_config(base_profile_name, registry)
        summary_profile_cfg = self._resolve_profile_config(summarizer_profile_name, registry)
        analysis_directive = self._build_profile_directive(
            base_profile_name,
            base_profile_cfg,
            stage="analysis",
        )

        with ThreadPoolExecutor(max_workers=len(requested_agents)) as executor:
            futures = {
                executor.submit(
                    self._execute_runtime_agent,
                    agent,
                    prompt,
                    cwd,
                    self._profile_runtime_options(base_profile_cfg, agent),
                    analysis_directive,
                ): agent
                for agent in requested_agents
            }
            for future in as_completed(futures):
                agent = futures[future]
                try:
                    responses[agent] = future.result()
                except Exception as exc:
                    responses[agent] = BridgeResponse.from_error(
                        agent,
                        f"Parallel execution error: {exc}",
                    )

        success_agents = [
            agent
            for agent in requested_agents
            if agent in responses and responses[agent].success
        ]
        failed_agents = [
            agent
            for agent in requested_agents
            if agent not in success_agents
        ]
        if len(success_agents) >= 3:
            execution_level = "triad"
        elif len(success_agents) == 2:
            execution_level = "dual"
        elif len(success_agents) == 1:
            execution_level = "single"
        else:
            execution_level = "failed"

        synthesis_runtime = ""
        synthesis_notes: list[str] = []
        synthesis_resp: BridgeResponse | None = None
        if success_agents:
            preferred = summarizer if summarizer in self.RUNTIME_AGENTS else "claude"
            if preferred in success_agents:
                synthesis_runtime = preferred
            else:
                synthesis_runtime = success_agents[0]
                synthesis_notes.append(
                    f"Parallel summarizer '{preferred}' unavailable; "
                    f"fallback to '{synthesis_runtime}'."
                )
            synthesis_prompt = self._build_parallel_synthesis_prompt(
                original_prompt=prompt,
                success_agents=success_agents,
                failed_agents=failed_agents,
                responses=responses,
            )
            summary_directive = self._build_profile_directive(
                summarizer_profile_name,
                summary_profile_cfg,
                stage="summary",
            )
            synthesis_resp = self._execute_runtime_agent(
                synthesis_runtime,
                synthesis_prompt,
                cwd,
                self._profile_runtime_options(summary_profile_cfg, synthesis_runtime),
                summary_directive,
            )

        parallel_header = get_text("parallel_execution") if execution_level == "full" else get_text("parallel_execution_dual")
        merged_parts: list[str] = [
            parallel_header,
            f"- Requested agents: {', '.join(requested_agents)}",
            f"- Successful agents: {', '.join(success_agents) if success_agents else 'none'}",
            get_text("failed_agents", agents=', '.join(failed_agents) if failed_agents else 'none'),
            f"- Base profile: {base_profile_name}",
            f"- Summarizer profile: {summarizer_profile_name}",
        ]
        for note in synthesis_notes:
            merged_parts.append(f"- {note}")

        for agent in requested_agents:
            response = responses.get(agent)
            if not response:
                continue
            merged_parts.extend(
                [
                    "",
                    f"## {agent.capitalize()} Analysis",
                    response.content if response.success else f"[FAILED] {response.error}",
                ]
            )

        merged_parts.extend(["", get_text("synthesis")])
        if synthesis_resp and synthesis_resp.success:
            merged_parts.append(
                f"[Summarizer: {synthesis_runtime}]"
            )
            merged_parts.append(synthesis_resp.content)
        elif success_agents:
            if synthesis_resp and synthesis_resp.error:
                merged_parts.append(
                    f"[Summarizer failed: {synthesis_runtime}] {synthesis_resp.error}"
                )
            merged_parts.append(
                self._build_parallel_fallback_summary(
                    success_agents=success_agents,
                    failed_agents=failed_agents,
                    responses=responses,
                )
            )
        else:
            merged_parts.append("No successful agent analysis available.")

        merged = "\n".join(merged_parts)
        confidence = self._calculate_parallel_confidence(
            success_count=len(success_agents),
            synthesis_success=bool(synthesis_resp and synthesis_resp.success),
        )

        return CollaborationResult(
            mode="parallel",
            task_description=prompt[:200],
            codex_response=responses.get("codex"),
            claude_response=responses.get("claude"),
            gemini_response=responses.get("gemini"),
            merged_analysis=merged,
            confidence=confidence,
            recommendations=self._extract_recommendations(merged),
        )

    def _build_parallel_synthesis_prompt(
        self,
        original_prompt: str,
        success_agents: list[str],
        failed_agents: list[str],
        responses: dict[str, BridgeResponse],
    ) -> str:
        output_blocks: list[str] = []
        for agent in success_agents:
            output_blocks.append(
                f"### {agent.upper()} OUTPUT\n{responses[agent].content}"
            )
        failed_text = ", ".join(failed_agents) if failed_agents else "none"
        return f"""Synthesize multi-agent parallel analysis into one actionable conclusion.

Original task:
{original_prompt}

Successful analyzers: {", ".join(success_agents)}
Failed analyzers: {failed_text}

Analyzer outputs:
{"\n\n".join(output_blocks)}

Produce sections:
- Consensus Summary
- Key Disagreements
- Risk Assessment
- Prioritized Next Actions
- Confidence (0-1)
"""

    def _build_parallel_fallback_summary(
        self,
        success_agents: list[str],
        failed_agents: list[str],
        responses: dict[str, BridgeResponse],
    ) -> str:
        lines = [
            "Fallback summary generated because synthesis step was unavailable.",
            f"- Successful analyzers: {', '.join(success_agents)}",
            f"- Failed analyzers: {', '.join(failed_agents) if failed_agents else 'none'}",
            "- Cross-check recommendation: review disagreements manually before execution.",
        ]
        for agent in success_agents:
            content = responses.get(agent).content if responses.get(agent) else ""
            preview = content.splitlines()[0][:220] if content else ""
            if preview:
                lines.append(f"- {agent} lead point: {preview}")
        return "\n".join(lines)

    def _calculate_parallel_confidence(
        self,
        success_count: int,
        synthesis_success: bool,
    ) -> float:
        if success_count >= 3 and synthesis_success:
            return 0.93
        if success_count >= 3:
            return 0.86
        if success_count == 2 and synthesis_success:
            return 0.8
        if success_count == 2:
            return 0.72
        if success_count == 1 and synthesis_success:
            return 0.62
        if success_count == 1:
            return 0.5
        return 0.0
    
    def _chain_verify(
        self, prompt: str, cwd: Path, generator: str = "codex"
    ) -> CollaborationResult:
        """
        Chain mode: One model generates, another verifies.
        
        Best for: Algorithm implementation, code generation
        """
        verifier_by_generator = {
            "codex": "claude",
            "claude": "gemini",
            "gemini": "codex",
        }
        verify_agent = verifier_by_generator.get(generator, "claude")
        gen_resp = self._execute_runtime_agent(generator, prompt, cwd)
        verify_resp: BridgeResponse | None = None
        if gen_resp.success:
            verify_prompt = self._build_verification_prompt(gen_resp.content)
            verify_resp = self._execute_runtime_agent(verify_agent, verify_prompt, cwd)
        
        # Build merged analysis
        if gen_resp.success and verify_resp and verify_resp.success:
            merged = self._merge_with_verification(gen_resp, verify_resp)
            confidence = 0.85
        elif gen_resp.success:
            merged = f"[Generated but not verified]\n{gen_resp.content}"
            confidence = 0.5
        else:
            merged = f"Generation failed: {gen_resp.error}"
            confidence = 0.0
        
        return CollaborationResult(
            mode="chain",
            task_description=prompt[:200],
            codex_response=gen_resp if generator == "codex" else (
                verify_resp if verify_agent == "codex" else None
            ),
            claude_response=gen_resp if generator == "claude" else (
                verify_resp if verify_agent == "claude" else None
            ),
            gemini_response=gen_resp if generator == "gemini" else (
                verify_resp if verify_agent == "gemini" else None
            ),
            merged_analysis=merged,
            confidence=confidence,
            recommendations=self._extract_recommendations(merged),
        )
    
    def _role_based(
        self,
        cwd: Path,
        codex_task: str | None,
        claude_task: str | None,
        gemini_task: str | None,
    ) -> CollaborationResult:
        """
        Role-based mode: Divide tasks by model specialty.
        
        Codex: Code generation, implementation, fixing
        Claude: Structured drafting, critique, synthesis
        Gemini: Explanation, documentation, analysis
        """
        codex_resp = None
        claude_resp = None
        gemini_resp = None
        
        if codex_task:
            codex_resp = self.codex.execute(codex_task, cwd)
        
        if claude_task:
            claude_resp = self.claude.execute(claude_task, cwd)
        
        if gemini_task:
            gemini_resp = self.gemini.execute(gemini_task, cwd)
        
        # Merge outputs
        parts = []
        if codex_resp and codex_resp.success:
            parts.append(f"## Codex Output\n\n{codex_resp.content}")
        if claude_resp and claude_resp.success:
            parts.append(f"## Claude Output\n\n{claude_resp.content}")
        if gemini_resp and gemini_resp.success:
            parts.append(f"## Gemini Output\n\n{gemini_resp.content}")
        
        merged = "\n\n---\n\n".join(parts) if parts else "No successful outputs."
        
        # Calculate confidence
        success_count = sum([
            1 for r in [codex_resp, claude_resp, gemini_resp]
            if r and r.success
        ])
        requested_count = sum([
            1 if codex_task else 0,
            1 if claude_task else 0,
            1 if gemini_task else 0,
        ])
        confidence = (
            success_count / float(requested_count)
            if requested_count > 0 else 0.0
        )
        
        task_desc = (
            f"Codex: {codex_task or 'N/A'} | "
            f"Claude: {claude_task or 'N/A'} | "
            f"Gemini: {gemini_task or 'N/A'}"
        )
        
        return CollaborationResult(
            mode="role_based",
            task_description=task_desc[:200],
            codex_response=codex_resp,
            claude_response=claude_resp,
            gemini_response=gemini_resp,
            merged_analysis=merged,
            confidence=confidence,
            recommendations=[],
        )
    
    def _single_execute(
        self,
        prompt: str,
        cwd: Path,
        model: str,
        session_id: str | None = None,
    ) -> CollaborationResult:
        """Single model execution for simple tasks."""
        if model == "codex":
            resp = self.codex.execute(prompt, cwd, session_id=session_id)
            return CollaborationResult(
                mode="single",
                task_description=prompt[:200],
                codex_response=resp,
                merged_analysis=resp.content if resp.success else resp.error or "",
                confidence=1.0 if resp.success else 0.0,
            )
        if model == "claude":
            resp = self.claude.execute(prompt, cwd, session_id=session_id)
            return CollaborationResult(
                mode="single",
                task_description=prompt[:200],
                claude_response=resp,
                merged_analysis=resp.content if resp.success else resp.error or "",
                confidence=1.0 if resp.success else 0.0,
            )
        if model == "gemini":
            resp = self.gemini.execute(prompt, cwd, session_id=session_id)
            return CollaborationResult(
                mode="single",
                task_description=prompt[:200],
                gemini_response=resp,
                merged_analysis=resp.content if resp.success else resp.error or "",
                confidence=1.0 if resp.success else 0.0,
            )
        return CollaborationResult(
            mode="single",
            task_description=prompt[:200],
            merged_analysis=f"Unsupported single model: {model}",
            confidence=0.0,
            recommendations=[],
        )
    
    def _merge_analyses(self, codex: str, gemini: str) -> str:
        """Merge two model outputs into unified analysis."""
        return f"""## Parallel Analysis Results

### Codex Analysis
{codex}

---

### Gemini Analysis
{gemini}

---

### Merged Conclusions
Points mentioned by both models have high confidence.
Review differences for additional insights.
"""
    
    def _merge_with_verification(
        self, gen_resp: BridgeResponse, verify_resp: BridgeResponse
    ) -> str:
        """Merge generator output with verification feedback."""
        return f"""## Chain Verification Results

### Generated Content
{gen_resp.content}

---

### Verification Feedback
{verify_resp.content}

---

### Final Assessment
Content has been generated and verified by separate models.
"""
    
    def _build_verification_prompt(self, content: str) -> str:
        """Build verification prompt for the second model."""
        return f"""Please review and verify the following code/analysis.
Check for:
1. Correctness and accuracy
2. Potential bugs or issues
3. Improvements and optimizations
4. Consistency with best practices

Content to verify:
```
{content}
```

Provide your verification assessment.
"""
    
    def _calculate_agreement(
        self, resp1: BridgeResponse, resp2: BridgeResponse
    ) -> float:
        """Calculate agreement score between two responses."""
        if not resp1.success or not resp2.success:
            return 0.3
        
        # Simple heuristic: both succeeded = base 0.7
        # Could be enhanced with semantic similarity
        return 0.75
    
    def _extract_recommendations(self, merged: str) -> list[str]:
        """Extract actionable recommendations from merged analysis."""
        # Simple extraction - could be enhanced with NLP
        recommendations = []
        keywords = ["recommend", "suggest", "should", "consider", "improve"]
        
        for line in merged.split("\n"):
            line_lower = line.lower()
            if any(kw in line_lower for kw in keywords):
                clean = line.strip().lstrip("-*• ")
                if len(clean) > 10:
                    recommendations.append(clean)
        
        return recommendations[:5]  # Top 5

    def _standards_file(self, filename: str) -> Path:
        return self.standards_dir / filename

    def _read_standard(self, filename: str) -> str:
        path = self._standards_file(filename)
        if not path.exists():
            raise FileNotFoundError(f"Missing standards file: {path}")
        return path.read_text(encoding="utf-8")

    def _extract_top_level_section(self, content: str, key: str) -> str:
        match = re.search(rf"^{re.escape(key)}:\s*\n", content, flags=re.MULTILINE)
        if not match:
            return ""
        start = match.end()
        tail = content[start:]
        next_key = re.search(r"^[A-Za-z0-9_]+:\s*", tail, flags=re.MULTILINE)
        if not next_key:
            return tail
        return tail[: next_key.start()]

    def _extract_nested_section(self, block: str, key: str, indent: int) -> str:
        match = re.search(
            rf"^\s{{{indent}}}{re.escape(key)}:\s*\n",
            block,
            flags=re.MULTILINE,
        )
        if not match:
            return ""
        start = match.end()
        tail = block[start:]
        next_key = re.search(
            rf"^\s{{{indent}}}[A-Za-z0-9_-]+:\s*",
            tail,
            flags=re.MULTILINE,
        )
        if not next_key:
            return tail
        return tail[: next_key.start()]

    def _parse_yaml_list(self, section: str, item_indent: int) -> list[str]:
        values: list[str] = []
        for raw in re.findall(
            rf"^\s{{{item_indent}}}-\s*(.+?)\s*$",
            section,
            flags=re.MULTILINE,
        ):
            item = raw.strip()
            if (item.startswith('"') and item.endswith('"')) or (
                item.startswith("'") and item.endswith("'")
            ):
                item = item[1:-1]
            values.append(item)
        return values

    def _parse_yaml_scalar(self, block: str, key: str, indent: int) -> str:
        match = re.search(
            rf"^\s{{{indent}}}{re.escape(key)}:\s*(.+?)\s*$",
            block,
            flags=re.MULTILINE,
        )
        if not match:
            return ""
        value = match.group(1).strip()
        if (value.startswith('"') and value.endswith('"')) or (
            value.startswith("'") and value.endswith("'")
        ):
            return value[1:-1]
        return value

    def _load_task_outputs(self, task_id: str) -> tuple[str, list[str]]:
        contract = self._read_standard("research-workflow-contract.yaml")
        artifacts = self._extract_top_level_section(contract, "artifacts")
        artifact_root = self._parse_yaml_scalar(artifacts, "root", indent=2) or "RESEARCH/[topic]/"

        task_section = self._extract_top_level_section(contract, "task_catalog")
        task_match = re.search(
            rf"^\s{{2}}{re.escape(task_id)}:\n((?:^\s{{4}}.*\n?)+)",
            task_section,
            flags=re.MULTILINE,
        )
        if not task_match:
            raise ValueError(f"Task ID not found in research-workflow-contract.yaml: {task_id}")
        outputs_section = self._extract_nested_section(task_match.group(1), "outputs", indent=4)
        outputs = self._parse_yaml_list(outputs_section, item_indent=6)
        if not outputs:
            raise ValueError(f"No outputs configured for task {task_id} in contract")
        return artifact_root, outputs

    def _load_task_dependencies(self, task_id: str) -> dict[str, list[str]]:
        contract = self._read_standard("research-workflow-contract.yaml")
        dependency_section = self._extract_top_level_section(contract, "dependency_catalog")
        if not dependency_section:
            return {
                "prerequisites_all": [],
                "prerequisites_any": [],
                "recommended_prerequisites": [],
                "recommended_next": [],
            }
        task_match = re.search(
            rf"^\s{{2}}{re.escape(task_id)}:\n((?:^\s{{4}}.*\n?)+)",
            dependency_section,
            flags=re.MULTILINE,
        )
        if not task_match:
            return {
                "prerequisites_all": [],
                "prerequisites_any": [],
                "recommended_prerequisites": [],
                "recommended_next": [],
            }

        block = task_match.group(1)
        return {
            "prerequisites_all": self._parse_yaml_list(
                self._extract_nested_section(block, "prerequisites_all", indent=4),
                item_indent=6,
            ),
            "prerequisites_any": self._parse_yaml_list(
                self._extract_nested_section(block, "prerequisites_any", indent=4),
                item_indent=6,
            ),
            "recommended_prerequisites": self._parse_yaml_list(
                self._extract_nested_section(block, "recommended_prerequisites", indent=4),
                item_indent=6,
            ),
            "recommended_next": self._parse_yaml_list(
                self._extract_nested_section(block, "recommended_next", indent=4),
                item_indent=6,
            ),
        }

    def _project_root_for_topic(self, cwd: Path, artifact_root: str, topic: str) -> Path:
        return cwd / artifact_root.replace("[topic]", topic)

    def _check_task_completion(self, cwd: Path, artifact_root: str, topic: str, task_id: str) -> bool:
        _, outputs = self._load_task_outputs(task_id)
        project_root = self._project_root_for_topic(cwd, artifact_root, topic)
        for rel_path in outputs:
            target = project_root / rel_path
            if not target.exists():
                return False
        return True

    def _build_task_plan(self, task_id: str) -> dict[str, Any]:
        visited: set[str] = set()
        visiting: set[str] = set()
        ordered: list[str] = []

        def dfs(node: str) -> None:
            if node in visited:
                return
            if node in visiting:
                raise ValueError(f"Cycle detected in prerequisites_all near task: {node}")
            visiting.add(node)
            spec = self._load_task_dependencies(node)
            for prereq in spec.get("prerequisites_all", []):
                dfs(prereq)
            visiting.remove(node)
            visited.add(node)
            ordered.append(node)

        dfs(task_id)
        root_spec = self._load_task_dependencies(task_id)
        any_of_requirements = [
            {
                "task": node,
                "any_of": self._load_task_dependencies(node).get("prerequisites_any", []),
            }
            for node in ordered
            if self._load_task_dependencies(node).get("prerequisites_any", [])
        ]

        return {
            "task_id": task_id,
            "requires_all_order": ordered,
            "root_dependencies": root_spec,
            "any_of_requirements": any_of_requirements,
        }

    def task_plan(
        self,
        task_id: str,
        paper_type: str,
        topic: str,
        cwd: Path,
    ) -> CollaborationResult:
        """Render an execution plan based on contract dependency_catalog + filesystem evidence."""
        normalized_task = task_id.strip().upper()
        normalized_topic = self._normalize_topic(topic)

        try:
            artifact_root, _required_outputs = self._load_task_outputs(normalized_task)
            plan = self._build_task_plan(normalized_task)
        except (FileNotFoundError, ValueError) as exc:
            return CollaborationResult(
                mode="task-plan",
                task_description=f"{normalized_task} {paper_type} {normalized_topic}"[:200],
                merged_analysis=str(exc),
                confidence=0.0,
                recommendations=[],
            )

        project_root = self._project_root_for_topic(cwd, artifact_root, normalized_topic)

        required_chain = [node for node in plan["requires_all_order"] if node != normalized_task]
        completion: dict[str, bool] = {}
        missing_required: list[str] = []
        for node in required_chain:
            done = self._check_task_completion(cwd, artifact_root, normalized_topic, node)
            completion[node] = done
            if not done:
                missing_required.append(node)

        any_of_status: list[dict[str, Any]] = []
        missing_any_of: list[str] = []
        for item in plan["any_of_requirements"]:
            options = [str(opt) for opt in item.get("any_of", []) if str(opt).strip()]
            satisfied_by = None
            for opt in options:
                if self._check_task_completion(cwd, artifact_root, normalized_topic, opt):
                    satisfied_by = opt
                    break
            ok = bool(satisfied_by) if options else True
            any_of_status.append(
                {
                    "task": item["task"],
                    "any_of": options,
                    "satisfied": ok,
                    "satisfied_by": satisfied_by,
                }
            )
            if options and not satisfied_by:
                missing_any_of.append(item["task"])

        root_deps = plan.get("root_dependencies", {})
        recommended_next = [str(x) for x in root_deps.get("recommended_next", []) if str(x).strip()]
        recommended_prereq = [
            str(x) for x in root_deps.get("recommended_prerequisites", []) if str(x).strip()
        ]

        mermaid_lines = ["graph TD"]
        for node in plan["requires_all_order"]:
            spec = self._load_task_dependencies(node)
            for prereq in spec.get("prerequisites_all", []):
                mermaid_lines.append(f"  {prereq} --> {node}")
            for prereq in spec.get("prerequisites_any", []):
                mermaid_lines.append(f"  {prereq} -.-> {node}")
        mermaid = "\n".join(mermaid_lines)

        summary_lines = [
            "## Task Plan",
            f"- task_id: `{normalized_task}`",
            f"- paper_type: `{paper_type}`",
            f"- artifact_root: `{artifact_root}`",
            f"- project_root: `{project_root}`",
            "",
            "### Required prerequisites (prerequisites_all, transitive)",
        ]
        if required_chain:
            for index, node in enumerate(required_chain, start=1):
                status = "OK" if completion.get(node) else "MISSING"
                summary_lines.append(f"{index}. `{node}` [{status}]")
        else:
            summary_lines.append("- None")

        summary_lines.append("")
        summary_lines.append("### Any-of requirements (prerequisites_any)")
        if any_of_status:
            for item in any_of_status:
                options = ", ".join(f"`{opt}`" for opt in item["any_of"])
                satisfied_by = item.get("satisfied_by")
                status = "OK" if item["satisfied"] else "MISSING"
                suffix = f" (satisfied_by `{satisfied_by}`)" if satisfied_by else ""
                summary_lines.append(f"- `{item['task']}` requires any of: {options} [{status}]{suffix}")
        else:
            summary_lines.append("- None")

        summary_lines.append("")
        summary_lines.append("### Recommended prerequisites")
        summary_lines.append(
            "- " + ", ".join(f"`{item}`" for item in recommended_prereq)
            if recommended_prereq
            else "- None"
        )

        summary_lines.append("")
        summary_lines.append("### Suggested next tasks")
        summary_lines.append(
            "- " + ", ".join(f"`{item}`" for item in recommended_next)
            if recommended_next
            else "- None"
        )

        summary_lines.append("")
        summary_lines.append("### Dependency graph (Mermaid)")
        summary_lines.append("```mermaid")
        summary_lines.append(mermaid)
        summary_lines.append("```")

        confidence = 1.0 if not missing_required and not missing_any_of else 0.6
        recommendations = []
        if missing_required:
            recommendations.append("Run missing prerequisites_all tasks first: " + ", ".join(missing_required))
        if missing_any_of:
            recommendations.append(
                "Satisfy prerequisites_any for: "
                + ", ".join(f"{tid} (choose one option)" for tid in missing_any_of)
            )
        if recommended_next:
            recommendations.append("After completion, consider next tasks: " + ", ".join(recommended_next))

        return CollaborationResult(
            mode="task-plan",
            task_description=f"{normalized_task} {paper_type} {normalized_topic}"[:200],
            merged_analysis="\n".join(summary_lines),
            confidence=confidence,
            recommendations=recommendations,
            data={
                "task_id": normalized_task,
                "paper_type": paper_type,
                "topic": normalized_topic,
                "artifact_root": artifact_root,
                "project_root": str(project_root),
                "requires_all_order": plan["requires_all_order"],
                "missing_prerequisites_all": missing_required,
                "any_of_requirements": any_of_status,
                "recommended_prerequisites": recommended_prereq,
                "recommended_next": recommended_next,
                "mermaid": mermaid,
            },
        )

    def _load_task_agent_plan(self, task_id: str) -> dict[str, Any]:
        capability_map = self._read_standard("mcp-agent-capability-map.yaml")
        mcp_registry = set(
            self._parse_yaml_list(
                self._extract_top_level_section(capability_map, "mcp_registry"),
                item_indent=2,
            )
        )
        agent_registry = set(
            self._parse_yaml_list(
                self._extract_top_level_section(capability_map, "agent_registry"),
                item_indent=2,
            )
        )
        skill_registry = set(
            self._parse_yaml_list(
                self._extract_top_level_section(capability_map, "skill_registry"),
                item_indent=2,
            )
        )
        if not skill_registry:
            raise ValueError("skill_registry is missing or empty in mcp-agent-capability-map.yaml")

        skill_mapping_section = self._extract_top_level_section(
            capability_map,
            "task_skill_mapping",
        )
        skill_mapping_match = re.search(
            rf"^\s{{2}}{re.escape(task_id)}:\n((?:^\s{{4}}.*\n?)+)",
            skill_mapping_section,
            flags=re.MULTILINE,
        )
        if not skill_mapping_match:
            raise ValueError(f"Task ID not found in task_skill_mapping: {task_id}")
        required_skills = self._parse_yaml_list(
            self._extract_nested_section(skill_mapping_match.group(1), "required_skills", indent=4),
            item_indent=6,
        )
        if not required_skills:
            raise ValueError(f"Task {task_id} has empty required_skills")
        unknown_skills = sorted(set(required_skills) - skill_registry)
        if unknown_skills:
            raise ValueError(
                f"Task {task_id} has unknown skill entries: {', '.join(unknown_skills)}"
            )
        skill_catalog_section = self._extract_top_level_section(capability_map, "skill_catalog")
        required_skill_cards: list[dict[str, Any]] = []
        for skill_name in required_skills:
            card_match = re.search(
                rf"^\s{{2}}{re.escape(skill_name)}:\n((?:^\s{{4}}.*\n?)+)",
                skill_catalog_section,
                flags=re.MULTILINE,
            )
            if not card_match:
                raise ValueError(
                    f"Task {task_id} required skill missing catalog entry: {skill_name}"
                )
            card_block = card_match.group(1)
            skill_file = self._parse_yaml_scalar(card_block, "file", indent=4)
            category = self._parse_yaml_scalar(card_block, "category", indent=4)
            focus = self._parse_yaml_scalar(card_block, "focus", indent=4)
            default_outputs = self._parse_yaml_list(
                self._extract_nested_section(card_block, "default_outputs", indent=4),
                item_indent=6,
            )
            if not skill_file:
                raise ValueError(f"Skill catalog entry missing file for {skill_name}")
            if not category:
                raise ValueError(f"Skill catalog entry missing category for {skill_name}")
            if not focus:
                raise ValueError(f"Skill catalog entry missing focus for {skill_name}")
            required_skill_cards.append(
                {
                    "skill": skill_name,
                    "file": skill_file,
                    "category": category,
                    "focus": focus,
                    "default_outputs": default_outputs,
                }
            )

        task_section = self._extract_top_level_section(capability_map, "task_execution")
        task_match = re.search(
            rf"^\s{{2}}{re.escape(task_id)}:\n((?:^\s{{4}}.*\n?)+)",
            task_section,
            flags=re.MULTILINE,
        )
        if not task_match:
            raise ValueError(f"Task ID not found in mcp-agent-capability-map.yaml: {task_id}")
        block = task_match.group(1)

        required_mcp = self._parse_yaml_list(
            self._extract_nested_section(block, "required_mcp", indent=4),
            item_indent=6,
        )
        if not required_mcp:
            raise ValueError(f"Task {task_id} has empty required_mcp")
        unknown_mcp = sorted(set(required_mcp) - mcp_registry)
        if unknown_mcp:
            raise ValueError(f"Task {task_id} has unknown MCP entries: {', '.join(unknown_mcp)}")

        primary_agent = self._parse_yaml_scalar(block, "primary_agent", indent=4)
        review_agent = self._parse_yaml_scalar(block, "review_agent", indent=4)
        fallback_agent = self._parse_yaml_scalar(block, "fallback_agent", indent=4)
        for field_name, field_value in (
            ("primary_agent", primary_agent),
            ("review_agent", review_agent),
            ("fallback_agent", fallback_agent),
        ):
            if not field_value:
                raise ValueError(f"Task {task_id} missing {field_name}")
            if field_value not in agent_registry:
                raise ValueError(
                    f"Task {task_id} {field_name} must be in agent_registry: {field_value}"
                )

        quality_gates = self._parse_yaml_list(
            self._extract_nested_section(block, "quality_gates", indent=4),
            item_indent=6,
        )
        if not quality_gates:
            raise ValueError(f"Task {task_id} has empty quality_gates")

        return {
            "required_mcp": required_mcp,
            "required_skills": required_skills,
            "required_skill_cards": required_skill_cards,
            "primary_agent": primary_agent,
            "review_agent": review_agent,
            "fallback_agent": fallback_agent,
            "quality_gates": quality_gates,
        }

    def _check_command_available(self, command: str) -> tuple[bool, str]:
        try:
            parsed = shlex.split(command)
        except ValueError as exc:
            return False, f"Invalid command syntax: {exc}"
        if not parsed:
            return False, "Command is empty."
        executable = parsed[0]
        if "/" in executable:
            candidate = Path(executable).expanduser()
            if candidate.exists():
                return True, str(candidate)
            return False, f"Executable not found: {candidate}"
        resolved = shutil.which(executable)
        if resolved:
            return True, resolved
        return False, f"Command not found in PATH: {executable}"

    def doctor(self, cwd: Path) -> CollaborationResult:
        """Run local preflight checks for CLIs, API keys, and MCP command wiring."""
        checks: list[dict[str, str]] = []
        recommendations: list[str] = []
        target_cwd = cwd if cwd.is_absolute() else (Path.cwd() / cwd)

        def add_check(
            name: str,
            status: str,
            detail: str,
            recommendation: str | None = None,
        ) -> None:
            checks.append(
                {
                    "name": name,
                    "status": status,
                    "detail": detail,
                }
            )
            if recommendation and status in {"warning", "error"}:
                recommendations.append(recommendation)

        if target_cwd.exists():
            add_check("Working directory", "ok", str(target_cwd))
        else:
            add_check(
                "Working directory",
                "error",
                f"Missing path: {target_cwd}",
                "Provide a valid --cwd path before running parallel/task-run.",
            )

        for filename in ("research-workflow-contract.yaml", "mcp-agent-capability-map.yaml"):
            path = self._standards_file(filename)
            if path.exists():
                add_check(f"Standards file {filename}", "ok", str(path))
            else:
                add_check(
                    f"Standards file {filename}",
                    "error",
                    f"Missing file: {path}",
                    f"Restore {filename} under standards/ before running task-run.",
                )

        runtime_registry = {
            "codex": "OPENAI_API_KEY",
            "claude": "ANTHROPIC_API_KEY",
            "gemini": "GOOGLE_API_KEY",
        }
        for cli_name, api_env in runtime_registry.items():
            cli_path = shutil.which(cli_name)
            if cli_path:
                add_check(f"CLI {cli_name}", "ok", cli_path)
            else:
                add_check(
                    f"CLI {cli_name}",
                    "warning",
                    f"{cli_name} not found in PATH.",
                    f"Install {cli_name} CLI or route tasks away from {cli_name}.",
                )

            if os.environ.get(api_env, "").strip():
                add_check(f"Env {api_env}", "ok", "configured")
            else:
                add_check(
                    f"Env {api_env}",
                    "warning",
                    "not configured",
                    f"Export {api_env} for {cli_name} runtime authentication.",
                )

        try:
            capability_map = self._read_standard("mcp-agent-capability-map.yaml")
            mcp_registry = self._parse_yaml_list(
                self._extract_top_level_section(capability_map, "mcp_registry"),
                item_indent=2,
            )
        except (FileNotFoundError, ValueError) as exc:
            add_check(
                "MCP registry",
                "error",
                str(exc),
                "Fix standards/mcp-agent-capability-map.yaml before using task-run.",
            )
            mcp_registry = []

        for provider in mcp_registry:
            if provider == "filesystem":
                add_check("MCP filesystem", "ok", "local provider (no external command required)")
                continue
            env_name = self.mcp_connector._provider_env_var(provider)
            command = os.environ.get(env_name, "").strip()
            
            if not command:
                safe_provider_name = provider.replace("-", "_")
                native_script = Path(__file__).resolve().parents[1] / "scripts" / f"mcp_{safe_provider_name}.py"
                if native_script.exists():
                    add_check(
                        f"MCP {provider}", 
                        "ok", 
                        f"builtin provider ({native_script.name})"
                    )
                    continue
                    
                add_check(
                    f"MCP {provider}",
                    "warning",
                    f"{env_name} not configured",
                    f"Set {env_name} to enable {provider} evidence collection.",
                )
                continue
            command_ok, detail = self._check_command_available(command)
            if command_ok:
                add_check(f"MCP {provider}", "ok", f"{env_name} -> {detail}")
            else:
                add_check(
                    f"MCP {provider}",
                    "error",
                    f"{env_name} invalid: {detail}",
                    f"Fix {env_name} command so {provider} can execute.",
                )

        status_counts = {"ok": 0, "warning": 0, "error": 0}
        for item in checks:
            status_counts[item["status"]] = status_counts.get(item["status"], 0) + 1
        total_checks = len(checks)
        confidence = (
            status_counts["ok"] / total_checks
            if total_checks > 0
            else 0.0
        )

        merged_parts = [
            get_text("doctor_summary"),
            f"- Working directory: {target_cwd}",
            f"- Total checks: {total_checks}",
            f"- OK: {status_counts['ok']}",
            f"- Warnings: {status_counts['warning']}",
            f"- Errors: {status_counts['error']}",
            "",
            get_text("check_details"),
        ]
        for item in checks:
            merged_parts.append(
                f"- [{item['status'].upper()}] {item['name']}: {item['detail']}"
            )
        if recommendations:
            merged_parts.extend(["", get_text("recommendations")])
            for rec in sorted(set(recommendations)):
                merged_parts.append(f"- {rec}")

        return CollaborationResult(
            mode="doctor",
            task_description=f"doctor {target_cwd}"[:200],
            merged_analysis="\n".join(merged_parts),
            confidence=confidence,
            recommendations=sorted(set(recommendations)),
        )

    def _resolve_runtime_agent(
        self,
        preferred_agent: str,
        fallback_chain: list[str],
        exclude_agent: str | None = None,
    ) -> tuple[str, list[str]]:
        notes: list[str] = []
        seen: set[str] = set()
        candidates = [preferred_agent, *fallback_chain, "codex", "claude", "gemini"]
        for candidate in candidates:
            if not candidate or candidate in seen:
                continue
            seen.add(candidate)
            if candidate == exclude_agent:
                continue
            if candidate in self.RUNTIME_AGENTS:
                if candidate != preferred_agent:
                    notes.append(
                        f"Runtime routed agent '{preferred_agent}' to '{candidate}'."
                    )
                return candidate, notes
        raise ValueError(
            f"No runtime agent available for preferred={preferred_agent}, exclude={exclude_agent}"
        )

    def _execute_runtime_agent(
        self,
        agent_name: str,
        prompt: str,
        cwd: Path,
        runtime_options: dict[str, Any] | None = None,
        profile_directive: str | None = None,
    ) -> BridgeResponse:
        final_prompt = prompt
        if profile_directive:
            final_prompt = f"{prompt}\n\n{profile_directive}"
        options = runtime_options or {}
        if agent_name == "codex":
            return self.codex.execute(final_prompt, cwd, **options)
        if agent_name == "claude":
            return self.claude.execute(final_prompt, cwd, **options)
        if agent_name == "gemini":
            return self.gemini.execute(final_prompt, cwd, **options)
        return BridgeResponse.from_error(
            agent_name,
            f"Unsupported runtime agent for this orchestrator: {agent_name}",
        )

    def _normalize_topic(self, topic: str) -> str:
        normalized = re.sub(r"[^a-z0-9]+", "-", topic.strip().lower())
        return normalized.strip("-")

    def _build_task_packet(
        self,
        task_id: str,
        paper_type: str,
        topic: str,
        venue: str | None,
        artifact_root: str,
        required_outputs: list[str],
        required_mcp: list[str],
        required_skills: list[str],
        required_skill_cards: list[dict[str, Any]],
        quality_gates: list[str],
    ) -> dict[str, Any]:
        return {
            "task_id": task_id,
            "paper_type": paper_type,
            "topic": topic,
            "venue": venue or "",
            "artifact_root": artifact_root,
            "required_outputs": required_outputs,
            "required_mcp": required_mcp,
            "required_skills": required_skills,
            "required_skill_cards": required_skill_cards,
            "quality_gates": quality_gates,
        }

    def _collect_skill_context(
        self,
        task_packet: dict[str, Any],
        strict: bool = False,
    ) -> tuple[list[dict[str, Any]], list[str]]:
        skill_cards = task_packet.get("required_skill_cards", [])
        if not isinstance(skill_cards, list):
            return [], ["required_skill_cards packet field is not a list."]

        resolved_cards: list[dict[str, Any]] = []
        notes: list[str] = []
        missing_files: list[str] = []
        repo_root = self.standards_dir.parent

        for card in skill_cards:
            if not isinstance(card, dict):
                continue
            skill_name = str(card.get("skill", "")).strip()
            file_rel = str(card.get("file", "")).strip()
            file_path = repo_root / file_rel if file_rel else Path("")
            exists = bool(file_rel) and file_path.exists()
            status = "ok" if exists else "missing"
            if not exists:
                missing_files.append(skill_name or file_rel or "<unknown>")
                notes.append(
                    f"Skill '{skill_name}' file missing: {file_rel}"
                )
            resolved_cards.append(
                {
                    "skill": skill_name,
                    "category": str(card.get("category", "")).strip(),
                    "focus": str(card.get("focus", "")).strip(),
                    "file": file_rel,
                    "default_outputs": [
                        str(item) for item in card.get("default_outputs", []) if str(item).strip()
                    ],
                    "status": status,
                }
            )
        if strict and missing_files:
            raise ValueError(
                "Skills strict mode blocked execution due to missing skill files: "
                + ", ".join(missing_files)
            )
        return resolved_cards, notes

    def _format_skill_context(self, skill_cards: list[dict[str, Any]]) -> str:
        if not skill_cards:
            return "- No required skill cards available."
        lines: list[str] = []
        for card in skill_cards:
            skill_name = str(card.get("skill", "")).strip() or "unknown-skill"
            category = str(card.get("category", "")).strip() or "unspecified"
            focus = str(card.get("focus", "")).strip() or "No focus provided."
            status = str(card.get("status", "ok")).strip() or "ok"
            outputs = [
                str(item)
                for item in card.get("default_outputs", [])
                if str(item).strip()
            ]
            file_rel = str(card.get("file", "")).strip() or "-"
            lines.append(
                f"- {skill_name} [{status}] ({category}): {focus}"
            )
            lines.append(f"  spec: {file_rel}")
            if outputs:
                lines.append(f"  default_outputs: {', '.join(outputs)}")
        return "\n".join(lines)

    def _collect_mcp_evidence(
        self,
        task_packet: dict[str, Any],
        cwd: Path,
        strict: bool = False,
    ) -> tuple[list[MCPEvidence], list[str]]:
        evidence_items: list[MCPEvidence] = []
        notes: list[str] = []
        required_mcp = [str(item) for item in task_packet.get("required_mcp", [])]

        for provider in required_mcp:
            evidence = self.mcp_connector.collect(provider, task_packet, cwd)
            evidence_items.append(evidence)
            if evidence.status != "ok":
                notes.append(
                    f"MCP '{provider}' status={evidence.status}: {evidence.summary}"
                )

        if strict:
            hard_fail = [
                item.provider
                for item in evidence_items
                if item.status in {"error", "not_configured"}
            ]
            if hard_fail:
                raise ValueError(
                    "MCP strict mode blocked execution due to unavailable providers: "
                    + ", ".join(hard_fail)
                )
        return evidence_items, notes

    def _format_mcp_evidence(self, evidence_items: list[MCPEvidence]) -> str:
        if not evidence_items:
            return "- No MCP evidence collected."
        lines: list[str] = []
        for item in evidence_items:
            lines.append(f"- {item.provider} [{item.status}]: {item.summary}")
            if item.provenance:
                lines.append(
                    f"  provenance: {', '.join(item.provenance[:3])}"
                )
        return "\n".join(lines)

    def _build_task_draft_prompt(
        self,
        task_packet: dict[str, Any],
        mcp_evidence: list[MCPEvidence],
        skill_cards: list[dict[str, Any]],
        extra_context: str | None,
    ) -> str:
        return f"""You are executing one canonical research workflow task.

Task packet (JSON):
{json.dumps(task_packet, ensure_ascii=False, indent=2)}

Execution rules:
1. Follow sequence: plan -> mcp-evidence -> draft -> self-check.
2. Produce deliverables aligned to required_outputs and include those exact paths.
3. For each deliverable, cite the MCP evidence source used.
4. Apply every required skill in required_skills as a method constraint in your draft process.
   - Use required_skill_cards for concrete method focus and default outputs.
5. Explicitly check quality_gates and mark each as PASS/WARN/FAIL.
6. If required input is missing, list it under "Missing Inputs" and continue with placeholders.

MCP evidence snapshot:
{self._format_mcp_evidence(mcp_evidence)}

Required skill cards:
{self._format_skill_context(skill_cards)}

Additional context:
{extra_context or "No additional context."}

Return sections:
- Execution Plan
- Draft Outputs (by file path)
- Quality Gate Check
- Missing Inputs
- Next Actions
"""

    def _build_task_review_prompt(
        self,
        task_packet: dict[str, Any],
        mcp_evidence: list[MCPEvidence],
        skill_cards: list[dict[str, Any]],
        draft_output: str,
    ) -> str:
        return f"""Review the draft for this canonical research workflow task.

Task packet (JSON):
{json.dumps(task_packet, ensure_ascii=False, indent=2)}

Primary draft:
{draft_output}

MCP evidence snapshot:
{self._format_mcp_evidence(mcp_evidence)}

Required skill cards:
{self._format_skill_context(skill_cards)}

Review checklist:
1. Output path coverage against required_outputs.
2. Quality gate compliance against quality_gates.
3. Required_skills usage fidelity and completeness.
4. Internal consistency (claims/methods/evidence alignment).
5. Missing citations, assumptions, or unresolved risks.

Return sections:
- Verdict (PASS/BLOCK)
- Critical Issues
- Suggested Fixes
- Confidence (0-1)
"""

    def _build_task_triad_prompt(
        self,
        task_packet: dict[str, Any],
        mcp_evidence: list[MCPEvidence],
        skill_cards: list[dict[str, Any]],
        draft_output: str,
        review_output: str,
    ) -> str:
        return f"""Perform a third independent audit for this canonical research task.

Task packet (JSON):
{json.dumps(task_packet, ensure_ascii=False, indent=2)}

Primary draft:
{draft_output}

Independent review:
{review_output}

MCP evidence snapshot:
{self._format_mcp_evidence(mcp_evidence)}

Required skill cards:
{self._format_skill_context(skill_cards)}

Audit checklist:
1. Identify unresolved disagreements between draft and review.
2. Verify contract output paths and quality gates.
3. Check claim-method-evidence integrity.
4. Prioritize top 3 fixes by impact.

Return sections:
- Triad Verdict (PASS/BLOCK)
- Consensus Status (AGREE/PARTIAL/CONFLICT)
- Highest-Priority Fixes
- Confidence (0-1)
"""

    def task_run(
        self,
        task_id: str,
        paper_type: str,
        topic: str,
        cwd: Path,
        venue: str | None = None,
        context: str | None = None,
        mcp_strict: bool = False,
        skills_strict: bool = False,
        triad: bool = False,
        profile_file: Path | None = None,
        profile: str = "default",
        draft_profile: str | None = None,
        review_profile: str | None = None,
        triad_profile: str | None = None,
    ) -> CollaborationResult:
        """Run task-level orchestration using capability map and contract."""
        normalized_task = task_id.strip().upper()
        normalized_topic = self._normalize_topic(topic)
        routing_notes: list[str] = []

        try:
            agent_plan = self._load_task_agent_plan(normalized_task)
            artifact_root, required_outputs = self._load_task_outputs(normalized_task)
        except (FileNotFoundError, ValueError) as exc:
            msg = f"Task '{normalized_task}' is invalid or missing in workflow contract."
            raise ConfigError(ERR_CFG_INVALID_TASK, detail=msg) from exc

        try:
            profile_registry, task_profile_overrides = self._load_profile_bundle(profile_file)
            selected_profiles = self._resolve_task_profile_names(
                normalized_task,
                task_profile_overrides,
                base_profile=profile,
                draft_profile=draft_profile,
                review_profile=review_profile,
                triad_profile=triad_profile,
            )
            _ = self._resolve_profile_config(
                selected_profiles["base"],
                profile_registry,
            )
            draft_profile_cfg = self._resolve_profile_config(
                selected_profiles["draft"],
                profile_registry,
            )
            review_profile_cfg = self._resolve_profile_config(
                selected_profiles["review"],
                profile_registry,
            )
            triad_profile_cfg = self._resolve_profile_config(
                selected_profiles["triad"],
                profile_registry,
            )
        except ValueError as exc:
            raise ConfigError(ERR_CFG_INVALID_PROFILE, detail=str(exc)) from exc


        plan_result = self.task_plan(
            task_id=normalized_task,
            paper_type=paper_type,
            topic=normalized_topic,
            cwd=cwd,
        )
        packet = self._build_task_packet(
            task_id=normalized_task,
            paper_type=paper_type,
            topic=normalized_topic,
            venue=venue,
            artifact_root=artifact_root,
            required_outputs=required_outputs,
            required_mcp=agent_plan["required_mcp"],
            required_skills=agent_plan["required_skills"],
            required_skill_cards=agent_plan["required_skill_cards"],
            quality_gates=agent_plan["quality_gates"],
        )
        if plan_result.data:
            packet["task_plan"] = dict(plan_result.data)
        else:
            packet["task_plan"] = {
                "status": "unavailable",
                "detail": plan_result.merged_analysis,
            }
        try:
            skill_cards, skill_notes = self._collect_skill_context(
                packet,
                strict=skills_strict,
            )
            packet["required_skill_cards"] = skill_cards
            routing_notes.extend(skill_notes)
        except ValueError as exc:
            error_resp = BridgeResponse.from_error("skills", str(exc))
            return CollaborationResult(
                mode="task-run",
                task_description=f"{normalized_task} {paper_type} {normalized_topic}"[:200],
                merged_analysis=str(exc),
                confidence=0.0,
                recommendations=[],
                codex_response=error_resp if "codex" in str(exc).lower() else None,
                claude_response=error_resp if "claude" in str(exc).lower() else None,
                gemini_response=error_resp if "gemini" in str(exc).lower() else None,
            )

        try:
            mcp_evidence, mcp_notes = self._collect_mcp_evidence(
                packet,
                cwd,
                strict=mcp_strict,
            )
            routing_notes.extend(mcp_notes)
        except ValueError as exc:
            error_resp = BridgeResponse.from_error("mcp", str(exc))
            return CollaborationResult(
                mode="task-run",
                task_description=f"{normalized_task} {paper_type} {normalized_topic}"[:200],
                merged_analysis=str(exc),
                confidence=0.0,
                recommendations=[],
                codex_response=error_resp if "codex" in str(exc).lower() else None,
                claude_response=error_resp if "claude" in str(exc).lower() else None,
                gemini_response=error_resp if "gemini" in str(exc).lower() else None,
            )

        primary_runtime, primary_notes = self._resolve_runtime_agent(
            preferred_agent=agent_plan["primary_agent"],
            fallback_chain=[agent_plan["fallback_agent"]],
        )
        routing_notes.extend(primary_notes)

        draft_prompt = self._build_task_draft_prompt(
            packet,
            mcp_evidence,
            packet.get("required_skill_cards", []),
            context,
        )
        draft_resp = self._execute_runtime_agent(
            primary_runtime,
            draft_prompt,
            cwd,
            self._profile_runtime_options(draft_profile_cfg, primary_runtime),
            self._build_profile_directive(
                selected_profiles["draft"],
                draft_profile_cfg,
                stage="draft",
            ),
        )
        draft_runtime = primary_runtime

        fallback_resp: BridgeResponse | None = None
        fallback_runtime: str | None = None
        if not draft_resp.success:
            candidate_runtime, fallback_notes = self._resolve_runtime_agent(
                preferred_agent=agent_plan["fallback_agent"],
                fallback_chain=[agent_plan["review_agent"]],
            )
            routing_notes.extend(fallback_notes)
            if candidate_runtime != primary_runtime:
                fallback_runtime = candidate_runtime
                fallback_resp = self._execute_runtime_agent(
                    fallback_runtime,
                    draft_prompt,
                    cwd,
                    self._profile_runtime_options(draft_profile_cfg, fallback_runtime),
                    self._build_profile_directive(
                        selected_profiles["draft"],
                        draft_profile_cfg,
                        stage="draft",
                    ),
                )
                if fallback_resp.success:
                    draft_resp = fallback_resp
                    draft_runtime = fallback_runtime
            else:
                routing_notes.append("Fallback agent resolved to same runtime as primary; skipped rerun.")

        review_resp: BridgeResponse | None = None
        review_runtime: str | None = None
        if draft_resp.success:
            review_runtime, review_notes = self._resolve_runtime_agent(
                preferred_agent=agent_plan["review_agent"],
                fallback_chain=[agent_plan["fallback_agent"]],
                exclude_agent=draft_runtime,
            )
            routing_notes.extend(review_notes)
            review_prompt = self._build_task_review_prompt(
                packet,
                mcp_evidence,
                packet.get("required_skill_cards", []),
                draft_resp.content,
            )
            review_resp = self._execute_runtime_agent(
                review_runtime,
                review_prompt,
                cwd,
                self._profile_runtime_options(review_profile_cfg, review_runtime),
                self._build_profile_directive(
                    selected_profiles["review"],
                    review_profile_cfg,
                    stage="review",
                ),
            )
        else:
            routing_notes.append("Skipped independent review because draft generation failed.")

        triad_resp: BridgeResponse | None = None
        triad_runtime: str | None = None
        if triad and draft_resp.success and review_resp and review_resp.success:
            third_candidates = [
                agent for agent in ("codex", "claude", "gemini")
                if agent not in {draft_runtime, review_runtime}
            ]
            for candidate in third_candidates:
                if candidate in self.RUNTIME_AGENTS:
                    triad_runtime = candidate
                    break
            if triad_runtime:
                triad_prompt = self._build_task_triad_prompt(
                    packet,
                    mcp_evidence,
                    packet.get("required_skill_cards", []),
                    draft_resp.content,
                    review_resp.content,
                )
                triad_resp = self._execute_runtime_agent(
                    triad_runtime,
                    triad_prompt,
                    cwd,
                    self._profile_runtime_options(triad_profile_cfg, triad_runtime),
                    self._build_profile_directive(
                        selected_profiles["triad"],
                        triad_profile_cfg,
                        stage="triad",
                    ),
                )
            else:
                routing_notes.append(
                    "Triad mode requested, but no third runtime agent available."
                )
        elif triad:
            routing_notes.append(
                "Triad audit skipped because draft/review did not both succeed."
            )

        codex_resp = None
        claude_resp = None
        gemini_resp = None
        for runtime_agent, response in (
            (draft_runtime, draft_resp),
            (review_runtime, review_resp),
            (fallback_runtime, fallback_resp),
            (triad_runtime, triad_resp),
        ):
            if not response:
                continue
            if runtime_agent == "codex" and codex_resp is None:
                codex_resp = response
            if runtime_agent == "claude" and claude_resp is None:
                claude_resp = response
            if runtime_agent == "gemini" and gemini_resp is None:
                gemini_resp = response

        merged_parts = [
            plan_result.merged_analysis,
            "",
            get_text("task_packet"),
            json.dumps(packet, ensure_ascii=False, indent=2),
            "",
            get_text("mcp_evidence"),
            self._format_mcp_evidence(mcp_evidence),
            "",
            get_text("skill_cards"),
            self._format_skill_context(packet.get("required_skill_cards", [])),
            "",
            get_text("agent_profiles"),
            "\n".join(
                [
                    f"- base: {selected_profiles['base']}",
                    f"- draft: {selected_profiles['draft']}",
                    f"- review: {selected_profiles['review']}",
                    f"- triad: {selected_profiles['triad']}",
                ]
            ),
            "",
            get_text("routing"),
            "\n".join(f"- {item}" for item in routing_notes) if routing_notes else "- Direct mapping used.",
            "",
            get_text("draft", agent=draft_runtime),
            draft_resp.content if draft_resp.success else f"[FAILED] {draft_resp.error}",
        ]
        if review_resp:
            merged_parts.extend(
                [
                    "",
                    get_text("review", agent=review_runtime),
                    review_resp.content if review_resp.success else f"[FAILED] {review_resp.error}",
                ]
            )
        if fallback_resp and fallback_runtime and fallback_runtime != draft_runtime:
            merged_parts.extend(
                [
                    "",
                    f"## Fallback Attempt ({fallback_runtime})",
                    fallback_resp.content if fallback_resp.success else f"[FAILED] {fallback_resp.error}",
                ]
            )
        if triad_resp and triad_runtime:
            merged_parts.extend(
                [
                    "",
                    get_text("triad_audit", agent=triad_runtime),
                    triad_resp.content if triad_resp.success else f"[FAILED] {triad_resp.error}",
                ]
            )
        merged = "\n".join(merged_parts)

        if (
            draft_resp.success
            and review_resp and review_resp.success
            and triad_resp and triad_resp.success
        ):
            confidence = 0.95
        elif draft_resp.success and review_resp and review_resp.success and review_runtime != draft_runtime:
            confidence = 0.9
        elif draft_resp.success and review_resp and review_resp.success:
            confidence = 0.8
        elif draft_resp.success:
            confidence = 0.65
        else:
            confidence = 0.0

        return CollaborationResult(
            mode="task-run",
            task_description=f"{normalized_task} {paper_type} {normalized_topic}"[:200],
            codex_response=codex_resp,
            claude_response=claude_resp,
            gemini_response=gemini_resp,
            merged_analysis=merged,
            confidence=confidence,
            recommendations=self._extract_recommendations(merged),
        )



    def code_build(
        self,
        method: str,
        cwd: Path,
        domain: str = "auto",
        tier: str = "standard",
        paper_path: str | None = None,
    ) -> CollaborationResult:
        """
        Build academic research code.
        
        Args:
            method: Methodology name (e.g. "DID", "GARCH")
            cwd: Working directory
            domain: finance, econ, metrics, cs, or auto
            tier: standard (template) or advanced (decomposition)
            paper_path: Optional path to paper for context
        """
        # 1. Prompt engineering based on tier
        if tier == "advanced":
            prompt = f"""
You are an expert Research Software Engineer.
Task: Implement the methodology '{method}' from first principles (Tier 2 Advanced Mode).
Domain: {domain}

REQUIREMENTS:
1. Do NOT use high-level library wrappers.
2. If this is an optimization problem, define the Objective Function (Likelihood/Loss) explicitly.
3. Use JAX or PyTorch if gradients are needed.
4. If this is a structural model, ensure all equations are translated to code.

CONTEXT:
{f'Reference Paper: {paper_path}' if paper_path else 'No specific paper provided.'}

OUTPUT:
- Single Python file
- Mathematical comments next to code lines
- Verification script
"""
            return self.execute(
                mode=CollaborationMode.CHAIN,
                cwd=cwd,
                prompt=prompt,
                generator="codex"
            )
        else:
            # Standard Tier
            prompt = f"""
You are an expert Research Software Engineer.
Task: Implement the methodology '{method}' using standard libraries (Tier 1 Standard Mode).
Domain: {domain}

REQUIREMENTS:
1. Use standard, robust libraries (e.g. statsmodels, arch, linearmodels, pyfixest).
2. Follow best practices for the domain.
3. Include data loading and validation steps.

CONTEXT:
{f'Reference Paper: {paper_path}' if paper_path else 'No specific paper provided.'}
"""
            # Use role-based for standard: Codex builds, Gemini explains usage
            return self.execute(
                mode=CollaborationMode.ROLE_BASED,
                cwd=cwd,
                codex_task=f"{prompt}\n\nGenerate the implementation code.",
                gemini_task=f"{prompt}\n\nExplain the library usage and parameter choices."
            )

def main():
    """CLI entry point."""
    configure_stdio()
    
    parser = argparse.ArgumentParser(
        description="Multi-Model Orchestrator for Academic Research"
    )
    
    subparsers = parser.add_subparsers(dest="mode", required=True)
    
    # ... (previous parsers: parallel, chain, role, single) ...
    
    # Parallel mode
    parallel = subparsers.add_parser("parallel", help="Triad concurrent analysis with synthesis")
    parallel.add_argument("--prompt", required=True, help="Analysis prompt")
    parallel.add_argument("--cwd", required=True, type=Path, help="Working directory")
    parallel.add_argument(
        "--summarizer",
        choices=["codex", "claude", "gemini"],
        default="claude",
        help="Model used for post-parallel synthesis",
    )
    parallel.add_argument(
        "--profile-file",
        type=Path,
        help="Optional JSON file defining user agent profiles and per-task overrides",
    )
    parallel.add_argument(
        "--profile",
        default="default",
        help="Base profile name for analyzer agents (default: default)",
    )
    parallel.add_argument(
        "--summarizer-profile",
        help="Profile name for summarizer agent (defaults to --profile)",
    )
    
    # Chain mode
    chain = subparsers.add_parser("chain", help="One generates, other verifies")
    chain.add_argument("--prompt", required=True, help="Generation prompt")
    chain.add_argument("--cwd", required=True, type=Path, help="Working directory")
    chain.add_argument("--generator", choices=["codex", "claude", "gemini"], default="codex")
    
    # Role-based mode
    role = subparsers.add_parser("role", help="Task division by specialty")
    role.add_argument("--cwd", required=True, type=Path, help="Working directory")
    role.add_argument("--codex-task", help="Task for Codex")
    role.add_argument("--claude-task", help="Task for Claude")
    role.add_argument("--gemini-task", help="Task for Gemini")
    
    # Single mode
    single = subparsers.add_parser("single", help="Single model execution")
    single.add_argument("--prompt", required=True, help="Task prompt")
    single.add_argument("--cwd", required=True, type=Path, help="Working directory")
    single.add_argument("--model", choices=["codex", "claude", "gemini"], default="codex")
    single.add_argument("--session-id", help="Resume existing session")
    
    # NEW: Code Build mode
    code_build = subparsers.add_parser("code-build", help="Build academic research code")
    code_build.add_argument("--method", required=True, help="Methodology name")
    code_build.add_argument("--cwd", required=True, type=Path, help="Working directory")
    code_build.add_argument("--domain", default="auto", help="finance, econ, metrics, cs")
    code_build.add_argument("--tier", choices=["standard", "advanced"], default="standard")
    code_build.add_argument("--paper", help="Path to paper PDF/URL")

    task_run = subparsers.add_parser(
        "task-run",
        help="Run standardized Task-ID workflow with capability-map agent routing",
    )
    task_run.add_argument("--task-id", required=True, help="Canonical Task ID (e.g. F3)")
    task_run.add_argument(
        "--paper-type",
        required=True,
        choices=["empirical", "systematic-review", "methods", "theory"],
        help="Paper type from workflow contract",
    )
    task_run.add_argument("--topic", required=True, help="Research topic slug/name")
    task_run.add_argument("--cwd", required=True, type=Path, help="Working directory")
    task_run.add_argument("--venue", help="Target venue (optional)")
    task_run.add_argument("--context", help="Additional execution context (optional)")
    task_run.add_argument(
        "--mcp-strict",
        action="store_true",
        help="Fail task-run if required MCP providers are unavailable",
    )
    task_run.add_argument(
        "--skills-strict",
        action="store_true",
        help="Fail task-run if required skill specs are missing",
    )
    task_run.add_argument(
        "--triad",
        action="store_true",
        help="Enable third independent agent audit when draft/review succeed",
    )
    task_run.add_argument(
        "--profile-file",
        type=Path,
        help="Optional JSON file defining user agent profiles and per-task overrides",
    )
    task_run.add_argument(
        "--profile",
        default="default",
        help="Base profile name for task-run stages (default: default)",
    )
    task_run.add_argument(
        "--draft-profile",
        help="Profile name for draft stage (defaults to --profile)",
    )
    task_run.add_argument(
        "--review-profile",
        help="Profile name for review stage (defaults to --profile)",
    )
    task_run.add_argument(
        "--triad-profile",
        help="Profile name for triad stage (defaults to --profile)",
    )
    task_plan = subparsers.add_parser(
        "task-plan",
        help="Render dependency-based task plan from workflow contract",
    )
    task_plan.add_argument("--task-id", required=True, help="Canonical Task ID (e.g. F3)")
    task_plan.add_argument(
        "--paper-type",
        required=True,
        choices=["empirical", "systematic-review", "methods", "theory"],
        help="Paper type from workflow contract",
    )
    task_plan.add_argument("--topic", required=True, help="Research topic slug/name")
    task_plan.add_argument("--cwd", required=True, type=Path, help="Working directory")
    doctor = subparsers.add_parser(
        "doctor",
        help="Run local preflight checks for CLIs, API keys, and MCP command wiring",
    )
    doctor.add_argument(
        "--cwd",
        default=Path.cwd(),
        type=Path,
        help="Working directory used by orchestrator preflight checks",
    )

    args = parser.parse_args()
    
    orchestrator = ModelOrchestrator()
    
    if args.mode == "code-build":
        result = orchestrator.code_build(
            method=args.method,
            cwd=args.cwd,
            domain=args.domain,
            tier=args.tier,
            paper_path=args.paper
        )
    elif args.mode == "doctor":
        result = orchestrator.doctor(cwd=args.cwd)
    elif args.mode == "task-run":
        result = orchestrator.task_run(
            task_id=args.task_id,
            paper_type=args.paper_type,
            topic=args.topic,
            cwd=args.cwd,
            venue=args.venue,
            context=args.context,
            mcp_strict=args.mcp_strict,
            skills_strict=args.skills_strict,
            triad=args.triad,
            profile_file=getattr(args, "profile_file", None),
            profile=getattr(args, "profile", "default"),
            draft_profile=getattr(args, "draft_profile", None),
            review_profile=getattr(args, "review_profile", None),
            triad_profile=getattr(args, "triad_profile", None),
        )
    elif args.mode == "task-plan":
        result = orchestrator.task_plan(
            task_id=args.task_id,
            paper_type=args.paper_type,
            topic=args.topic,
            cwd=args.cwd,
        )
    else:
        # Map previous modes
        mode_enum = CollaborationMode.PARALLEL # Default placeholder logic needed or direct mapping
        if args.mode == "parallel": mode_enum = CollaborationMode.PARALLEL
        elif args.mode == "chain": mode_enum = CollaborationMode.CHAIN
        elif args.mode == "role": mode_enum = CollaborationMode.ROLE_BASED
        elif args.mode == "single": mode_enum = CollaborationMode.SINGLE
        
        result = orchestrator.execute(
            mode=mode_enum,
            cwd=args.cwd,
            prompt=getattr(args, "prompt", None),
            codex_task=getattr(args, "codex_task", None),
            claude_task=getattr(args, "claude_task", None),
            gemini_task=getattr(args, "gemini_task", None),
            generator=getattr(args, "generator", "codex"),
            single_model=getattr(args, "model", "codex"),
            parallel_summarizer=getattr(args, "summarizer", "claude"),
            profile_file=getattr(args, "profile_file", None),
            profile=getattr(args, "profile", "default"),
            summarizer_profile=getattr(args, "summarizer_profile", None),
            session_id=getattr(args, "session_id", None),
        )
    
    print(result.to_json())

if __name__ == "__main__":
    try:
        main()
    except ResearchError as e:
        print(f"\n❌ {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ [ERR-RS-UNKNOWN] Unexpected system error: {str(e)}", file=sys.stderr)
        sys.exit(1)
