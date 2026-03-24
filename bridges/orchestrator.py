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

Python 3.12+ required.
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
from .i18n import get_language, get_text
from .critique_questions import get_critique_questions
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
        "focused-delivery": {
            "persona": "Execution assistant optimizing for minimal artifact sprawl and tight scope control.",
            "draft_style": "Only deliver the active outputs for this run, consolidate support into those files, and defer secondary artifacts explicitly.",
            "review_style": "Reject unnecessary helper outputs, scope creep, and weakly justified breadth.",
            "summary_style": "Summarize only the essential outputs, blockers, and deferred items.",
            "runtime_options": {
                "codex": {"sandbox": "read-only", "non_interactive": True, "timeout_seconds": 240},
                "claude": {"permission_mode": "default", "non_interactive": True, "timeout_seconds": 240},
                "gemini": {"sandbox": False, "non_interactive": True, "timeout_seconds": 240},
            },
        },
        "deep-research": {
            "persona": "Research analyst optimizing for exhaustive evidence expansion, contradiction hunting, and narrow high-confidence conclusions.",
            "analysis_style": "Expand the evidence base in rounds, search for counterevidence, and separate evidence from inference aggressively.",
            "draft_style": "Prefer fewer but strongly supported conclusions, with explicit scope boundaries, counterevidence checks, and decision rationale.",
            "review_style": "Audit evidence depth, contradiction coverage, and boundary-case reasoning before surface polish.",
            "summary_style": "Collapse broad findings into the smallest strongly supported claim set and list remaining evidence gaps.",
            "triad_style": "Adjudicate unresolved evidence conflicts and force a decision-grade rationale.",
            "runtime_options": {
                "codex": {"sandbox": "read-only", "non_interactive": True, "timeout_seconds": 480},
                "claude": {"permission_mode": "default", "non_interactive": True, "timeout_seconds": 540},
                "gemini": {"sandbox": True, "non_interactive": True, "timeout_seconds": 420},
            },
        },
    }
    RUNTIME_AGENTS = {"codex", "claude", "gemini"}
    DOMAIN_PROFILE_ALIASES = {
        "business": "business-management",
        "business-management": "business-management",
        "finance": "finance",
        "econ": "economics",
        "economics": "economics",
        "econometrics": "economics",
        "management": "business-management",
        "management-studies": "business-management",
        "organizational-behavior": "business-management",
        "organization-studies": "business-management",
        "metrics": "economics",
        "psych": "psychology",
        "psychology": "psychology",
        "biomed": "biomedical",
        "biomedical": "biomedical",
        "education": "education",
        "edu": "education",
        "cs": "cs-ai",
        "ai": "cs-ai",
        "ml": "cs-ai",
        "llm": "cs-ai",
        "cs-ai": "cs-ai",
        "political-science": "political-science",
        "political-science-sociology": "political-science",
        "politicalscience": "political-science",
        "polisci": "political-science",
        "epi": "epidemiology",
        "epidemiology": "epidemiology",
        "ecology": "ecology-environmental",
        "environmental": "ecology-environmental",
        "ecology-environmental": "ecology-environmental",
    }
    CODE_BUILD_FOCUS_TO_TASK = {
        "implementation": "I1",
        "method_implementation": "I1",
        "method": "I1",
        "reproduction": "I2",
        "replication": "I2",
        "data_pipeline": "I3",
        "pipeline": "I3",
        "reproducibility_audit": "I4",
        "audit": "I4",
        "code_specification": "I5",
        "spec": "I5",
        "specification": "I5",
        "code_planning": "I6",
        "planning": "I6",
        "plan": "I6",
        "execution_performance": "I7",
        "execution": "I7",
        "execute": "I7",
        "performance": "I7",
        "code_review": "I8",
        "review": "I8",
        "full": "FULL",
    }
    STAGE_I_TEMPLATE_TYPE_BY_TASK = {
        "I4": "reproducibility_audit",
        "I5": "code_specification",
        "I6": "code_plan",
        "I7": "performance_profile",
        "I8": "code_review",
    }
    STAGE_I_CONTRACT_HEADING_BY_TASK = {
        "I4": "Audit Contract Block",
        "I5": "Spec Contract Block",
        "I6": "Plan Contract Block",
        "I7": "Execution Contract Block",
        "I8": "Review Contract Block",
    }
    STAGE_I_CONTRACT_KEYS_BY_TASK = {
        "I4": [
            "task_id",
            "topic",
            "audit_artifact",
            "reviewed_artifacts",
            "environment_files",
            "seed_policy_status",
            "rerun_entrypoints",
            "verdict",
            "blocking_gaps",
        ],
        "I5": [
            "task_id",
            "topic",
            "method_or_pipeline",
            "primary_artifact",
            "inputs",
            "outputs",
            "dependencies",
            "seeds_policy",
            "acceptance_tests",
            "blocked_decisions",
        ],
        "I6": [
            "task_id",
            "topic",
            "spec_source",
            "plan_artifact",
            "steps",
        ],
        "I7": [
            "task_id",
            "topic",
            "plan_source",
            "performance_artifact",
            "analysis_outputs",
            "documentation_outputs",
            "container_outputs",
            "validation_runs",
            "profiling_targets",
        ],
        "I8": [
            "task_id",
            "topic",
            "review_target",
            "spec_source",
            "plan_source",
            "review_artifact",
            "verdict",
            "blocking_findings",
            "review_coverage",
        ],
    }
    STAGE_I_FRONTMATTER_KEYS = [
        "task_id",
        "template_type",
        "topic",
        "primary_artifact",
    ]
    STAGE_I_PRIMARY_ARTIFACT_BY_TASK = {
        "I4": "code/reproducibility_audit.md",
        "I5": "code/code_specification.md",
        "I6": "code/plan.md",
        "I7": "code/performance_profile.md",
        "I8": "code/code_review.md",
    }
    DEFAULT_STANDARDS_DIR = Path(__file__).resolve().parents[1] / "standards"
    
    def __init__(
        self,
        codex_sandbox: str = "read-only",
        claude_permission_mode: str | None = None,
        gemini_sandbox: bool = False,
        standards_dir: Path | None = None,
        mcp_timeout_seconds: int = 20,
        interactive: bool = False,
    ):
        """Initialize orchestrator with bridges."""
        self.codex = CodexBridge(sandbox=codex_sandbox)
        self.claude = ClaudeBridge(permission_mode=claude_permission_mode)
        self.gemini = GeminiBridge(sandbox=gemini_sandbox)
        self.standards_dir = standards_dir or self.DEFAULT_STANDARDS_DIR
        self.mcp_connector = MCPConnector(timeout_seconds=mcp_timeout_seconds)
        self.interactive = interactive
        self._skill_registry_metadata_cache: dict[str, dict[str, str]] | None = None

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

        if execution_level == "triad":
            parallel_header = get_text("parallel_execution")
        else:
            parallel_header = get_text("parallel_execution_dual")
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
        joined_outputs = "\n\n".join(output_blocks)
        return f"""Synthesize multi-agent parallel analysis into one actionable conclusion.

Original task:
{original_prompt}

Successful analyzers: {", ".join(success_agents)}
Failed analyzers: {failed_text}

Analyzer outputs:
{joined_outputs}

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
        runtime_options = {"session_id": session_id} if session_id else {}
        resp = self._execute_runtime_agent(model, prompt, cwd, runtime_options)
        if model == "codex":
            return CollaborationResult(
                mode="single",
                task_description=prompt[:200],
                codex_response=resp,
                merged_analysis=resp.content if resp.success else resp.error or "",
                confidence=1.0 if resp.success else 0.0,
            )
        if model == "claude":
            return CollaborationResult(
                mode="single",
                task_description=prompt[:200],
                claude_response=resp,
                merged_analysis=resp.content if resp.success else resp.error or "",
                confidence=1.0 if resp.success else 0.0,
            )
        if model == "gemini":
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

    def _load_skill_registry_metadata(self) -> dict[str, dict[str, str]]:
        if self._skill_registry_metadata_cache is not None:
            return self._skill_registry_metadata_cache

        registry_path = self.standards_dir.parent / "skills" / "registry.yaml"
        if not registry_path.exists():
            self._skill_registry_metadata_cache = {}
            return self._skill_registry_metadata_cache

        try:
            content = registry_path.read_text(encoding="utf-8")
        except OSError:
            self._skill_registry_metadata_cache = {}
            return self._skill_registry_metadata_cache

        section = self._extract_top_level_section(content, "skills")
        entry_pattern = re.compile(
            r"^\s{2}-\s*id:\s*([A-Za-z0-9_-]+)\s*$",
            flags=re.MULTILINE,
        )
        matches = list(entry_pattern.finditer(section))
        metadata: dict[str, dict[str, str]] = {}
        for index, match in enumerate(matches):
            start = match.start()
            end = matches[index + 1].start() if index + 1 < len(matches) else len(section)
            block = section[start:end]
            skill_id = match.group(1)
            metadata[skill_id] = {
                "summary": self._parse_yaml_scalar(block, "summary", indent=4),
                "summary_zh": self._parse_yaml_scalar(block, "summary_zh", indent=4),
                "display_name_zh": self._parse_yaml_scalar(block, "display_name_zh", indent=4),
                "when_to_use_zh": self._parse_yaml_scalar(block, "when_to_use_zh", indent=4),
            }

        self._skill_registry_metadata_cache = metadata
        return self._skill_registry_metadata_cache

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

    def _load_task_functional_plan(
        self,
        task_id: str,
        capability_map: str | None = None,
    ) -> dict[str, Any]:
        capability_map = capability_map or self._read_standard("mcp-agent-capability-map.yaml")
        functional_agent_registry = set(
            self._parse_yaml_list(
                self._extract_top_level_section(capability_map, "functional_agent_registry"),
                item_indent=2,
            )
        )
        if not functional_agent_registry:
            raise ValueError(
                "functional_agent_registry is missing or empty in mcp-agent-capability-map.yaml"
            )

        routing_section = self._extract_top_level_section(
            capability_map,
            "task_functional_routing",
        )
        defaults_section = self._extract_nested_section(
            routing_section,
            "defaults_by_stage",
            indent=2,
        )
        routing_defaults = {
            stage: owner
            for stage, owner in re.findall(
                r'^\s{4}([A-Z]):\s*"?(.*?)"?\s*$',
                defaults_section,
                flags=re.MULTILINE,
            )
        }
        overrides_section = self._extract_nested_section(
            routing_section,
            "overrides",
            indent=2,
        )
        routing_overrides = {
            routed_task: owner
            for routed_task, owner in re.findall(
                r'^\s{4}([A-I][0-9_]+):\s*"?(.*?)"?\s*$',
                overrides_section,
                flags=re.MULTILINE,
            )
        }

        stage_id = task_id[:1]
        stage_default_owner = routing_defaults.get(stage_id, "")
        functional_owner = routing_overrides.get(task_id) or stage_default_owner
        owner_source = "override" if task_id in routing_overrides else "stage-default"
        if not functional_owner:
            raise ValueError(f"Task {task_id} has no functional owner in task_functional_routing")
        if functional_owner not in functional_agent_registry:
            raise ValueError(
                f"Task {task_id} resolves to unknown functional owner: {functional_owner}"
            )

        functional_agents_section = self._extract_top_level_section(
            capability_map,
            "functional_agents",
        )
        agent_match = re.search(
            rf"^\s{{2}}{re.escape(functional_owner)}:\n((?:^\s{{4}}.*\n?)+)",
            functional_agents_section,
            flags=re.MULTILINE,
        )
        if not agent_match:
            raise ValueError(
                f"Functional owner {functional_owner} missing from functional_agents catalog"
            )
        agent_block = agent_match.group(1)
        role_file = self._parse_yaml_scalar(agent_block, "file", indent=4)
        mapped_role = self._parse_yaml_scalar(agent_block, "mapped_role", indent=4)
        focus = self._parse_yaml_scalar(agent_block, "focus", indent=4)

        role_id = mapped_role or functional_owner
        display_name = functional_owner
        tone = ""
        preferred_skills: list[str] = []
        if role_file:
            role_path = self.standards_dir.parent / role_file
            if role_path.exists():
                role_content = role_path.read_text(encoding="utf-8")
                role_id = self._parse_yaml_scalar(role_content, "id", indent=0) or role_id
                display_name = (
                    self._parse_yaml_scalar(role_content, "display_name", indent=0)
                    or display_name
                )
                tone = self._parse_yaml_scalar(role_content, "tone", indent=0)
                preferred_skills = self._parse_yaml_list(
                    self._extract_top_level_section(role_content, "preferred_skills"),
                    item_indent=2,
                )

        return {
            "functional_owner": functional_owner,
            "functional_owner_source": owner_source,
            "functional_stage_default_owner": stage_default_owner,
            "functional_stage_id": stage_id,
            "functional_role_id": role_id,
            "functional_display_name": display_name,
            "functional_role_file": role_file,
            "functional_focus": focus,
            "functional_tone": tone,
            "functional_preferred_skills": preferred_skills,
        }

    def _build_functional_handoff_trace(
        self,
        task_ids: list[str],
        capability_map: str | None = None,
    ) -> dict[str, Any]:
        capability_map = capability_map or self._read_standard("mcp-agent-capability-map.yaml")
        trace: list[dict[str, Any]] = []
        owner_chain: list[str] = []

        for routed_task in task_ids:
            functional_plan = self._load_task_functional_plan(routed_task, capability_map)
            trace.append(
                {
                    "task_id": routed_task,
                    "functional_owner": functional_plan["functional_owner"],
                    "functional_owner_source": functional_plan["functional_owner_source"],
                    "functional_role_id": functional_plan["functional_role_id"],
                    "functional_display_name": functional_plan["functional_display_name"],
                }
            )
            if not owner_chain or owner_chain[-1] != functional_plan["functional_owner"]:
                owner_chain.append(functional_plan["functional_owner"])

        return {
            "trace": trace,
            "owner_chain": owner_chain,
            "owner_chain_text": " -> ".join(owner_chain),
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
            agent_plan = self._load_task_agent_plan(normalized_task)
            functional_handoff = self._build_functional_handoff_trace(
                plan["requires_all_order"],
            )
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
        runtime_plan = dict(agent_plan.get("runtime_plan", {}))
        handoff_trace = functional_handoff.get("trace", [])
        handoff_chain = functional_handoff.get("owner_chain", [])

        mermaid_lines = ["graph TD"]
        for node in plan["requires_all_order"]:
            spec = self._load_task_dependencies(node)
            for prereq in spec.get("prerequisites_all", []):
                mermaid_lines.append(f"  {prereq} --> {node}")
            for prereq in spec.get("prerequisites_any", []):
                mermaid_lines.append(f"  {prereq} -.-> {node}")
        mermaid = "\n".join(mermaid_lines)
        zh_ui = get_language() == "zh-CN"
        none_label = "None / 无" if zh_ui else "None"
        ok_label = "OK / 已满足" if zh_ui else "OK"
        missing_label = "MISSING / 缺失" if zh_ui else "MISSING"

        summary_lines = [
            "## Task Plan / 任务规划" if zh_ui else "## Task Plan",
            f"- {'task_id / 任务ID' if zh_ui else 'task_id'}: `{normalized_task}`",
            f"- {'paper_type / 论文类型' if zh_ui else 'paper_type'}: `{paper_type}`",
            f"- {'artifact_root / 产物根目录' if zh_ui else 'artifact_root'}: `{artifact_root}`",
            f"- {'project_root / 项目根目录' if zh_ui else 'project_root'}: `{project_root}`",
            "",
            "### Functional routing / 功能路由" if zh_ui else "### Functional routing",
            (
                f"- {'target_owner / 功能负责人' if zh_ui else 'target_owner'}: `{agent_plan['functional_owner']}` "
                f"[{agent_plan['functional_owner_source']}]"
            ),
            f"- {'role / 角色' if zh_ui else 'role'}: `{agent_plan['functional_role_id']}`",
        ]
        if agent_plan.get("functional_display_name"):
            summary_lines.append(
                f"- {'display_name / 显示名称' if zh_ui else 'display_name'}: {agent_plan['functional_display_name']}"
            )
        if agent_plan.get("functional_focus"):
            summary_lines.append(
                f"- {'focus / 核心关注' if zh_ui else 'focus'}: {agent_plan['functional_focus']}"
            )
        if agent_plan.get("functional_tone"):
            summary_lines.append(
                f"- {'tone / 语气要求' if zh_ui else 'tone'}: {agent_plan['functional_tone']}"
            )
        if handoff_chain:
            summary_lines.append(
                f"- {'handoff_chain / 交接链' if zh_ui else 'handoff_chain'}: "
                + " -> ".join(f"`{owner}`" for owner in handoff_chain)
            )
        else:
            summary_lines.append(f"- {'handoff_chain / 交接链' if zh_ui else 'handoff_chain'}: {none_label}")

        summary_lines.extend(
            [
                "",
                "### Functional handoff trace / 功能交接轨迹" if zh_ui else "### Functional handoff trace",
            ]
        )
        if handoff_trace:
            for item in handoff_trace:
                summary_lines.append(
                    f"- `{item['task_id']}` -> `{item['functional_owner']}` "
                    f"[{item['functional_owner_source']}]"
                )
        else:
            summary_lines.append(f"- {none_label}")

        summary_lines.extend(
            [
                "",
                "### Runtime routing plan / 运行时路由预案" if zh_ui else "### Runtime routing plan",
                f"- {'draft / 起草' if zh_ui else 'draft'}: `{runtime_plan.get('primary_agent', '-') or '-'}`",
                f"- {'review / 复核' if zh_ui else 'review'}: `{runtime_plan.get('review_agent', '-') or '-'}`",
                f"- {'fallback / 回退' if zh_ui else 'fallback'}: `{runtime_plan.get('fallback_agent', '-') or '-'}`",
                "",
                (
                    "### Required prerequisites (prerequisites_all, transitive) / 必需前置任务"
                    if zh_ui else
                    "### Required prerequisites (prerequisites_all, transitive)"
                ),
            ]
        )
        if required_chain:
            for index, node in enumerate(required_chain, start=1):
                status = ok_label if completion.get(node) else missing_label
                summary_lines.append(f"{index}. `{node}` [{status}]")
        else:
            summary_lines.append(f"- {none_label}")

        summary_lines.append("")
        summary_lines.append(
            "### Any-of requirements (prerequisites_any) / 满足其一的前置条件"
            if zh_ui else
            "### Any-of requirements (prerequisites_any)"
        )
        if any_of_status:
            for item in any_of_status:
                options = ", ".join(f"`{opt}`" for opt in item["any_of"])
                satisfied_by = item.get("satisfied_by")
                status = ok_label if item["satisfied"] else missing_label
                suffix = (
                    f" ({'satisfied_by / 由此满足' if zh_ui else 'satisfied_by'} `{satisfied_by}`)"
                    if satisfied_by else ""
                )
                summary_lines.append(
                    f"- `{item['task']}` "
                    f"{'requires any of / 满足其一即可' if zh_ui else 'requires any of'}: "
                    f"{options} [{status}]{suffix}"
                )
        else:
            summary_lines.append(f"- {none_label}")

        summary_lines.append("")
        summary_lines.append(
            "### Recommended prerequisites / 建议先完成的任务"
            if zh_ui else
            "### Recommended prerequisites"
        )
        summary_lines.append(
            "- " + ", ".join(f"`{item}`" for item in recommended_prereq)
            if recommended_prereq
            else f"- {none_label}"
        )

        summary_lines.append("")
        summary_lines.append(
            "### Suggested next tasks / 推荐下一步任务"
            if zh_ui else
            "### Suggested next tasks"
        )
        summary_lines.append(
            "- " + ", ".join(f"`{item}`" for item in recommended_next)
            if recommended_next
            else f"- {none_label}"
        )

        summary_lines.append("")
        summary_lines.append(
            "### Dependency graph (Mermaid) / 依赖图"
            if zh_ui else
            "### Dependency graph (Mermaid)"
        )
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
                "functional_owner": agent_plan["functional_owner"],
                "functional_owner_source": agent_plan["functional_owner_source"],
                "functional_role_id": agent_plan["functional_role_id"],
                "functional_display_name": agent_plan["functional_display_name"],
                "functional_focus": agent_plan["functional_focus"],
                "functional_tone": agent_plan["functional_tone"],
                "functional_preferred_skills": agent_plan["functional_preferred_skills"],
                "functional_handoff_trace": handoff_trace,
                "functional_owner_chain": handoff_chain,
                "runtime_plan": runtime_plan,
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
        skill_registry_metadata = self._load_skill_registry_metadata()
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
            registry_metadata = skill_registry_metadata.get(skill_name, {})
            required_skill_cards.append(
                {
                    "skill": skill_name,
                    "file": skill_file,
                    "category": category,
                    "focus": focus,
                    "summary": str(registry_metadata.get("summary", "")).strip(),
                    "summary_zh": str(registry_metadata.get("summary_zh", "")).strip(),
                    "display_name_zh": str(registry_metadata.get("display_name_zh", "")).strip(),
                    "when_to_use_zh": str(registry_metadata.get("when_to_use_zh", "")).strip(),
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

        functional_plan = self._load_task_functional_plan(task_id, capability_map)

        return {
            "required_mcp": required_mcp,
            "required_skills": required_skills,
            "required_skill_cards": required_skill_cards,
            "primary_agent": primary_agent,
            "review_agent": review_agent,
            "fallback_agent": fallback_agent,
            "runtime_plan": {
                "primary_agent": primary_agent,
                "review_agent": review_agent,
                "fallback_agent": fallback_agent,
            },
            "quality_gates": quality_gates,
            **functional_plan,
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
        
        if getattr(self, "interactive", False) and sys.stdin.isatty():
            print(f"\n\033[94m[Interactive Step]\033[0m Ready to execute agent: \033[1m{agent_name}\033[0m")
            print(f"Directory: {cwd}")
            if profile_directive:
                print(f"Profile context: {profile_directive[:100]}...")
            
            while True:
                choice = input("Proceed? [Y]es / [n]o, skip agent / [p]rompt view / [q]uit: ").strip().lower()
                if choice in ('', 'y', 'yes'):
                    print("\033[92mExecuting...\033[0m")
                    break
                elif choice in ('n', 'no'):
                    print("\033[93mSkipped agent execution.\033[0m")
                    return BridgeResponse.from_error(agent_name, "Skipped by user in interactive mode.")
                elif choice in ('q', 'quit'):
                    print("\033[91mAborted by user.\033[0m")
                    sys.exit(0)
                elif choice == 'p':
                    print("\n\033[90m--- PROMPT START ---\033[0m\n" + final_prompt + "\n\033[90m--- PROMPT END ---\033[0m\n")
                else:
                    print("Invalid choice. Please enter Y, n, p, or q.")
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

    def _normalize_domain(self, domain: str | None) -> str:
        raw = str(domain or "").strip().lower()
        if not raw or raw == "auto":
            return "auto"
        key = re.sub(r"[^a-z0-9]+", "-", raw).strip("-")
        return self.DOMAIN_PROFILE_ALIASES.get(key, key)

    def _load_domain_profile_context(self, domain: str | None) -> dict[str, Any]:
        requested = str(domain or "").strip()
        normalized = self._normalize_domain(domain)
        if normalized == "auto":
            return {
                "requested_domain": requested or "auto",
                "domain": "auto",
                "status": "auto",
                "display_name": "Auto-detect",
                "file": "",
                "excerpt": "",
            }

        profile_path = self.standards_dir.parent / "skills" / "domain-profiles" / f"{normalized}.yaml"
        if not profile_path.exists():
            return {
                "requested_domain": requested or normalized,
                "domain": normalized,
                "status": "missing",
                "display_name": normalized,
                "file": str(profile_path.relative_to(self.standards_dir.parent)),
                "excerpt": "",
            }

        try:
            content = profile_path.read_text(encoding="utf-8")
        except OSError:
            content = ""

        display_name = normalized
        if content:
            match = re.search(r'^display_name:\s*"?(.*?)"?\s*$', content, re.MULTILINE)
            if match and match.group(1).strip():
                display_name = match.group(1).strip()

        excerpt_lines = content.splitlines()[:120] if content else []
        excerpt = "\n".join(excerpt_lines).strip()
        return {
            "requested_domain": requested or normalized,
            "domain": normalized,
            "status": "loaded",
            "display_name": display_name,
            "file": str(profile_path.relative_to(self.standards_dir.parent)),
            "excerpt": excerpt,
        }

    def _build_domain_packet_fields(self, domain_context: dict[str, Any]) -> dict[str, Any]:
        if not domain_context:
            return {}
        return {
            "requested_domain": str(domain_context.get("requested_domain", "")).strip(),
            "domain": str(domain_context.get("domain", "")).strip(),
            "domain_profile_status": str(domain_context.get("status", "")).strip(),
            "domain_profile_display_name": str(domain_context.get("display_name", "")).strip(),
            "domain_profile_file": str(domain_context.get("file", "")).strip(),
            "domain_profile_excerpt": str(domain_context.get("excerpt", "")).strip(),
        }

    def _format_domain_context(self, task_packet: dict[str, Any]) -> str:
        zh_ui = get_language() == "zh-CN"
        domain = str(task_packet.get("domain", "")).strip()
        if not domain or domain == "auto":
            return "- No explicit domain profile injected." if not zh_ui else "- 当前未注入显式 domain profile。"

        status = str(task_packet.get("domain_profile_status", "")).strip() or "unknown"
        display_name = str(task_packet.get("domain_profile_display_name", "")).strip() or domain
        file_rel = str(task_packet.get("domain_profile_file", "")).strip() or "-"
        if zh_ui:
            lines = [
                f"- requested_domain / 用户请求领域: {str(task_packet.get('requested_domain', domain)).strip() or domain}",
                f"- resolved_domain / 解析后领域: {domain}",
                f"- status / 状态: {status}",
                f"- display_name / 显示名称: {display_name}",
                f"- profile_file / 配置文件: {file_rel}",
            ]
        else:
            lines = [
                f"- requested_domain: {str(task_packet.get('requested_domain', domain)).strip() or domain}",
                f"- resolved_domain: {domain}",
                f"- status: {status}",
                f"- display_name: {display_name}",
                f"- profile_file: {file_rel}",
            ]
        excerpt = str(task_packet.get("domain_profile_excerpt", "")).strip()
        if excerpt:
            lines.extend(
                [
                    "```yaml",
                    excerpt,
                    "```",
                ]
            )
        return "\n".join(lines)

    def _build_code_lane_rules(
        self,
        task_packet: dict[str, Any],
        stage: str,
    ) -> str:
        task_id = str(task_packet.get("task_id", "")).strip().upper()
        if not task_id.startswith("I"):
            return ""

        common_rules = [
            "Treat this as academic research code, not product/application scaffolding.",
            "Optimize for method fidelity, statistical validity, and reproducibility before generic software architecture polish.",
            "Keep outputs aligned to the contract paths under RESEARCH/[topic]/ and the code/ documentation artifacts listed in required_outputs.",
            "Document deterministic settings, dependency versions, validation datasets, and rerun commands explicitly.",
        ]
        phase_specific: list[str]
        if task_id == "I5":
            phase_specific = [
                "Produce a machine-parseable constraint set with explicit success criteria and disallowed shortcuts.",
                "Lock down I/O contracts, seeds, validation targets, and acceptance tests before any implementation freedom is introduced.",
            ]
        elif task_id == "I6":
            phase_specific = [
                "Transform the specification into a zero-decision execution plan with checkpoints, concrete commands, and rollback points.",
                "Expose which tasks can run in parallel and which must remain sequential due to statistical or dependency risk.",
            ]
        elif task_id == "I7":
            phase_specific = [
                "Execute the plan exactly; do not improvise new methodology without flagging it as a blocked decision.",
                "Implementation output should include runnable scripts/notebooks, validation evidence, profiling notes, and reproducibility instructions.",
            ]
        elif task_id == "I8":
            phase_specific = [
                "Review against the spec (I5), plan (I6), implementation outputs (I7), and reproducibility expectations.",
                "Prioritize findings that break statistical validity, method fidelity, or rerunnability over style concerns.",
            ]
        elif task_id == "I4":
            phase_specific = [
                "Audit seed handling, version pinning, environment capture, rerun commands, and fail-graceful contingencies.",
            ]
        else:
            phase_specific = [
                "Keep method assumptions, synthetic verification, and statistical diagnostics explicit.",
            ]

        if stage == "review":
            phase_specific.append(
                "Block if the work looks like generic engineering output rather than a method-faithful academic implementation."
            )
        if stage == "triad":
            phase_specific.append(
                "Resolve disagreements by favoring the narrowest implementation claim set that remains reproducible and statistically defensible."
            )

        lines = [f"{index}. {rule}" for index, rule in enumerate(common_rules + phase_specific, start=1)]
        return "\n".join(lines)

    def _build_code_lane_template_requirements(
        self,
        task_packet: dict[str, Any],
    ) -> str:
        task_id = str(task_packet.get("task_id", "")).strip().upper()
        if task_id == "I5":
            return """
Deliverable template requirements:
- The primary deliverable `code/code_specification.md` must use this exact heading skeleton:
  - `# Code Specification`
  - `## Spec Contract Block`
  - `## Goal`
  - `## Non-Goals`
  - `## Inputs (Schema)`
  - `## Outputs (Paths)`
  - `## Functional Requirements`
  - `## Non-Functional Requirements`
  - `## Edge Cases And Failure Modes`
  - `## Validation Matrix`
  - `## Disallowed Shortcuts`
  - `## Blocked Decisions / Escalations`
- The document must start with YAML frontmatter containing at least:
  - `task_id`, `template_type`, `topic`, `primary_artifact`
- Under `## Spec Contract Block`, include a fenced `json` block with at least:
  - `task_id`, `topic`, `method_or_pipeline`, `primary_artifact`, `inputs`, `outputs`, `dependencies`, `seeds_policy`, `acceptance_tests`, `blocked_decisions`
- `## Validation Matrix` must tie each test/check to a metric or observable pass condition.
- `## Disallowed Shortcuts` must name the shortcuts that would invalidate the research code or reproduction claim.
"""
        if task_id == "I6":
            return """
Deliverable template requirements:
- The primary deliverable `code/plan.md` must use this exact heading skeleton:
  - `# Execution Plan`
  - `## Plan Contract Block`
  - `## Scope Lock`
  - `## Assumptions From Spec`
  - `## Step Ledger`
  - `## Checkpoint Matrix`
  - `## Exact Run Commands`
  - `## Parallelization / Dependency Map`
  - `## Rollback / Recovery`
  - `## Risks / Blockers`
- The document must start with YAML frontmatter containing at least:
  - `task_id`, `template_type`, `topic`, `primary_artifact`
- Under `## Plan Contract Block`, include a fenced `json` block with at least:
  - `task_id`, `topic`, `spec_source`, `plan_artifact`, `steps`
- Each `steps` item must carry:
  - `step_id`, `depends_on`, `owner`, `command`, `outputs`, `checkpoint`, `rollback`
- `## Step Ledger` must be zero-decision: no step may rely on an implicit judgment call during execution.
- `## Exact Run Commands` must provide copy-pastable commands or notebook/script entrypoints.
"""
        if task_id == "I7":
            return """
Deliverable template requirements:
- The primary deliverable `code/performance_profile.md` must use this exact heading skeleton:
  - `# Performance Profile`
  - `## Execution Contract Block`
  - `## Scope Executed`
  - `## Implementation Ledger`
  - `## Validation Evidence`
  - `## Artifact Inventory`
  - `## Environment / Containerization`
  - `## Profiling Results`
  - `## Optimization Actions Taken`
  - `## Reproduction Commands`
  - `## Remaining Gaps / Blockers`
- The document must start with YAML frontmatter containing at least:
  - `task_id`, `template_type`, `topic`, `primary_artifact`
- Under `## Execution Contract Block`, include a fenced `json` block with at least:
  - `task_id`, `topic`, `plan_source`, `performance_artifact`, `analysis_outputs`, `documentation_outputs`, `container_outputs`, `validation_runs`, `profiling_targets`
- `## Implementation Ledger` must map executed steps back to `code/plan.md` step IDs.
- `## Validation Evidence` must record observable outcomes, not just claims that tests passed.
- `## Artifact Inventory` must enumerate what was produced under `analysis/`, `code/documentation/`, and `code/container_config/`.
- `## Reproduction Commands` must be copy-pastable rerun commands for the main pipeline and profiling path.
"""
        if task_id == "I4":
            return """
Deliverable template requirements:
- The primary deliverable `code/reproducibility_audit.md` must use this exact heading skeleton:
  - `# Reproducibility Audit`
  - `## Audit Contract Block`
  - `## Audit Scope`
  - `## Environment Evidence`
  - `## Data Provenance / Immutability`
  - `## Determinism / Seed Control`
  - `## Rerun Recipe`
  - `## Failure Points / Recovery`
  - `## Audit Verdict`
  - `## Required Remediations`
  - `## Confidence`
- The document must start with YAML frontmatter containing at least:
  - `task_id`, `template_type`, `topic`, `primary_artifact`
- Under `## Audit Contract Block`, include a fenced `json` block with at least:
  - `task_id`, `topic`, `audit_artifact`, `reviewed_artifacts`, `environment_files`, `seed_policy_status`, `rerun_entrypoints`, `verdict`, `blocking_gaps`
- `## Rerun Recipe` must provide a short ordered set of copy-pastable commands.
- `## Audit Verdict` must classify the repo as `PASS`, `WARN`, or `BLOCK` and justify that decision from the evidence.
"""
        if task_id == "I8":
            return """
Deliverable template requirements:
- The primary deliverable `code/code_review.md` must use this exact heading skeleton:
  - `# Code Review`
  - `## Review Contract Block`
  - `## Verdict`
  - `## Scope Reviewed`
  - `## Findings Table`
  - `## Blocking Findings`
  - `## Non-Blocking Findings`
  - `## Domain Checklist Status`
  - `## Reproducibility / Statistical Validity`
  - `## Required Fix Order`
  - `## Residual Risks`
  - `## Confidence`
- The document must start with YAML frontmatter containing at least:
  - `task_id`, `template_type`, `topic`, `primary_artifact`
- Under `## Review Contract Block`, include a fenced `json` block with at least:
  - `task_id`, `topic`, `review_target`, `spec_source`, `plan_source`, `review_artifact`, `verdict`, `blocking_findings`, `review_coverage`
- `## Findings Table` must classify each issue by `finding_id`, severity (`P0`/`P1`/`P2`/`P3`), area, evidence, and required action.
- `## Blocking Findings` must contain only items that justify a `BLOCK` verdict.
- `## Required Fix Order` must be explicitly prioritized.
"""
        return ""

    def _build_code_lane_review_requirements(
        self,
        task_packet: dict[str, Any],
    ) -> str:
        task_id = str(task_packet.get("task_id", "")).strip().upper()
        if task_id == "I5":
            return """
Stage-I structure checks:
- Block if `code/code_specification.md` is missing the required heading skeleton, YAML frontmatter, or the fenced `json` spec contract block.
- Block if acceptance tests are vague, non-measurable, or detached from the stated outputs.
- Block if blocked decisions or disallowed shortcuts are left implicit.
"""
        if task_id == "I6":
            return """
Stage-I structure checks:
- Block if `code/plan.md` is missing the required heading skeleton, YAML frontmatter, or the fenced `json` plan contract block.
- Block if any execution step lacks a concrete command, output path, checkpoint, or rollback path.
- Block if the plan still requires execution-time judgment calls rather than zero-decision instructions.
"""
        if task_id == "I7":
            return """
Stage-I structure checks:
- Block if `code/performance_profile.md` is missing the required heading skeleton, YAML frontmatter, or the fenced `json` execution contract block.
- Block if executed steps cannot be traced back to `code/plan.md`, or if validation evidence is only asserted rather than observed.
- Block if artifact inventory, profiling results, or reproduction commands are incomplete for the claimed outputs.
"""
        if task_id == "I4":
            return """
Stage-I structure checks:
- Block if `code/reproducibility_audit.md` is missing the required heading skeleton, YAML frontmatter, or the fenced `json` audit contract block.
- Block if environment evidence, seed handling, or rerun commands are missing or not actionable.
- Block if the audit verdict is not justified by concrete evidence and blocking gaps.
"""
        if task_id == "I8":
            return """
Stage-I structure checks:
- Block if `code/code_review.md` is missing the required heading skeleton, YAML frontmatter, or the fenced `json` review contract block.
- Block if findings are not severity-ranked or do not cite evidence from the spec, plan, code outputs, or reproducibility artifacts.
- Block if the verdict and blocking findings are inconsistent.
"""
        return ""

    def _normalize_code_build_focus(self, focus: str | None) -> str:
        raw = str(focus or "").strip().lower()
        if not raw:
            return "implementation"
        key = re.sub(r"[^a-z0-9]+", "_", raw).strip("_")
        return key or "implementation"

    def _build_code_build_context(
        self,
        method: str,
        tier: str,
        focus: str,
        domain_context: dict[str, Any],
        paper_path: str | None,
        strict_task_id: str | None = None,
    ) -> str:
        lines = [
            "Academic code workflow request:",
            f"- method: {method}",
            f"- tier: {tier}",
            f"- requested_focus: {focus}",
        ]
        if strict_task_id:
            lines.append(f"- strict_task_id: {strict_task_id}")
        domain_name = str(domain_context.get("domain", "auto")).strip() or "auto"
        lines.append(f"- domain: {domain_name}")
        if str(domain_context.get("display_name", "")).strip():
            lines.append(f"- domain_display_name: {domain_context['display_name']}")
        if paper_path:
            lines.append(f"- reference_paper: {paper_path}")
        lines.extend(
            [
                "- academic_code_requirements:",
                "  - preserve method fidelity to the stated methodology or paper",
                "  - prefer reproducibility artifacts over generic app scaffolding",
                "  - surface seeds, baseline comparisons, effect sizes / uncertainty, and failure cases",
                "  - align all outputs to RESEARCH/[topic]/ contract paths",
            ]
        )
        return "\n".join(lines)

    def _wrap_code_build_result(
        self,
        result: CollaborationResult,
        method: str,
        focus: str,
        task_id: str,
        topic: str,
    ) -> CollaborationResult:
        merged = "\n".join(
            [
                "## Code-Build Academic Flow",
                f"- method: {method}",
                f"- focus: {focus}",
                f"- mapped_task_id: {task_id}",
                f"- topic: {topic}",
                "",
                result.merged_analysis,
            ]
        )
        return CollaborationResult(
            mode="code-build",
            task_description=f"{method} {focus} {topic}"[:200],
            codex_response=result.codex_response,
            claude_response=result.claude_response,
            gemini_response=result.gemini_response,
            merged_analysis=merged,
            confidence=result.confidence,
            recommendations=list(result.recommendations),
            data=dict(result.data) if result.data else {},
        )

    def _resolve_code_build_target_map(
        self,
        mapped_task_id: str,
        only_targets: list[str] | None,
    ) -> dict[str, list[str]]:
        normalized_targets: list[str] = []
        seen_targets: set[str] = set()
        for raw in only_targets or []:
            item = str(raw).strip()
            if not item or item in seen_targets:
                continue
            seen_targets.add(item)
            normalized_targets.append(item)
        if not normalized_targets:
            return {}

        if mapped_task_id == "FULL":
            allowed_stages = {"I5", "I6", "I7", "I8"}
            stage_map: dict[str, list[str]] = {}
            stage_seen: dict[str, set[str]] = {}
            for selector in normalized_targets:
                stage_raw, separator, target_raw = selector.partition(":")
                stage_id = stage_raw.strip().upper()
                target_id = target_raw.strip()
                if not separator or not stage_id or not target_id:
                    raise ValueError(
                        "When --focus full is used, each --only-target must use STAGE_ID:TARGET format "
                        "(for example I6:S1 or I8:P1-01)."
                    )
                if stage_id not in allowed_stages:
                    raise ValueError(
                        "--focus full only supports --only-target selectors for I5, I6, I7, or I8."
                    )
                stage_map.setdefault(stage_id, [])
                stage_seen.setdefault(stage_id, set())
                if target_id in stage_seen[stage_id]:
                    continue
                stage_seen[stage_id].add(target_id)
                stage_map[stage_id].append(target_id)
            return stage_map

        if mapped_task_id not in self.STAGE_I_PRIMARY_ARTIFACT_BY_TASK:
            raise ValueError(
                "--only-target is currently supported only for structured Stage-I tasks I4 through I8."
            )

        resolved_targets: list[str] = []
        seen_resolved: set[str] = set()
        for selector in normalized_targets:
            stage_raw, separator, target_raw = selector.partition(":")
            if separator:
                stage_id = stage_raw.strip().upper()
                target_id = target_raw.strip()
                if stage_id != mapped_task_id or not target_id:
                    raise ValueError(
                        f"Selector '{selector}' does not match mapped task {mapped_task_id}."
                    )
            else:
                target_id = selector
            if target_id in seen_resolved:
                continue
            seen_resolved.add(target_id)
            resolved_targets.append(target_id)
        return {mapped_task_id: resolved_targets}

    def _parse_simple_frontmatter(self, content: str) -> tuple[dict[str, str], str]:
        match = re.match(r"^---\n(.*?)\n---\n?(.*)$", content, flags=re.DOTALL)
        if not match:
            return {}, content
        block = match.group(1)
        body = match.group(2)
        parsed: dict[str, str] = {}
        for line in block.splitlines():
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            item = re.match(r"^([A-Za-z0-9_-]+):\s*(.+?)\s*$", stripped)
            if not item:
                continue
            key, value = item.group(1), item.group(2)
            if (value.startswith('"') and value.endswith('"')) or (
                value.startswith("'") and value.endswith("'")
            ):
                value = value[1:-1]
            parsed[key] = value
        return parsed, body

    def _extract_fenced_block_after_heading(
        self,
        content: str,
        heading: str,
        language: str,
    ) -> str:
        heading_match = re.search(
            rf"^##\s+{re.escape(heading)}\s*$",
            content,
            flags=re.MULTILINE,
        )
        if not heading_match:
            return ""
        tail = content[heading_match.end():]
        block_match = re.search(
            rf"```{re.escape(language)}\s*\n(.*?)\n```",
            tail,
            flags=re.DOTALL,
        )
        if not block_match:
            return ""
        return block_match.group(1).strip()

    def _build_stage_i_actionable_targets(
        self,
        task_id: str,
        contract: dict[str, Any],
    ) -> list[str]:
        if task_id == "I5":
            return [
                str(item).strip()
                for item in contract.get("blocked_decisions", [])
                if str(item).strip()
            ]
        if task_id == "I6":
            return [
                str(step.get("step_id", "")).strip()
                for step in contract.get("steps", [])
                if isinstance(step, dict) and str(step.get("step_id", "")).strip()
            ]
        if task_id == "I7":
            return [
                str(run.get("step_id", "")).strip()
                for run in contract.get("validation_runs", [])
                if isinstance(run, dict) and str(run.get("step_id", "")).strip()
            ]
        if task_id == "I4":
            return [
                str(item.get("command", "")).strip()
                for item in contract.get("rerun_entrypoints", [])
                if isinstance(item, dict) and str(item.get("command", "")).strip()
            ]
        if task_id == "I8":
            return [
                str(item.get("finding_id", "")).strip()
                for item in contract.get("blocking_findings", [])
                if isinstance(item, dict) and str(item.get("finding_id", "")).strip()
            ]
        return []

    def _summarize_stage_i_structured_output(
        self,
        task_id: str,
        frontmatter: dict[str, str],
        contract: dict[str, Any],
        valid: bool,
        missing_frontmatter_keys: list[str],
        missing_contract_keys: list[str],
        parse_error: str,
    ) -> list[str]:
        lines = [
            f"task_id: {task_id}",
            f"template_type: {frontmatter.get('template_type', '<missing>')}",
            f"valid: {'yes' if valid else 'no'}",
        ]
        if parse_error:
            lines.append(f"parse_error: {parse_error}")
        if missing_frontmatter_keys:
            lines.append("missing_frontmatter_keys: " + ", ".join(missing_frontmatter_keys))
        if missing_contract_keys:
            lines.append("missing_contract_keys: " + ", ".join(missing_contract_keys))

        if task_id == "I5":
            lines.append(
                "acceptance_tests: "
                + str(len(contract.get("acceptance_tests", [])))
            )
            lines.append(
                "blocked_decisions: "
                + str(len(contract.get("blocked_decisions", [])))
            )
        elif task_id == "I6":
            step_ids = [
                str(step.get("step_id", "")).strip()
                for step in contract.get("steps", [])
                if isinstance(step, dict) and str(step.get("step_id", "")).strip()
            ]
            lines.append("steps: " + str(len(step_ids)))
            if step_ids:
                lines.append("step_ids: " + ", ".join(step_ids[:6]))
        elif task_id == "I7":
            lines.append(
                "validation_runs: "
                + str(len(contract.get("validation_runs", [])))
            )
            lines.append(
                "profiling_targets: "
                + str(len(contract.get("profiling_targets", [])))
            )
            lines.append(
                "analysis_outputs: "
                + str(len(contract.get("analysis_outputs", [])))
            )
        elif task_id == "I4":
            lines.append("verdict: " + str(contract.get("verdict", "<missing>")))
            lines.append(
                "seed_policy_status: "
                + str(contract.get("seed_policy_status", "<missing>"))
            )
            lines.append(
                "rerun_entrypoints: "
                + str(len(contract.get("rerun_entrypoints", [])))
            )
        elif task_id == "I8":
            lines.append("verdict: " + str(contract.get("verdict", "<missing>")))
            lines.append(
                "blocking_findings: "
                + str(len(contract.get("blocking_findings", [])))
            )
            lines.append(
                "review_coverage: "
                + str(len(contract.get("review_coverage", [])))
            )
        return lines

    def _parse_stage_i_structured_output(
        self,
        content: str,
        task_id: str,
    ) -> dict[str, Any]:
        normalized_task = task_id.strip().upper()
        if normalized_task not in self.STAGE_I_TEMPLATE_TYPE_BY_TASK:
            return {}

        expected_template = self.STAGE_I_TEMPLATE_TYPE_BY_TASK[normalized_task]
        heading = self.STAGE_I_CONTRACT_HEADING_BY_TASK[normalized_task]
        expected_contract_keys = self.STAGE_I_CONTRACT_KEYS_BY_TASK[normalized_task]
        frontmatter, body = self._parse_simple_frontmatter(content)
        contract_block = self._extract_fenced_block_after_heading(body, heading, "json")

        contract: dict[str, Any] = {}
        parse_error = ""
        if contract_block:
            try:
                loaded = json.loads(contract_block)
                if isinstance(loaded, dict):
                    contract = loaded
                else:
                    parse_error = "Contract block JSON root must be an object."
            except json.JSONDecodeError as exc:
                parse_error = str(exc)
        else:
            parse_error = "Missing JSON contract block."

        missing_frontmatter_keys = [
            key for key in self.STAGE_I_FRONTMATTER_KEYS if not frontmatter.get(key, "").strip()
        ]
        if frontmatter.get("task_id", "").strip() and frontmatter.get("task_id", "").strip() != normalized_task:
            missing_frontmatter_keys.append("task_id=<mismatch>")
        if (
            frontmatter.get("template_type", "").strip()
            and frontmatter.get("template_type", "").strip() != expected_template
        ):
            missing_frontmatter_keys.append("template_type=<mismatch>")

        missing_contract_keys = [
            key for key in expected_contract_keys if key not in contract
        ]
        valid = not parse_error and not missing_frontmatter_keys and not missing_contract_keys
        actionable_targets = self._build_stage_i_actionable_targets(normalized_task, contract)
        summary_lines = self._summarize_stage_i_structured_output(
            normalized_task,
            frontmatter,
            contract,
            valid,
            missing_frontmatter_keys,
            missing_contract_keys,
            parse_error,
        )
        return {
            "task_id": normalized_task,
            "expected_template_type": expected_template,
            "heading": heading,
            "frontmatter": frontmatter,
            "contract": contract,
            "missing_frontmatter_keys": missing_frontmatter_keys,
            "missing_contract_keys": missing_contract_keys,
            "parse_error": parse_error,
            "valid": valid,
            "actionable_targets": actionable_targets,
            "summary_lines": summary_lines,
        }

    def _structured_workspace_artifact_path(
        self,
        task_id: str,
        topic: str,
        cwd: Path,
    ) -> Path:
        normalized_task = task_id.strip().upper()
        if normalized_task not in self.STAGE_I_PRIMARY_ARTIFACT_BY_TASK:
            raise ValueError(
                f"Structured artifact lookup is only supported for Stage-I tasks I4-I8, not {task_id}."
            )
        artifact_root, _ = self._load_task_outputs(normalized_task)
        project_root = self._project_root_for_topic(
            cwd,
            artifact_root,
            self._normalize_topic(topic),
        )
        return project_root / self.STAGE_I_PRIMARY_ARTIFACT_BY_TASK[normalized_task]

    def _load_stage_i_structured_output_from_workspace(
        self,
        task_id: str,
        topic: str,
        cwd: Path,
    ) -> dict[str, Any]:
        normalized_task = task_id.strip().upper()
        artifact_path = self._structured_workspace_artifact_path(normalized_task, topic, cwd)
        try:
            content = artifact_path.read_text(encoding="utf-8")
        except OSError as exc:
            raise ValueError(
                f"Unable to load existing Stage-I artifact for {normalized_task}: {artifact_path} ({exc})"
            ) from exc

        structured_output = self._parse_stage_i_structured_output(content, normalized_task)
        if not structured_output:
            raise ValueError(
                f"Structured parsing returned no data for existing Stage-I artifact: {artifact_path}"
            )
        if not structured_output.get("valid", False):
            summary = "; ".join(
                str(item).strip()
                for item in structured_output.get("summary_lines", [])
                if str(item).strip()
            )
            raise ValueError(
                f"Existing Stage-I artifact is not valid for targeted follow-up: {artifact_path}"
                + (f" ({summary})" if summary else "")
            )

        return {
            "artifact_path": artifact_path,
            "artifact_relpath": self.STAGE_I_PRIMARY_ARTIFACT_BY_TASK[normalized_task],
            "content": content,
            "structured_output": structured_output,
        }

    def _resolve_targeted_follow_up(
        self,
        task_id: str,
        topic: str,
        cwd: Path,
        only_targets: list[str] | None,
    ) -> dict[str, Any]:
        normalized_task = task_id.strip().upper()
        selected_targets: list[str] = []
        seen_targets: set[str] = set()
        for raw in only_targets or []:
            item = str(raw).strip()
            if not item or item in seen_targets:
                continue
            seen_targets.add(item)
            selected_targets.append(item)
        if not selected_targets:
            raise ValueError("--only-target requires at least one non-empty target selector.")
        if normalized_task not in self.STAGE_I_PRIMARY_ARTIFACT_BY_TASK:
            raise ValueError("--only-target is only supported for structured Stage-I tasks I4-I8.")

        loaded = self._load_stage_i_structured_output_from_workspace(
            normalized_task,
            topic,
            cwd,
        )
        structured_output = dict(loaded["structured_output"])
        available_targets = [
            str(item).strip()
            for item in structured_output.get("actionable_targets", [])
            if str(item).strip()
        ]
        if not available_targets:
            raise ValueError(
                f"No actionable targets were parsed from existing Stage-I artifact for {normalized_task}: "
                f"{loaded['artifact_path']}"
            )
        missing_targets = [
            item for item in selected_targets if item not in set(available_targets)
        ]
        if missing_targets:
            raise ValueError(
                f"Unknown --only-target values for {normalized_task}: {', '.join(missing_targets)}. "
                f"Available targets: {', '.join(available_targets)}"
            )

        return {
            "targeted_follow_up": True,
            "selected_actionable_targets": selected_targets,
            "available_actionable_targets": available_targets,
            "structured_source_artifact": str(loaded["artifact_relpath"]),
            "structured_source_path": str(loaded["artifact_path"]),
            "structured_source_frontmatter": dict(structured_output.get("frontmatter", {})),
            "structured_source_contract": dict(structured_output.get("contract", {})),
            "structured_source_summary_lines": list(structured_output.get("summary_lines", [])),
        }

    def _build_targeted_follow_up_context(
        self,
        task_packet: dict[str, Any],
    ) -> str:
        if not bool(task_packet.get("targeted_follow_up")):
            return ""

        selected_targets = [
            str(item) for item in task_packet.get("selected_actionable_targets", []) if str(item).strip()
        ]
        available_targets = [
            str(item) for item in task_packet.get("available_actionable_targets", []) if str(item).strip()
        ]
        summary_lines = [
            str(item) for item in task_packet.get("structured_source_summary_lines", []) if str(item).strip()
        ]
        contract = task_packet.get("structured_source_contract", {})
        contract_json = (
            json.dumps(contract, ensure_ascii=False, indent=2)
            if isinstance(contract, dict) and contract
            else "{}"
        )
        lines = [
            "Targeted follow-up mode is active.",
            f"- source_artifact: {str(task_packet.get('structured_source_artifact', '')).strip() or '<missing>'}",
            f"- source_path: {str(task_packet.get('structured_source_path', '')).strip() or '<missing>'}",
            "- selected_actionable_targets: " + ", ".join(selected_targets),
            "- available_actionable_targets: " + ", ".join(available_targets),
        ]
        if summary_lines:
            lines.append("- source_summary_lines:")
            lines.extend(f"  - {item}" for item in summary_lines)
        lines.extend(
            [
                "- source_contract_json:",
                "```json",
                contract_json,
                "```",
                "Targeted follow-up rules:",
                "1. Only reopen the selected_actionable_targets. Preserve all unrelated decisions, steps, and sections.",
                "2. Do not broaden scope just because the existing artifact contains additional actionable targets.",
                "3. If a selected target cannot be resolved without touching another section, explain that dependency explicitly before changing it.",
                "4. Include an explicit Targeted Change Log that maps every change back to the selected target IDs.",
            ]
        )
        return "\n".join(lines)

    def _build_task_packet(
        self,
        task_id: str,
        paper_type: str,
        topic: str,
        venue: str | None,
        artifact_root: str,
        required_outputs: list[str],
        contract_required_outputs: list[str],
        deferred_outputs: list[str],
        required_mcp: list[str],
        required_skills: list[str],
        required_skill_cards: list[dict[str, Any]],
        quality_gates: list[str],
        artifact_policy: str,
        research_depth: str,
        evidence_expansion_rounds: int,
    ) -> dict[str, Any]:
        return {
            "task_id": task_id,
            "paper_type": paper_type,
            "topic": topic,
            "venue": venue or "",
            "artifact_root": artifact_root,
            "required_outputs": required_outputs,
            "contract_required_outputs": contract_required_outputs,
            "deferred_outputs": deferred_outputs,
            "required_mcp": required_mcp,
            "required_skills": required_skills,
            "required_skill_cards": required_skill_cards,
            "quality_gates": quality_gates,
            "artifact_policy": artifact_policy,
            "research_depth": research_depth,
            "evidence_expansion_rounds": evidence_expansion_rounds,
        }

    def _select_task_outputs(
        self,
        contract_outputs: list[str],
        focus_outputs: list[str] | None = None,
        output_budget: int | None = None,
    ) -> tuple[list[str], list[str], list[str], str]:
        normalized_contract = [
            str(item).strip() for item in contract_outputs if str(item).strip()
        ]
        if not normalized_contract:
            raise ValueError("Task has no contract outputs to execute.")

        selected_outputs = list(normalized_contract)
        artifact_policy = "contract"
        normalized_focus = [
            str(item).strip() for item in (focus_outputs or []) if str(item).strip()
        ]

        if normalized_focus:
            artifact_policy = "focused"
            missing_focus = [
                item for item in normalized_focus if item not in normalized_contract
            ]
            if missing_focus:
                raise ValueError(
                    "Focused outputs must exist in the task contract. Unknown outputs: "
                    + ", ".join(missing_focus)
                )
            seen_focus: set[str] = set()
            selected_outputs = []
            for item in normalized_focus:
                if item in seen_focus:
                    continue
                seen_focus.add(item)
                selected_outputs.append(item)

        if output_budget is not None:
            if output_budget <= 0:
                raise ValueError("--output-budget must be greater than 0.")
            artifact_policy = "focused"
            selected_outputs = selected_outputs[:output_budget]

        if not selected_outputs:
            raise ValueError("No active outputs selected for this run.")

        selected_set = set(selected_outputs)
        deferred_outputs = [
            item for item in normalized_contract if item not in selected_set
        ]
        return normalized_contract, selected_outputs, deferred_outputs, artifact_policy

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
                    "summary": str(card.get("summary", "")).strip(),
                    "summary_zh": str(card.get("summary_zh", "")).strip(),
                    "display_name_zh": str(card.get("display_name_zh", "")).strip(),
                    "when_to_use_zh": str(card.get("when_to_use_zh", "")).strip(),
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
        zh_ui = get_language() == "zh-CN"
        if not skill_cards:
            return "- No required skill cards available." if not zh_ui else "- 当前没有可用的必需 skill 卡片。"
        lines: list[str] = []
        status_map = {
            "ok": "ok / 可用",
            "missing": "missing / 缺失",
        } if zh_ui else {}
        for card in skill_cards:
            skill_name = str(card.get("skill", "")).strip() or "unknown-skill"
            category = str(card.get("category", "")).strip() or "unspecified"
            summary = str(card.get("summary", "")).strip()
            summary_zh = str(card.get("summary_zh", "")).strip()
            display_name_zh = str(card.get("display_name_zh", "")).strip()
            when_to_use_zh = str(card.get("when_to_use_zh", "")).strip()
            focus = str(card.get("focus", "")).strip() or "No focus provided."
            status = str(card.get("status", "ok")).strip() or "ok"
            status_label = status_map.get(status, status)
            outputs = [
                str(item)
                for item in card.get("default_outputs", [])
                if str(item).strip()
            ]
            file_rel = str(card.get("file", "")).strip() or "-"
            display_summary = summary_zh if zh_ui and summary_zh else summary
            if not display_summary:
                display_summary = "未提供摘要。" if zh_ui else "No summary provided."
            if zh_ui and display_name_zh:
                lines.append(f"- {display_name_zh} (`{skill_name}`) [{status_label}] ({category})")
            else:
                lines.append(f"- {skill_name} [{status_label}] ({category})")
            lines.append(
                f"  {'summary' if not zh_ui else '摘要/summary'}: {display_summary}"
            )
            if zh_ui and when_to_use_zh:
                lines.append(
                    f"  适用场景/when_to_use: {when_to_use_zh}"
                )
            if focus:
                lines.append(
                    f"  {'focus' if not zh_ui else '方法焦点/focus'}: {focus}"
                )
            lines.append(f"  {'spec' if not zh_ui else '规范文件/spec'}: {file_rel}")
            if outputs:
                lines.append(
                    f"  {'default_outputs' if not zh_ui else '默认产物/default_outputs'}: {', '.join(outputs)}"
                )
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
        zh_ui = get_language() == "zh-CN"
        if not evidence_items:
            return "- No MCP evidence collected." if not zh_ui else "- 当前没有采集到 MCP 证据。"
        status_map = {
            "ok": "ok / 可用",
            "error": "error / 错误",
            "not_configured": "not_configured / 未配置",
            "missing": "missing / 缺失",
        } if zh_ui else {}
        lines: list[str] = []
        for item in evidence_items:
            status_label = status_map.get(item.status, item.status)
            lines.append(f"- {item.provider} [{status_label}]: {item.summary}")
            if item.provenance:
                lines.append(
                    f"  {'provenance' if not zh_ui else '来源/provenance'}: {', '.join(item.provenance[:3])}"
                )
        return "\n".join(lines)

    def _build_task_draft_prompt(
        self,
        task_packet: dict[str, Any],
        mcp_evidence: list[MCPEvidence],
        skill_cards: list[dict[str, Any]],
        extra_context: str | None,
    ) -> str:
        deferred_outputs = [
            str(item) for item in task_packet.get("deferred_outputs", []) if str(item).strip()
        ]
        artifact_policy = str(task_packet.get("artifact_policy", "contract")).strip() or "contract"
        research_depth = str(task_packet.get("research_depth", "standard")).strip() or "standard"
        evidence_rounds = int(task_packet.get("evidence_expansion_rounds", 1) or 1)
        output_control_lines = [
            f"- artifact_policy: {artifact_policy}",
            "- required_outputs_for_this_run: "
            + ", ".join(
                str(item) for item in task_packet.get("required_outputs", []) if str(item).strip()
            ),
        ]
        if deferred_outputs:
            output_control_lines.append(
                "- deferred_contract_outputs: " + ", ".join(deferred_outputs)
            )

        depth_rules = ""
        return_sections = [
            "- Execution Plan",
            "- Draft Outputs (by file path)",
        ]
        if research_depth == "deep":
            depth_rules = f"""
9. Deep research mode is active. Start by narrowing the problem into explicit subquestions and non-goals.
10. Perform at least {evidence_rounds} evidence-expansion passes:
   - pass 1: direct supporting evidence and foundational sources
   - pass 2+: contradiction search, boundary cases, and citation snowballing when available
11. Prefer fewer, better-supported conclusions over broad but weak coverage.
12. Separate direct evidence, inference, and unresolved gaps explicitly.
"""
            return_sections.extend(
                [
                    "- Scope Boundaries",
                    "- Evidence Expansion Log",
                    "- Counterevidence Check",
                ]
            )
        if deferred_outputs:
            return_sections.append("- Deferred Outputs")
        targeted_follow_up_section = self._build_targeted_follow_up_context(task_packet)
        if targeted_follow_up_section:
            return_sections.append("- Targeted Change Log")
        return_sections.extend(
            [
                "- Quality Gate Check",
                "- Missing Inputs",
                "- Next Actions",
            ]
        )
        domain_section = ""
        if str(task_packet.get("domain", "")).strip() and str(task_packet.get("domain", "")).strip() != "auto":
            domain_section = f"""
Domain profile guidance:
{self._format_domain_context(task_packet)}
"""
        code_lane_rules = self._build_code_lane_rules(task_packet, "draft")
        code_lane_section = ""
        if code_lane_rules:
            code_lane_section = f"""
Academic code lane rules:
{code_lane_rules}
"""
        code_lane_template = self._build_code_lane_template_requirements(task_packet)
        code_lane_template_section = ""
        if code_lane_template:
            code_lane_template_section = f"""
Structured deliverable template:
{code_lane_template}
"""
        targeted_section = ""
        targeted_rules = ""
        if targeted_follow_up_section:
            targeted_section = f"""
Targeted follow-up context:
{targeted_follow_up_section}
"""
            targeted_rules = """
13. Targeted follow-up mode is active. Revise only the selected actionable targets and keep unrelated sections stable.
14. Reuse the existing structured artifact as the authority for non-selected targets unless you explicitly mark a dependency.
"""
        return f"""You are executing one canonical research workflow task.

Task packet (JSON):
{json.dumps(task_packet, ensure_ascii=False, indent=2)}

Execution rules:
1. Follow sequence: plan -> mcp-evidence -> draft -> self-check.
2. Produce deliverables aligned to required_outputs for this run and include those exact paths.
3. Do not create extra helper files outside required_outputs unless the task packet explicitly requires them.
4. If deferred_outputs is non-empty, keep those outputs deferred for this run and explain the defer rationale instead of silently broadening scope.
5. For each deliverable, cite the MCP evidence source used.
6. Apply every required skill in required_skills as a method constraint in your draft process.
   - Use required_skill_cards for concrete method focus and default outputs.
   - Respect functional_owner as the accountable research owner for this task.
   - Preserve upstream assumptions recorded in functional_handoff_trace.
7. Explicitly check quality_gates and mark each as PASS/WARN/FAIL.
8. If required input is missing, list it under "Missing Inputs" and continue with placeholders.
{depth_rules}
{targeted_rules}

Output control:
{chr(10).join(output_control_lines)}

MCP evidence snapshot:
{self._format_mcp_evidence(mcp_evidence)}

Required skill cards:
{self._format_skill_context(skill_cards)}
{domain_section}{code_lane_section}{code_lane_template_section}{targeted_section}

Additional context:
{extra_context or "No additional context."}

Return sections:
{chr(10).join(return_sections)}
"""

    def _build_task_review_prompt(
        self,
        task_packet: dict[str, Any],
        mcp_evidence: list[MCPEvidence],
        skill_cards: list[dict[str, Any]],
        draft_output: str,
        revision_round: int = 0,
    ) -> str:
        """Build review prompt with self-critique question injection."""
        task_id = str(task_packet.get("task_id", "")).strip()
        critique_qs = get_critique_questions(task_id)
        research_depth = str(task_packet.get("research_depth", "standard")).strip() or "standard"
        deferred_outputs = [
            str(item) for item in task_packet.get("deferred_outputs", []) if str(item).strip()
        ]

        critique_section = ""
        if critique_qs:
            formatted = "\n".join(f"   {i+1}. {q}" for i, q in enumerate(critique_qs))
            critique_section = f"""
Stage-specific critique questions (MUST address each):
{formatted}
"""

        round_note = ""
        if revision_round > 0:
            round_note = f"\nThis is review round {revision_round + 1}. Be stricter on issues that were flagged in previous rounds but not adequately addressed.\n"

        depth_review = ""
        dynamic_question_count = "2-3"
        return_sections = [
            "- Verdict (PASS/BLOCK)",
            "- Critical Issues",
            "- Suggested Fixes",
        ]
        if deferred_outputs:
            return_sections.append("- Deferred Output Assessment")
        if research_depth == "deep":
            dynamic_question_count = "4-6"
            depth_review = """
8. In deep research mode, verify that the draft narrows scope before claiming breadth.
9. Check whether the draft explicitly searched for counterevidence, contradictions, and edge cases.
10. Block if the output is broad but weakly supported, or if evidence expansion is only superficial.
"""
            return_sections.append("- Evidence Depth Assessment")
        targeted_follow_up_section = self._build_targeted_follow_up_context(task_packet)
        if targeted_follow_up_section:
            return_sections.append("- Target Resolution Check")
        return_sections.append("- Confidence (0-1)")
        domain_section = ""
        if str(task_packet.get("domain", "")).strip() and str(task_packet.get("domain", "")).strip() != "auto":
            domain_section = f"""
Domain profile guidance:
{self._format_domain_context(task_packet)}
"""
        code_lane_rules = self._build_code_lane_rules(task_packet, "review")
        code_lane_section = ""
        if code_lane_rules:
            code_lane_section = f"""
Academic code lane rules:
{code_lane_rules}
"""
        code_lane_review = self._build_code_lane_review_requirements(task_packet)
        code_lane_review_section = ""
        if code_lane_review:
            code_lane_review_section = f"""
Structured deliverable review requirements:
{code_lane_review}
"""
        targeted_section = ""
        targeted_review_rule = ""
        if targeted_follow_up_section:
            targeted_section = f"""
Targeted follow-up context:
{targeted_follow_up_section}
"""
            targeted_review_rule = """
11. When targeted follow-up mode is active, assess whether the selected actionable targets were resolved without reopening unrelated scope.
"""

        return f"""Review the draft for this canonical research workflow task.
{round_note}
Task packet (JSON):
{json.dumps(task_packet, ensure_ascii=False, indent=2)}

Primary draft:
{draft_output}

MCP evidence snapshot:
{self._format_mcp_evidence(mcp_evidence)}

Required skill cards:
{self._format_skill_context(skill_cards)}
{domain_section}{code_lane_section}{code_lane_review_section}{targeted_section}

Review checklist:
1. Output path coverage against required_outputs.
2. If deferred_outputs exist, confirm the draft stayed within scope and justified the deferral.
3. Quality gate compliance against quality_gates.
4. Required_skills usage fidelity and completeness.
5. Internal consistency (claims/methods/evidence alignment).
6. Missing citations, assumptions, or unresolved risks.
7. Functional-owner scope fit and handoff consistency against functional_handoff_trace.
{depth_review}
{targeted_review_rule}
{critique_section}

Dynamic Literature-Based Critique:
- Before evaluating, analyze the provided MCP evidence (especially literature references and methodology details).
- Formulate {dynamic_question_count} highly specific, critical questions based on the gaps, controversies, or limitations identified in those exact papers.
- Explicitly evaluate whether the primary draft successfully addresses these dynamic, literature-specific nuances.

IMPORTANT: You MUST include a clear verdict line in your response:
- Verdict: PASS  (if all critical requirements are met)
- Verdict: BLOCK (if there are critical issues that must be fixed)
And a confidence score:
- Confidence: <float between 0 and 1>

Return sections:
{chr(10).join(return_sections)}
"""

    def _build_task_triad_prompt(
        self,
        task_packet: dict[str, Any],
        mcp_evidence: list[MCPEvidence],
        skill_cards: list[dict[str, Any]],
        draft_output: str,
        review_output: str,
    ) -> str:
        research_depth = str(task_packet.get("research_depth", "standard")).strip() or "standard"
        deferred_outputs = [
            str(item) for item in task_packet.get("deferred_outputs", []) if str(item).strip()
        ]
        extra_checks = ""
        return_sections = [
            "- Triad Verdict (PASS/BLOCK)",
            "- Consensus Status (AGREE/PARTIAL/CONFLICT)",
            "- Highest-Priority Fixes",
        ]
        if deferred_outputs:
            return_sections.append("- Deferred Output Decision")
        if research_depth == "deep":
            extra_checks = """
6. Verify the draft/review pair did real contradiction hunting instead of only summarizing the most visible evidence.
7. Force a decision on the narrowest claim set that remains strongly supported.
"""
            return_sections.append("- Evidence Conflict Resolution")
        targeted_follow_up_section = self._build_targeted_follow_up_context(task_packet)
        if targeted_follow_up_section:
            return_sections.append("- Target Resolution Decision")
        return_sections.append("- Confidence (0-1)")
        domain_section = ""
        if str(task_packet.get("domain", "")).strip() and str(task_packet.get("domain", "")).strip() != "auto":
            domain_section = f"""
Domain profile guidance:
{self._format_domain_context(task_packet)}
"""
        code_lane_rules = self._build_code_lane_rules(task_packet, "triad")
        code_lane_section = ""
        if code_lane_rules:
            code_lane_section = f"""
Academic code lane rules:
{code_lane_rules}
"""
        code_lane_review = self._build_code_lane_review_requirements(task_packet)
        code_lane_review_section = ""
        if code_lane_review:
            code_lane_review_section = f"""
Structured deliverable review requirements:
{code_lane_review}
"""
        targeted_section = ""
        targeted_checks = ""
        if targeted_follow_up_section:
            targeted_section = f"""
Targeted follow-up context:
{targeted_follow_up_section}
"""
            targeted_checks = """
6. Decide whether the selected actionable targets are now resolved, still blocked, or incorrectly widened into unrelated scope.
"""
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
{domain_section}{code_lane_section}{code_lane_review_section}{targeted_section}

Audit checklist:
1. Identify unresolved disagreements between draft and review.
2. Verify contract output paths and quality gates.
3. Check claim-method-evidence integrity.
4. Prioritize top 3 fixes by impact.
5. Confirm the draft stays within the functional_owner scope and handoff chain.
{targeted_checks}
{extra_checks}

Return sections:
{chr(10).join(return_sections)}
"""

    def _parse_review_verdict(self, review_content: str) -> tuple[str, float]:
        """Parse review output to extract verdict (PASS/BLOCK) and confidence.

        Returns ('PASS' | 'BLOCK', confidence_float).
        Defaults to ('BLOCK', 0.5) if parsing fails.
        """
        verdict = "BLOCK"
        confidence = 0.5

        verdict_patterns = [
            re.compile(r"Verdict\s*[:\(]\s*(PASS|BLOCK)", re.IGNORECASE),
            re.compile(
                r"^\s*-?\s*(?:Overall\s+)?Verdict\s*:\s*(PASS|BLOCK)",
                re.IGNORECASE | re.MULTILINE,
            ),
        ]
        for pattern in verdict_patterns:
            match = pattern.search(review_content)
            if match:
                verdict = match.group(1).upper()
                break

        conf_patterns = [
            re.compile(
                r"Confidence\s*(?:\(0-1\))?\s*[:\-]\s*(0(?:\.\d+)?|1(?:\.0+)?)",
                re.IGNORECASE,
            ),
            re.compile(
                r"^\s*-?\s*Confidence\s*:\s*(0(?:\.\d+)?|1(?:\.0+)?)",
                re.IGNORECASE | re.MULTILINE,
            ),
        ]
        for pattern in conf_patterns:
            match = pattern.search(review_content)
            if match:
                try:
                    confidence = float(match.group(1))
                    confidence = max(0.0, min(1.0, confidence))
                except ValueError:
                    pass
                break

        # Convergence: high confidence auto-passes even if labeled BLOCK
        if confidence >= 0.85 and verdict == "BLOCK":
            verdict = "PASS"

        return verdict, confidence

    def _build_task_revision_prompt(
        self,
        task_packet: dict[str, Any],
        mcp_evidence: list[MCPEvidence],
        skill_cards: list[dict[str, Any]],
        previous_draft: str,
        review_feedback: str,
        revision_round: int,
    ) -> str:
        """Build a revision prompt that integrates review feedback."""
        task_json = json.dumps(task_packet, ensure_ascii=False, indent=2)
        mcp_section = self._format_mcp_evidence(mcp_evidence)
        skill_section = self._format_skill_context(skill_cards)
        domain_section = ""
        if str(task_packet.get("domain", "")).strip() and str(task_packet.get("domain", "")).strip() != "auto":
            domain_section = (
                "Domain profile guidance:\n"
                f"{self._format_domain_context(task_packet)}\n\n"
            )
        code_lane_rules = self._build_code_lane_rules(task_packet, "draft")
        code_lane_section = ""
        if code_lane_rules:
            code_lane_section = (
                "Academic code lane rules:\n"
                f"{code_lane_rules}\n\n"
            )
        code_lane_template = self._build_code_lane_template_requirements(task_packet)
        code_lane_template_section = ""
        if code_lane_template:
            code_lane_template_section = (
                "Structured deliverable template:\n"
                f"{code_lane_template}\n\n"
            )
        targeted_follow_up_section = self._build_targeted_follow_up_context(task_packet)
        targeted_section = ""
        targeted_revision_rules = ""
        targeted_return_sections = ""
        if targeted_follow_up_section:
            targeted_section = (
                "Targeted follow-up context:\n"
                f"{targeted_follow_up_section}\n\n"
            )
            targeted_revision_rules = (
                "7. Targeted follow-up mode is active. Resolve only the selected actionable targets unless the review proves a dependency.\n"
                "8. Preserve wording and structure for unrelated sections whenever possible.\n"
            )
            targeted_return_sections = "- Targeted Change Log\n"
        return (
            f"You are revising a research workflow task draft based on review feedback.\n"
            f"This is revision round {revision_round}.\n\n"
            f"Task packet (JSON):\n{task_json}\n\n"
            f"Your previous draft:\n{previous_draft}\n\n"
            f"Review feedback (address ALL issues):\n{review_feedback}\n\n"
            f"MCP evidence snapshot:\n{mcp_section}\n\n"
            f"Required skill cards:\n{skill_section}\n\n"
            f"{domain_section}"
            f"{code_lane_section}"
            f"{code_lane_template_section}"
            f"{targeted_section}"
            "Revision rules:\n"
            "1. Address every Critical Issue raised in the review.\n"
            "2. Apply every Suggested Fix unless you can justify why it is not applicable.\n"
            "3. Do NOT regenerate from scratch — revise the existing draft.\n"
            "4. Clearly mark what changed with inline comments like [REVISED: reason].\n"
            "5. Re-check all quality_gates and mark each as PASS/WARN/FAIL.\n"
            "6. If any issue cannot be resolved, explain why under 'Unresolved Issues'.\n"
            f"{targeted_revision_rules}\n"
            "Return sections:\n"
            "- Revision Summary (what changed and why)\n"
            f"{targeted_return_sections}"
            "- Revised Draft Outputs (by file path)\n"
            "- Quality Gate Check\n"
            "- Unresolved Issues\n"
            "- Next Actions\n"
        )

    # ── Team-Run (Research Fanout/Fanin) ──────────────────────────────

    def _load_team_run_config(self, task_id: str) -> dict[str, Any]:
        """Load team_run_config for a specific task from capability map."""
        capability_map = self._read_standard("mcp-agent-capability-map.yaml")
        section = self._extract_top_level_section(capability_map, "team_run_config")
        if not section:
            raise ConfigError(
                ERR_CFG_INVALID_TASK,
                detail=f"team_run_config section missing in capability map",
            )
        task_match = re.search(
            rf"^\s{{2}}{re.escape(task_id)}:\n((?:^\s{{4}}.*\n?)+)",
            section,
            flags=re.MULTILINE,
        )
        if not task_match:
            raise ConfigError(
                ERR_CFG_INVALID_TASK,
                detail=f"Task {task_id} not found in team_run_config. MVP supports: B1, H3",
            )
        block = task_match.group(1)
        config: dict[str, Any] = {
            "task_id": task_id,
            "execution_mode": self._parse_yaml_scalar(block, "execution_mode", indent=4),
            "partition_strategy": self._parse_yaml_scalar(block, "partition_strategy", indent=4),
            "max_parallel_units": int(
                self._parse_yaml_scalar(block, "max_parallel_units", indent=4) or "3"
            ),
            "planner_agent": self._parse_yaml_scalar(block, "planner_agent", indent=4) or "claude",
            "merge_agent": self._parse_yaml_scalar(block, "merge_agent", indent=4) or "claude",
            "consensus_policy": self._parse_yaml_scalar(block, "consensus_policy", indent=4),
        }
        # Barrier rules
        barrier_section = self._extract_nested_section(block, "barrier_rules", indent=4)
        raw_min = self._parse_yaml_scalar(barrier_section, "min_success_ratio", indent=6) or "0.6"
        raw_failure = self._parse_yaml_scalar(barrier_section, "on_failure", indent=6) or "degrade"
        # Strip inline YAML comments (e.g. "degrade  # degrade | block | retry")
        raw_min = raw_min.split("#")[0].strip()
        raw_failure = raw_failure.split("#")[0].strip()
        config["barrier_rules"] = {
            "min_success_ratio": float(raw_min),
            "on_failure": raw_failure,
        }
        # Worker pool
        worker_pool_section = self._extract_nested_section(block, "worker_pool", indent=4)
        config["worker_pool"] = self._parse_yaml_list(worker_pool_section, item_indent=6) or [
            "codex", "claude", "gemini"
        ]
        # Review pool
        review_pool_section = self._extract_nested_section(block, "review_pool", indent=4)
        config["review_pool"] = self._parse_yaml_list(review_pool_section, item_indent=6) or [
            "codex", "gemini"
        ]
        # Shard / canonical outputs
        shard_section = self._extract_nested_section(block, "shard_outputs", indent=4)
        config["shard_outputs"] = self._parse_yaml_list(shard_section, item_indent=6)
        canonical_section = self._extract_nested_section(block, "canonical_outputs", indent=4)
        config["canonical_outputs"] = self._parse_yaml_list(canonical_section, item_indent=6)
        # Personas (H3)
        personas: list[dict[str, str]] = []
        persona_section = self._extract_nested_section(block, "personas", indent=4)
        if persona_section:
            # Handle flat YAML: "      - id: X\n        focus: Y"
            for pm in re.finditer(
                r"^\s*-\s+id:\s*(.+?)\s*$",
                persona_section,
                flags=re.MULTILINE,
            ):
                p_id = pm.group(1).strip().strip('"').strip("'")
                # Find the focus line following this id line
                rest = persona_section[pm.end():]
                focus_match = re.match(r"\s*\n\s*focus:\s*(.+?)\s*$", rest, flags=re.MULTILINE)
                p_focus = ""
                if focus_match:
                    p_focus = focus_match.group(1).strip().strip('"').strip("'")
                if p_id:
                    personas.append({"id": p_id, "focus": p_focus})
        config["personas"] = personas
        return config

    def _generate_work_units(
        self,
        team_config: dict[str, Any],
        task_packet: dict[str, Any],
        mcp_evidence: list[MCPEvidence],
        skill_cards: list[dict[str, Any]],
        cwd: Path,
        run_id: str,
    ) -> list[dict[str, Any]]:
        """Generate work units: persona-based (static) or planner-based (dynamic)."""
        partition = team_config.get("partition_strategy", "")
        topic = task_packet.get("topic", "unknown")
        artifact_root = task_packet.get("artifact_root", "RESEARCH/[topic]/")
        shard_base = f"{artifact_root.replace('[topic]', topic)}runs/{run_id}/shards"

        if partition == "by_reviewer_persona" and team_config.get("personas"):
            units = []
            for persona in team_config["personas"]:
                unit_id = persona["id"]
                units.append({
                    "unit_id": unit_id,
                    "persona": persona,
                    "shard_root": f"{shard_base}/{unit_id}/",
                    "partition_strategy": partition,
                })
            return units

        # Dynamic planner-based partitioning (e.g. B1 by_paper_batch)
        max_units = team_config.get("max_parallel_units", 3)
        planner_prompt = self._build_planner_prompt(
            team_config, task_packet, mcp_evidence, skill_cards, max_units,
        )
        planner_agent = team_config.get("planner_agent", "claude")
        planner_runtime, _ = self._resolve_runtime_agent(planner_agent, ["claude", "gemini", "codex"])
        planner_resp = self._execute_runtime_agent(planner_runtime, planner_prompt, cwd)

        if planner_resp.success:
            units = self._parse_planner_output(planner_resp.content, shard_base, partition)
            if units:
                return units[:max_units]

        # Fallback: single unit (degrade gracefully)
        return [{
            "unit_id": "batch_1",
            "description": "Single-unit fallback (planner did not produce structured output)",
            "shard_root": f"{shard_base}/batch_1/",
            "partition_strategy": partition,
        }]

    def _build_planner_prompt(
        self,
        team_config: dict[str, Any],
        task_packet: dict[str, Any],
        mcp_evidence: list[MCPEvidence],
        skill_cards: list[dict[str, Any]],
        max_units: int,
    ) -> str:
        return f"""You are a research task planner. Split the following task into parallel work units.

Task packet (JSON):
{json.dumps(task_packet, ensure_ascii=False, indent=2)}

Partition strategy: {team_config.get("partition_strategy", "by_paper_batch")}
Maximum work units: {max_units}

MCP evidence:
{self._format_mcp_evidence(mcp_evidence)}

Required skill cards:
{self._format_skill_context(skill_cards)}

IMPORTANT: Return ONLY a JSON array of work unit objects. Each object must have:
- "unit_id": string identifier (e.g. "batch_1", "batch_2")
- "description": brief description of what this unit covers
- "scope": specific scope of the sub-task (queries, paper IDs, or topic facets)

Example output:
[
  {{"unit_id": "batch_1", "description": "AI teaching methods", "scope": "papers on AI-based pedagogy"}},
  {{"unit_id": "batch_2", "description": "AI assessment tools", "scope": "papers on AI-driven evaluation"}}
]
"""

    def _parse_planner_output(
        self, content: str, shard_base: str, partition: str,
    ) -> list[dict[str, Any]]:
        """Extract JSON array of work units from planner output."""
        json_match = re.search(r"\[[\s\S]*?\]", content)
        if not json_match:
            return []
        try:
            raw_units = json.loads(json_match.group())
        except json.JSONDecodeError:
            return []
        if not isinstance(raw_units, list):
            return []
        units = []
        for item in raw_units:
            if not isinstance(item, dict):
                continue
            uid = str(item.get("unit_id", f"unit_{len(units)+1}")).strip()
            units.append({
                "unit_id": uid,
                "description": str(item.get("description", "")).strip(),
                "scope": str(item.get("scope", "")).strip(),
                "shard_root": f"{shard_base}/{uid}/",
                "partition_strategy": partition,
            })
        return units

    def _build_worker_prompt(
        self,
        work_unit: dict[str, Any],
        task_packet: dict[str, Any],
        mcp_evidence: list[MCPEvidence],
        skill_cards: list[dict[str, Any]],
        team_config: dict[str, Any],
    ) -> str:
        """Build isolated prompt for a single worker."""
        persona_block = ""
        if work_unit.get("persona"):
            p = work_unit["persona"]
            persona_block = f"""
Reviewer Persona:
- ID: {p.get("id", "unknown")}
- Focus: {p.get("focus", "")}
You MUST adopt this specific reviewer perspective throughout your review.
"""

        scope_block = ""
        if work_unit.get("scope"):
            scope_block = f"\nAssigned scope: {work_unit['scope']}\n"
        if work_unit.get("description"):
            scope_block += f"Description: {work_unit['description']}\n"

        shard_outputs = team_config.get("shard_outputs", [])
        shard_root = work_unit.get("shard_root", "shard/")
        output_paths = [f"  - {shard_root}{f}" for f in shard_outputs]

        return f"""You are one of several parallel workers executing a research task.
You MUST write your outputs ONLY to your assigned shard directory.

Work Unit ID: {work_unit["unit_id"]}
Shard Root: {shard_root}
{persona_block}{scope_block}
Task packet (JSON):
{json.dumps(task_packet, ensure_ascii=False, indent=2)}

Your shard output files:
{chr(10).join(output_paths)}

MCP evidence:
{self._format_mcp_evidence(mcp_evidence)}

Required skill cards:
{self._format_skill_context(skill_cards)}

Rules:
1. Stay within your assigned scope/persona ONLY.
2. Write outputs ONLY under your shard root: {shard_root}
3. Do NOT write to any canonical output path directly.
4. Cite evidence sources for every finding.
5. Apply all required_skills constraints.

Return your shard deliverables as structured output.
"""

    def _fanout_execute(
        self,
        work_units: list[dict[str, Any]],
        task_packet: dict[str, Any],
        mcp_evidence: list[MCPEvidence],
        skill_cards: list[dict[str, Any]],
        team_config: dict[str, Any],
        cwd: Path,
        profile_cfg: dict[str, Any],
        profile_name: str,
    ) -> list[dict[str, Any]]:
        """Execute work units in parallel using ThreadPoolExecutor."""
        worker_pool = team_config.get("worker_pool", ["codex", "claude", "gemini"])
        results: list[dict[str, Any]] = []

        def execute_worker(idx: int, unit: dict[str, Any]) -> dict[str, Any]:
            agent_name = worker_pool[idx % len(worker_pool)]
            runtime, _ = self._resolve_runtime_agent(agent_name, worker_pool)
            prompt = self._build_worker_prompt(
                unit, task_packet, mcp_evidence, skill_cards, team_config,
            )
            resp = self._execute_runtime_agent(
                runtime,
                prompt,
                cwd,
                self._profile_runtime_options(profile_cfg, runtime),
                self._build_profile_directive(profile_name, profile_cfg, stage="draft"),
            )
            return {
                "unit_id": unit["unit_id"],
                "shard_root": unit.get("shard_root", ""),
                "agent": runtime,
                "success": resp.success,
                "content": resp.content if resp.success else "",
                "error": resp.error if not resp.success else None,
            }

        with ThreadPoolExecutor(max_workers=len(work_units)) as executor:
            futures = {
                executor.submit(execute_worker, i, unit): unit["unit_id"]
                for i, unit in enumerate(work_units)
            }
            for future in as_completed(futures):
                try:
                    results.append(future.result())
                except Exception as exc:
                    unit_id = futures[future]
                    results.append({
                        "unit_id": unit_id,
                        "shard_root": "",
                        "agent": "unknown",
                        "success": False,
                        "content": "",
                        "error": str(exc),
                    })
        return results

    def _apply_failure_policy(
        self,
        shard_results: list[dict[str, Any]],
        barrier_rules: dict[str, Any],
    ) -> tuple[list[dict[str, Any]], str, list[str]]:
        """Apply barrier rules. Returns (successful_shards, status, notes)."""
        total = len(shard_results)
        successes = [s for s in shard_results if s.get("success")]
        failures = [s for s in shard_results if not s.get("success")]
        notes: list[str] = []

        if not total:
            return [], "blocked", ["No work units were dispatched."]

        success_ratio = len(successes) / total
        min_ratio = barrier_rules.get("min_success_ratio", 0.6)
        policy = barrier_rules.get("on_failure", "degrade")

        for f in failures:
            notes.append(
                f"Worker {f['unit_id']} failed ({f.get('agent', '?')}): {f.get('error', 'unknown')}"
            )

        if not failures:
            return successes, "ok", notes

        if policy == "block":
            notes.append("Barrier policy=block: halting because not all workers succeeded.")
            return [], "blocked", notes

        if policy == "degrade":
            if success_ratio >= min_ratio:
                notes.append(
                    f"Barrier policy=degrade: {len(successes)}/{total} succeeded "
                    f"(ratio={success_ratio:.2f} >= {min_ratio}). Proceeding with available shards."
                )
                return successes, "degraded", notes
            else:
                notes.append(
                    f"Barrier policy=degrade: {len(successes)}/{total} succeeded "
                    f"(ratio={success_ratio:.2f} < {min_ratio}). Blocked."
                )
                return [], "blocked", notes

        # retry policy — in MVP we treat retry as degrade after noting intent
        notes.append(f"Barrier policy=retry: retry not fully implemented in MVP; treating as degrade.")
        if success_ratio >= min_ratio:
            return successes, "degraded", notes
        return [], "blocked", notes

    def _build_merge_prompt(
        self,
        successful_shards: list[dict[str, Any]],
        task_packet: dict[str, Any],
        team_config: dict[str, Any],
    ) -> str:
        """Build merge prompt combining all shard outputs."""
        shard_blocks = []
        for shard in successful_shards:
            shard_blocks.append(
                f"### Shard: {shard['unit_id']} (agent: {shard.get('agent', '?')})\n"
                f"{shard.get('content', '[empty]')}"
            )
        joined_shards = "\n\n---\n\n".join(shard_blocks)
        canonical_outputs = team_config.get("canonical_outputs", [])
        consensus_policy = team_config.get("consensus_policy", "majority_rules")

        return f"""You are the merge agent for a team-run research task.
Multiple parallel workers have each produced shard outputs. Your job is to
merge them into a single canonical output.

Task packet (JSON):
{json.dumps(task_packet, ensure_ascii=False, indent=2)}

Consensus policy: {consensus_policy}
Total shards received: {len(successful_shards)}

Worker shard outputs:
{joined_shards}

Canonical output files to produce:
{chr(10).join(f"  - {f}" for f in canonical_outputs)}

Merge rules:
1. Do NOT simply concatenate — synthesize, deduplicate, and reconcile.
2. Include a CONFLICT SUMMARY section documenting inter-shard disagreements.
3. Include a CONSENSUS SUMMARY section documenting converging findings.
4. Include a GAP SUMMARY section identifying areas no shard covered.
5. The final output must be a coherent, unified deliverable — not fragments.
6. Mark the provenance of each major finding (which shard contributed it).

Return sections:
- Merged Output (by canonical file)
- Conflict Summary
- Consensus Summary
- Gap Summary
"""

    def team_run(
        self,
        task_id: str,
        paper_type: str,
        topic: str,
        cwd: Path,
        venue: str | None = None,
        context: str | None = None,
        max_parallel_units: int | None = None,
        mcp_strict: bool = False,
        skills_strict: bool = False,
        profile_file: Path | None = None,
        profile: str = "default",
    ) -> CollaborationResult:
        """Run team-based fanout/fanin parallel execution for a research task.

        MVP supports Task IDs: B1 (systematic review) and H3 (peer-review simulation).
        """
        import uuid

        normalized_task = task_id.strip().upper()
        normalized_topic = self._normalize_topic(topic)
        routing_notes: list[str] = []
        run_id = str(uuid.uuid4())[:8]

        # 1. Load team-run config
        team_config = self._load_team_run_config(normalized_task)
        if max_parallel_units is not None:
            team_config["max_parallel_units"] = max_parallel_units
        routing_notes.append(
            f"Team-run config loaded for {normalized_task}: "
            f"partition={team_config['partition_strategy']}, "
            f"max_units={team_config['max_parallel_units']}, "
            f"consensus={team_config['consensus_policy']}."
        )

        # 2. Load standard task metadata (reuses task-run infrastructure)
        try:
            agent_plan = self._load_task_agent_plan(normalized_task)
            artifact_root, required_outputs = self._load_task_outputs(normalized_task)
        except (FileNotFoundError, ValueError) as exc:
            raise ConfigError(ERR_CFG_INVALID_TASK, detail=str(exc)) from exc

        try:
            profile_registry, _ = self._load_profile_bundle(profile_file)
            profile_cfg = self._resolve_profile_config(profile, profile_registry)
        except ValueError as exc:
            raise ConfigError(ERR_CFG_INVALID_PROFILE, detail=str(exc)) from exc

        packet = self._build_task_packet(
            task_id=normalized_task,
            paper_type=paper_type,
            topic=normalized_topic,
            venue=venue,
            artifact_root=artifact_root,
            required_outputs=required_outputs,
            contract_required_outputs=required_outputs,
            deferred_outputs=[],
            required_mcp=agent_plan["required_mcp"],
            required_skills=agent_plan["required_skills"],
            required_skill_cards=agent_plan["required_skill_cards"],
            quality_gates=agent_plan["quality_gates"],
            artifact_policy="contract",
            research_depth="standard",
            evidence_expansion_rounds=1,
        )
        packet["team_run_config"] = {
            "run_id": run_id,
            "partition_strategy": team_config["partition_strategy"],
            "max_parallel_units": team_config["max_parallel_units"],
        }

        # 3. Collect skill cards and MCP evidence
        try:
            skill_cards, skill_notes = self._collect_skill_context(packet, strict=skills_strict)
            packet["required_skill_cards"] = skill_cards
            routing_notes.extend(skill_notes)
        except ValueError as exc:
            return CollaborationResult(
                mode="team-run",
                task_description=f"{normalized_task} {paper_type} {normalized_topic}"[:200],
                merged_analysis=str(exc),
                confidence=0.0,
                recommendations=[],
            )

        try:
            mcp_evidence, mcp_notes = self._collect_mcp_evidence(packet, cwd, strict=mcp_strict)
            routing_notes.extend(mcp_notes)
        except ValueError as exc:
            return CollaborationResult(
                mode="team-run",
                task_description=f"{normalized_task} {paper_type} {normalized_topic}"[:200],
                merged_analysis=str(exc),
                confidence=0.0,
                recommendations=[],
            )

        # 4. Generate work units
        work_units = self._generate_work_units(
            team_config, packet, mcp_evidence, skill_cards, cwd, run_id,
        )
        routing_notes.append(f"Generated {len(work_units)} work units for run_id={run_id}.")
        for unit in work_units:
            routing_notes.append(f"  - {unit['unit_id']}: shard={unit.get('shard_root', '?')}")

        # 5. Fanout execute
        shard_results = self._fanout_execute(
            work_units, packet, mcp_evidence, skill_cards,
            team_config, cwd, profile_cfg, profile,
        )

        # 6. Apply failure policy
        barrier_rules = team_config.get("barrier_rules", {})
        successful_shards, barrier_status, barrier_notes = self._apply_failure_policy(
            shard_results, barrier_rules,
        )
        routing_notes.extend(barrier_notes)

        # 7. Merge shards
        merge_resp: BridgeResponse | None = None
        merge_runtime: str | None = None
        if barrier_status != "blocked" and successful_shards:
            merge_prompt = self._build_merge_prompt(successful_shards, packet, team_config)
            merge_agent = team_config.get("merge_agent", "claude")
            merge_runtime, merge_notes = self._resolve_runtime_agent(merge_agent, ["claude", "gemini", "codex"])
            routing_notes.extend(merge_notes)
            merge_resp = self._execute_runtime_agent(
                merge_runtime,
                merge_prompt,
                cwd,
                self._profile_runtime_options(profile_cfg, merge_runtime),
                self._build_profile_directive(profile, profile_cfg, stage="summary"),
            )
            routing_notes.append(
                f"Merge completed by {merge_runtime}: "
                f"{'success' if merge_resp.success else 'failed'}."
            )

        # 8. Review merge result
        review_resp: BridgeResponse | None = None
        review_runtime: str | None = None
        if merge_resp and merge_resp.success:
            review_pool = team_config.get("review_pool", ["codex", "gemini"])
            preferred_reviewer = review_pool[0] if review_pool else "codex"
            review_runtime, review_notes = self._resolve_runtime_agent(
                preferred_reviewer, review_pool, exclude_agent=merge_runtime,
            )
            routing_notes.extend(review_notes)
            review_prompt = self._build_task_review_prompt(
                packet, mcp_evidence, skill_cards, merge_resp.content,
            )
            review_resp = self._execute_runtime_agent(
                review_runtime,
                review_prompt,
                cwd,
                self._profile_runtime_options(profile_cfg, review_runtime),
                self._build_profile_directive(profile, profile_cfg, stage="review"),
            )

        # 9. Assemble result
        codex_resp = None
        claude_resp = None
        gemini_resp = None
        for runtime_agent, response in (
            (merge_runtime, merge_resp),
            (review_runtime, review_resp),
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
            f"## Team-Run: {normalized_task} (run_id={run_id})",
            "",
            f"- Partition strategy: {team_config['partition_strategy']}",
            f"- Work units dispatched: {len(work_units)}",
            f"- Successful shards: {len(successful_shards)}/{len(shard_results)}",
            f"- Barrier status: {barrier_status}",
            f"- Consensus policy: {team_config['consensus_policy']}",
            f"- Profile: {profile}",
            "",
            "## Routing Notes",
            "\n".join(f"- {n}" for n in routing_notes),
            "",
            "## Worker Shard Results",
        ]
        for sr in shard_results:
            status = "✓" if sr["success"] else "✗"
            merged_parts.append(
                f"### [{status}] {sr['unit_id']} (agent: {sr.get('agent', '?')})"
            )
            if sr["success"]:
                merged_parts.append(sr.get("content", "")[:2000])
            else:
                merged_parts.append(f"[FAILED] {sr.get('error', 'unknown')}")
            merged_parts.append("")

        if merge_resp:
            merged_parts.extend([
                "## Merge Output" + (f" ({merge_runtime})" if merge_runtime else ""),
                merge_resp.content if merge_resp.success else f"[FAILED] {merge_resp.error}",
                "",
            ])
        elif barrier_status == "blocked":
            merged_parts.extend([
                "## Merge Output",
                "[BLOCKED] Merge was not attempted because barrier rules were not met.",
                "",
            ])

        if review_resp:
            merged_parts.extend([
                f"## Post-Merge Review ({review_runtime})",
                review_resp.content if review_resp.success else f"[FAILED] {review_resp.error}",
                "",
            ])

        merged = "\n".join(merged_parts)

        # Confidence calculation
        if barrier_status == "blocked":
            confidence = 0.0
        elif merge_resp and merge_resp.success and review_resp and review_resp.success:
            confidence = 0.92
        elif merge_resp and merge_resp.success:
            confidence = 0.78
        elif barrier_status == "degraded":
            confidence = 0.45
        else:
            confidence = 0.3

        return CollaborationResult(
            mode="team-run",
            task_description=f"{normalized_task} {paper_type} {normalized_topic}"[:200],
            codex_response=codex_resp,
            claude_response=claude_resp,
            gemini_response=gemini_resp,
            merged_analysis=merged,
            confidence=confidence,
            recommendations=self._extract_recommendations(merged),
        )

    def task_run(
        self,
        task_id: str,
        paper_type: str,
        topic: str,
        cwd: Path,
        domain: str = "auto",
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
        max_revision_rounds: int = 2,
        focus_outputs: list[str] | None = None,
        output_budget: int | None = None,
        research_depth: str = "standard",
        only_targets: list[str] | None = None,
    ) -> CollaborationResult:
        """Run task-level orchestration using capability map and contract."""
        normalized_task = task_id.strip().upper()
        normalized_topic = self._normalize_topic(topic)
        routing_notes: list[str] = []
        depth_mode = research_depth.strip().lower()
        if depth_mode not in {"standard", "deep"}:
            raise ValueError("research_depth must be 'standard' or 'deep'.")
        domain_context = self._load_domain_profile_context(domain)

        try:
            agent_plan = self._load_task_agent_plan(normalized_task)
            artifact_root, contract_outputs = self._load_task_outputs(normalized_task)
        except (FileNotFoundError, ValueError) as exc:
            msg = f"Task '{normalized_task}' is invalid or missing in workflow contract."
            raise ConfigError(ERR_CFG_INVALID_TASK, detail=msg) from exc

        contract_outputs, required_outputs, deferred_outputs, artifact_policy = self._select_task_outputs(
            contract_outputs,
            focus_outputs=focus_outputs,
            output_budget=output_budget,
        )
        evidence_expansion_rounds = 2 if depth_mode == "deep" else 1

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
            contract_required_outputs=contract_outputs,
            deferred_outputs=deferred_outputs,
            required_mcp=agent_plan["required_mcp"],
            required_skills=agent_plan["required_skills"],
            required_skill_cards=agent_plan["required_skill_cards"],
            quality_gates=agent_plan["quality_gates"],
            artifact_policy=artifact_policy,
            research_depth=depth_mode,
            evidence_expansion_rounds=evidence_expansion_rounds,
        )
        packet.update(self._build_domain_packet_fields(domain_context))
        if plan_result.data:
            packet["task_plan"] = dict(plan_result.data)
        else:
            packet["task_plan"] = {
                "status": "unavailable",
                "detail": plan_result.merged_analysis,
            }
        functional_handoff_trace = [
            dict(item)
            for item in packet["task_plan"].get("functional_handoff_trace", [])
            if isinstance(item, dict)
        ]
        functional_owner_chain = [
            str(item)
            for item in packet["task_plan"].get("functional_owner_chain", [])
            if str(item).strip()
        ]
        packet.update(
            {
                "functional_owner": agent_plan["functional_owner"],
                "functional_owner_source": agent_plan["functional_owner_source"],
                "functional_role_id": agent_plan["functional_role_id"],
                "functional_display_name": agent_plan["functional_display_name"],
                "functional_focus": agent_plan["functional_focus"],
                "functional_tone": agent_plan["functional_tone"],
                "functional_preferred_skills": agent_plan["functional_preferred_skills"],
                "functional_handoff_trace": functional_handoff_trace,
                "functional_owner_chain": functional_owner_chain,
                "runtime_plan": dict(agent_plan.get("runtime_plan", {})),
            }
        )
        zh_ui = get_language() == "zh-CN"
        routing_notes.append(
            f"Functional owner resolved for {normalized_task}: "
            f"{agent_plan['functional_owner']} "
            f"(role={agent_plan['functional_role_id']}, "
            f"source={agent_plan['functional_owner_source']})."
            + (" / 已完成功能负责人解析。" if zh_ui else "")
        )
        if agent_plan.get("functional_focus"):
            routing_notes.append(
                f"Functional focus: {agent_plan['functional_focus']}"
                + (" / 当前功能关注点。" if zh_ui else "")
            )
        if functional_owner_chain:
            routing_notes.append(
                "Functional handoff chain: " + " -> ".join(functional_owner_chain) + "."
                + (" / 功能交接链已展开。" if zh_ui else "")
            )
        routing_notes.append(
            "Runtime plan: "
            f"draft={agent_plan['primary_agent']}, "
            f"review={agent_plan['review_agent']}, "
            f"fallback={agent_plan['fallback_agent']}."
            + (" / 运行预案已确认。" if zh_ui else "")
        )
        routing_notes.append(
            "Output control: "
            f"policy={artifact_policy}, active_outputs={len(required_outputs)}/{len(contract_outputs)}."
            + (" / 产物输出范围已收敛。" if zh_ui else "")
        )
        if str(packet.get("domain", "")).strip() and str(packet.get("domain", "")).strip() != "auto":
            routing_notes.append(
                "Domain profile: "
                f"requested={packet.get('requested_domain', packet.get('domain', 'auto'))}, "
                f"resolved={packet.get('domain', 'auto')}, "
                f"status={packet.get('domain_profile_status', 'unknown')}."
                + (" / 已注入领域画像。" if zh_ui else "")
            )
            if packet.get("domain_profile_file"):
                routing_notes.append(
                    f"Domain profile file: {packet['domain_profile_file']}."
                    + (" / 领域画像文件路径。" if zh_ui else "")
                )
        if required_outputs:
            routing_notes.append(
                "Active outputs: " + ", ".join(required_outputs) + "."
                + (" / 当前激活产物。" if zh_ui else "")
            )
        if deferred_outputs:
            routing_notes.append(
                "Deferred outputs: " + ", ".join(deferred_outputs) + "."
                + (" / 延后产物。" if zh_ui else "")
            )
        routing_notes.append(
            f"Research depth: {depth_mode} (evidence_expansion_rounds={evidence_expansion_rounds})."
            + (" / 研究深度设置。" if zh_ui else "")
        )
        targeted_follow_up_data: dict[str, Any] = {}
        if only_targets:
            try:
                targeted_follow_up_data = self._resolve_targeted_follow_up(
                    normalized_task,
                    normalized_topic,
                    cwd,
                    only_targets,
                )
            except ValueError as exc:
                raise ConfigError(ERR_CFG_INVALID_TASK, detail=str(exc)) from exc
            packet.update(targeted_follow_up_data)
            routing_notes.append(
                "Targeted follow-up: selected_targets="
                + ", ".join(packet.get("selected_actionable_targets", []))
                + "."
            )
            routing_notes.append(
                "Targeted source artifact: "
                + str(packet.get("structured_source_artifact", "<missing>"))
                + "."
            )
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
        revision_history: list[dict[str, Any]] = []
        if draft_resp.success:
            review_runtime, review_notes = self._resolve_runtime_agent(
                preferred_agent=agent_plan["review_agent"],
                fallback_chain=[agent_plan["fallback_agent"]],
                exclude_agent=draft_runtime,
            )
            routing_notes.extend(review_notes)

            # --- Revision loop ---
            current_draft_content = draft_resp.content
            effective_max_rounds = max(0, max_revision_rounds)
            if depth_mode == "deep":
                effective_max_rounds = max(effective_max_rounds, 3)
            for revision_round in range(effective_max_rounds + 1):
                review_prompt = self._build_task_review_prompt(
                    packet,
                    mcp_evidence,
                    packet.get("required_skill_cards", []),
                    current_draft_content,
                    revision_round=revision_round,
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

                if not review_resp or not review_resp.success:
                    routing_notes.append(
                        f"Review failed at round {revision_round}; stopping revision loop."
                    )
                    break

                verdict, review_confidence = self._parse_review_verdict(review_resp.content)
                revision_history.append({
                    "round": revision_round,
                    "verdict": verdict,
                    "confidence": review_confidence,
                })

                if verdict == "PASS":
                    routing_notes.append(
                        f"Review PASSED at round {revision_round} (confidence={review_confidence:.2f})."
                    )
                    break

                # BLOCK — revise if we have rounds left
                if revision_round < effective_max_rounds:
                    routing_notes.append(
                        f"Review BLOCKED at round {revision_round} (confidence={review_confidence:.2f}); revising."
                    )
                    revision_prompt = self._build_task_revision_prompt(
                        packet,
                        mcp_evidence,
                        packet.get("required_skill_cards", []),
                        current_draft_content,
                        review_resp.content,
                        revision_round=revision_round + 1,
                    )
                    revised_resp = self._execute_runtime_agent(
                        draft_runtime,
                        revision_prompt,
                        cwd,
                        self._profile_runtime_options(draft_profile_cfg, draft_runtime),
                        self._build_profile_directive(
                            selected_profiles["draft"],
                            draft_profile_cfg,
                            stage="draft",
                        ),
                    )
                    if revised_resp.success:
                        current_draft_content = revised_resp.content
                        draft_resp = revised_resp
                    else:
                        routing_notes.append(
                            f"Revision draft failed at round {revision_round + 1}; keeping previous draft."
                        )
                        break
                else:
                    routing_notes.append(
                        f"Review BLOCKED at final round {revision_round} (confidence={review_confidence:.2f}); max rounds reached."
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

        structured_output: dict[str, Any] = {}
        if draft_resp.success and normalized_task in self.STAGE_I_TEMPLATE_TYPE_BY_TASK:
            structured_output = self._parse_stage_i_structured_output(
                draft_resp.content,
                normalized_task,
            )
            routing_notes.append(
                "Structured artifact parse: "
                f"valid={structured_output.get('valid', False)} for {normalized_task}."
            )
            if structured_output.get("actionable_targets"):
                routing_notes.append(
                    "Structured actionable targets: "
                    + ", ".join(str(item) for item in structured_output["actionable_targets"][:8])
                    + "."
                )

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
        ]
        if structured_output:
            merged_parts.extend(
                [
                    "## Structured Artifact Summary",
                    "\n".join(
                        f"- {line}" for line in structured_output.get("summary_lines", [])
                    ),
                    "",
                ]
            )
        merged_parts.extend(
            [
            get_text("draft", agent=draft_runtime),
            draft_resp.content if draft_resp.success else f"[FAILED] {draft_resp.error}",
            ]
        )
        if revision_history:
            merged_parts.extend([
                "",
                get_text("revision_history"),
                "\n".join(
                    f"- Round {r['round']}: {r['verdict']} (confidence={r['confidence']:.2f})"
                    for r in revision_history
                ),
            ])
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
            data={
                "task_packet": packet,
                "routing_notes": list(routing_notes),
                "revision_history": list(revision_history),
                "structured_output": dict(structured_output) if structured_output else {},
                "targeted_follow_up": {
                    "enabled": bool(targeted_follow_up_data),
                    "selected_targets": list(packet.get("selected_actionable_targets", [])),
                    "available_targets": list(packet.get("available_actionable_targets", [])),
                    "source_artifact": str(packet.get("structured_source_artifact", "")).strip(),
                    "source_path": str(packet.get("structured_source_path", "")).strip(),
                },
            },
        )



    def code_build(
        self,
        method: str,
        cwd: Path,
        domain: str = "auto",
        tier: str = "standard",
        paper_path: str | None = None,
        topic: str | None = None,
        focus: str | None = None,
        paper_type: str = "methods",
        triad: bool = False,
        only_targets: list[str] | None = None,
    ) -> CollaborationResult:
        """
        Build academic research code.

        Args:
            method: Methodology name (e.g. "DID", "GARCH")
            cwd: Working directory
            domain: finance, econ, metrics, cs, or auto
            tier: standard (template) or advanced (decomposition)
            paper_path: Optional path to paper for context
            topic: Research topic slug/name. When provided, route into strict Stage-I task flow.
            focus: Requested code lane focus, mapped to Stage-I Task IDs.
            paper_type: Workflow paper type used for strict task routing.
            triad: Enable independent third-agent audit for strict task routing.
        """
        normalized_focus = self._normalize_code_build_focus(focus)
        mapped_task_id = self.CODE_BUILD_FOCUS_TO_TASK.get(normalized_focus, "I1")
        domain_context = self._load_domain_profile_context(domain)

        if topic:
            normalized_topic = self._normalize_topic(topic)
            try:
                selected_target_map = self._resolve_code_build_target_map(mapped_task_id, only_targets)
            except ValueError as exc:
                raise ConfigError(ERR_CFG_INVALID_TASK, detail=str(exc)) from exc
            default_stage_sequence = ["I5", "I6", "I7", "I8"] if mapped_task_id == "FULL" else [mapped_task_id]
            if mapped_task_id == "FULL" and selected_target_map:
                stage_sequence = [
                    task_id for task_id in default_stage_sequence if task_id in selected_target_map
                ]
            else:
                stage_sequence = list(default_stage_sequence)
            stage_results: list[tuple[str, CollaborationResult]] = []
            combined_codex: BridgeResponse | None = None
            combined_claude: BridgeResponse | None = None
            combined_gemini: BridgeResponse | None = None
            structured_stage_outputs: dict[str, Any] = {}
            aggregated_actionable_targets: dict[str, list[str]] = {}

            for stage_task_id in stage_sequence:
                stage_context = self._build_code_build_context(
                    method=method,
                    tier=tier,
                    focus=normalized_focus,
                    domain_context=domain_context,
                    paper_path=paper_path,
                    strict_task_id=stage_task_id,
                )
                if stage_results:
                    completed = ", ".join(task for task, _ in stage_results)
                    stage_context += (
                        "\n- upstream_stage_contract:\n"
                        f"  - previously_completed: {completed}\n"
                        "  - treat earlier stage outputs as authoritative unless you identify a concrete blocker\n"
                        "  - do not reopen settled decisions without explicit evidence"
                    )
                stage_depth = "deep" if tier == "advanced" or stage_task_id in {"I4", "I5", "I6", "I8"} else "standard"
                stage_result = self.task_run(
                    task_id=stage_task_id,
                    paper_type=paper_type,
                    topic=normalized_topic,
                    cwd=cwd,
                    domain=domain,
                    context=stage_context,
                    triad=triad if stage_task_id == "I8" else False,
                    profile="focused-delivery",
                    draft_profile="deep-research" if stage_depth == "deep" else "focused-delivery",
                    review_profile="strict-review",
                    triad_profile="deep-research",
                    max_revision_rounds=3 if stage_depth == "deep" else 2,
                    research_depth=stage_depth,
                    only_targets=selected_target_map.get(stage_task_id),
                )
                stage_results.append((stage_task_id, stage_result))
                structured_output = {}
                if stage_result.data and isinstance(stage_result.data, dict):
                    structured_output = dict(stage_result.data.get("structured_output", {}) or {})
                if structured_output:
                    structured_stage_outputs[stage_task_id] = structured_output
                    actionable = [
                        str(item)
                        for item in structured_output.get("actionable_targets", [])
                        if str(item).strip()
                    ]
                    if actionable:
                        aggregated_actionable_targets[stage_task_id] = actionable
                if combined_codex is None and stage_result.codex_response:
                    combined_codex = stage_result.codex_response
                if combined_claude is None and stage_result.claude_response:
                    combined_claude = stage_result.claude_response
                if combined_gemini is None and stage_result.gemini_response:
                    combined_gemini = stage_result.gemini_response

                if stage_result.confidence <= 0.0:
                    break

            if mapped_task_id != "FULL" and stage_results:
                wrapped = self._wrap_code_build_result(
                    stage_results[0][1],
                    method=method,
                    focus=normalized_focus,
                    task_id=stage_results[0][0],
                    topic=normalized_topic,
                )
                if selected_target_map:
                    wrapped.data = dict(wrapped.data or {})
                    wrapped.data["selected_target_map"] = dict(selected_target_map)
                return wrapped

            merged_sections = [
                "## Code-Build Academic Flow",
                f"- method: {method}",
                f"- focus: {normalized_focus}",
                f"- topic: {normalized_topic}",
                f"- domain: {domain_context.get('domain', 'auto') or 'auto'}",
                "- strict_stage_sequence: " + " -> ".join(task for task, _ in stage_results),
                "",
            ]
            if selected_target_map:
                merged_sections.extend(
                    [
                        "- selected_target_map: "
                        + "; ".join(
                            f"{stage_id}={', '.join(targets)}"
                            for stage_id, targets in selected_target_map.items()
                        ),
                        "",
                    ]
                )
            for stage_task_id, stage_result in stage_results:
                merged_sections.extend(
                    [
                        f"### {stage_task_id}",
                        stage_result.merged_analysis,
                        "",
                    ]
                )
            confidence = min((item.confidence for _, item in stage_results), default=0.0)
            return CollaborationResult(
                mode="code-build",
                task_description=f"{method} {normalized_focus} {normalized_topic}"[:200],
                codex_response=combined_codex,
                claude_response=combined_claude,
                gemini_response=combined_gemini,
                merged_analysis="\n".join(merged_sections).strip(),
                confidence=confidence,
                recommendations=self._extract_recommendations("\n".join(merged_sections)),
                data={
                    "topic": normalized_topic,
                    "focus": normalized_focus,
                    "mapped_task_id": mapped_task_id,
                    "stage_sequence": [task for task, _ in stage_results],
                    "domain": domain_context.get("domain", "auto"),
                    "structured_stage_outputs": structured_stage_outputs,
                    "actionable_targets": aggregated_actionable_targets,
                    "selected_target_map": dict(selected_target_map),
                },
            )

        domain_packet = self._build_domain_packet_fields(domain_context)
        domain_section = self._format_domain_context(domain_packet)
        request_context = self._build_code_build_context(
            method=method,
            tier=tier,
            focus=normalized_focus,
            domain_context=domain_context,
            paper_path=paper_path,
        )

        # Backward-compatible non-topic prompt path
        if tier == "advanced":
            prompt = f"""
You are an expert Research Software Engineer.
Task: Implement the methodology '{method}' from first principles (Tier 2 Advanced Mode).
Domain guidance:
{domain_section}

REQUIREMENTS:
1. Do NOT use high-level library wrappers.
2. If this is an optimization problem, define the Objective Function (Likelihood/Loss) explicitly.
3. Use JAX or PyTorch if gradients are needed.
4. If this is a structural model, ensure all equations are translated to code.
5. Treat this as academic research code, not generic application scaffolding.

CONTEXT:
{request_context}

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
Domain guidance:
{domain_section}

REQUIREMENTS:
1. Use standard, robust libraries (e.g. statsmodels, arch, linearmodels, pyfixest).
2. Follow best practices for the domain.
3. Include data loading and validation steps.
4. Treat this as academic research code, not generic product scaffolding.

CONTEXT:
{request_context}
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
    parser.add_argument(
        "-i", "--interactive",
        action="store_true",
        help="Run in interactive step-by-step mode (requires TTY)",
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
    parallel.add_argument(
        "--role",
        type=str,
        help="Research team role (e.g., pi, methods-lead, statistician) mapped to roles/",
    )
    
    # Chain mode
    chain = subparsers.add_parser("chain", help="One generates, other verifies")
    chain.add_argument("--prompt", required=True, help="Generation prompt")
    chain.add_argument("--cwd", required=True, type=Path, help="Working directory")
    chain.add_argument("--generator", choices=["codex", "claude", "gemini"], default="codex")
    chain.add_argument(
        "--role",
        type=str,
        help="Research team role (e.g., pi, methods-lead, statistician) mapped to roles/",
    )
    
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
    code_build.add_argument("--topic", help="Research topic slug/name for strict Stage-I routing")
    code_build.add_argument(
        "--focus",
        default="implementation",
        help="Code-lane focus: implementation, reproduction, data_pipeline, code_specification, code_planning, execution_performance, code_review, reproducibility_audit, or full",
    )
    code_build.add_argument(
        "--paper-type",
        choices=["empirical", "qualitative", "systematic-review", "methods", "theory"],
        default="methods",
        help="Paper type used when code-build routes into strict Stage-I task flow",
    )
    code_build.add_argument(
        "--triad",
        action="store_true",
        help="Enable third independent audit for the final strict Stage-I review when --topic is set",
    )
    code_build.add_argument(
        "--only-target",
        action="append",
        dest="only_targets",
        help="Target a specific actionable item. Repeat for multiple items. Use STAGE_ID:TARGET when --focus full.",
    )

    task_run = subparsers.add_parser(
        "task-run",
        help="Run standardized Task-ID workflow with capability-map agent routing",
    )
    task_run.add_argument("--task-id", required=True, help="Canonical Task ID (e.g. F3)")
    task_run.add_argument(
        "--paper-type",
        required=True,
        choices=["empirical", "qualitative", "systematic-review", "methods", "theory"],
        help="Paper type from workflow contract",
    )
    task_run.add_argument("--topic", required=True, help="Research topic slug/name")
    task_run.add_argument("--cwd", required=True, type=Path, help="Working directory")
    task_run.add_argument(
        "--domain",
        default="auto",
        help="Optional domain profile to inject at runtime (for example econ, cs, psychology)",
    )
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
        "--role",
        type=str,
        help="Research team role (e.g., pi, methods-lead, statistician) mapped to roles/",
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
    task_run.add_argument(
        "--max-rounds",
        type=int,
        default=2,
        help="Maximum revision rounds when review returns BLOCK (default: 2, 0 = single-pass)",
    )
    task_run.add_argument(
        "--focus-output",
        action="append",
        dest="focus_outputs",
        help="Restrict this run to one contract output path. Repeat to focus multiple outputs.",
    )
    task_run.add_argument(
        "--output-budget",
        type=int,
        help="Maximum number of active contract outputs to execute in this run.",
    )
    task_run.add_argument(
        "--research-depth",
        choices=["standard", "deep"],
        default="standard",
        help="Execution depth for evidence expansion and review scrutiny (default: standard).",
    )
    task_run.add_argument(
        "--only-target",
        action="append",
        dest="only_targets",
        help="Target a specific actionable item from an existing Stage-I artifact. Repeat to focus multiple items.",
    )

    team_run_parser = subparsers.add_parser(
        "team-run",
        help="Fanout/fanin parallel execution for research tasks (MVP: B1, H3)",
    )
    team_run_parser.add_argument(
        "--task-id", required=True,
        help="Canonical Task ID (MVP: B1, H3)",
    )
    team_run_parser.add_argument(
        "--paper-type", required=True,
        choices=["empirical", "qualitative", "systematic-review", "methods", "theory"],
        help="Paper type from workflow contract",
    )
    team_run_parser.add_argument("--topic", required=True, help="Research topic slug/name")
    team_run_parser.add_argument("--cwd", required=True, type=Path, help="Working directory")
    team_run_parser.add_argument("--venue", help="Target venue (optional)")
    team_run_parser.add_argument("--context", help="Additional execution context (optional)")
    team_run_parser.add_argument(
        "--max-units", type=int,
        help="Override max_parallel_units from team_run_config",
    )
    team_run_parser.add_argument(
        "--mcp-strict", action="store_true",
        help="Fail if required MCP providers are unavailable",
    )
    team_run_parser.add_argument(
        "--skills-strict", action="store_true",
        help="Fail if required skill specs are missing",
    )
    team_run_parser.add_argument(
        "--profile-file", type=Path,
        help="JSON file defining user agent profiles",
    )
    team_run_parser.add_argument(
        "--profile", default="default",
        help="Base profile name (default: default)",
    )

    task_plan = subparsers.add_parser(
        "task-plan",
        help="Render dependency-based task plan from workflow contract",
    )
    task_plan.add_argument("--task-id", required=True, help="Canonical Task ID (e.g. F3)")
    task_plan.add_argument(
        "--paper-type",
        required=True,
        choices=["empirical", "qualitative", "systematic-review", "methods", "theory"],
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
    
    orchestrator = ModelOrchestrator(interactive=getattr(args, "interactive", False))
    
    if args.mode == "code-build":
        result = orchestrator.code_build(
            method=args.method,
            cwd=args.cwd,
            domain=args.domain,
            tier=args.tier,
            paper_path=args.paper,
            topic=getattr(args, "topic", None),
            focus=getattr(args, "focus", None),
            paper_type=getattr(args, "paper_type", "methods"),
            triad=getattr(args, "triad", False),
            only_targets=getattr(args, "only_targets", None),
        )
    elif args.mode == "doctor":
        result = orchestrator.doctor(cwd=args.cwd)
    elif args.mode == "task-run":
        result = orchestrator.task_run(
            task_id=args.task_id,
            paper_type=args.paper_type,
            topic=args.topic,
            cwd=args.cwd,
            domain=getattr(args, "domain", "auto"),
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
            max_revision_rounds=getattr(args, "max_rounds", 2),
            focus_outputs=getattr(args, "focus_outputs", None),
            output_budget=getattr(args, "output_budget", None),
            research_depth=getattr(args, "research_depth", "standard"),
            only_targets=getattr(args, "only_targets", None),
        )
    elif args.mode == "team-run":
        result = orchestrator.team_run(
            task_id=args.task_id,
            paper_type=args.paper_type,
            topic=args.topic,
            cwd=args.cwd,
            venue=getattr(args, "venue", None),
            context=getattr(args, "context", None),
            max_parallel_units=getattr(args, "max_units", None),
            mcp_strict=args.mcp_strict,
            skills_strict=args.skills_strict,
            profile_file=getattr(args, "profile_file", None),
            profile=getattr(args, "profile", "default"),
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
