#!/usr/bin/env python3
"""
Multi-Model Orchestrator for research-skills.
Single entry point for coordinating Codex and Gemini collaboration.

Usage:
    python orchestrator.py parallel --prompt "..." --cwd "/path"
    python orchestrator.py chain --prompt "..." --cwd "/path" --generator codex
    python orchestrator.py role --cwd "/path" --codex-task "..." --gemini-task "..."
    python orchestrator.py single --model codex --prompt "..." --cwd "/path"
    python orchestrator.py task-run --task-id F3 --paper-type empirical --topic ai-in-education --cwd "/path" --mcp-strict

Python 3.12+ required.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict
from enum import Enum
from pathlib import Path
from typing import Any

from .base_bridge import BridgeResponse, CollaborationResult, configure_stdio
from .codex_bridge import CodexBridge
from .gemini_bridge import GeminiBridge
from .mcp_connectors import MCPEvidence, MCPConnector


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
    RUNTIME_AGENTS = {"codex", "gemini"}
    DEFAULT_STANDARDS_DIR = Path(__file__).resolve().parents[1] / "standards"
    
    def __init__(
        self,
        codex_sandbox: str = "read-only",
        gemini_sandbox: bool = False,
        standards_dir: Path | None = None,
        mcp_timeout_seconds: int = 20,
    ):
        """Initialize orchestrator with bridges."""
        self.codex = CodexBridge(sandbox=codex_sandbox)
        self.gemini = GeminiBridge(sandbox=gemini_sandbox)
        self.standards_dir = standards_dir or self.DEFAULT_STANDARDS_DIR
        self.mcp_connector = MCPConnector(timeout_seconds=mcp_timeout_seconds)
    
    def execute(
        self,
        mode: CollaborationMode,
        cwd: Path,
        prompt: str | None = None,
        codex_task: str | None = None,
        gemini_task: str | None = None,
        generator: str = "codex",
        single_model: str = "codex",
        session_id: str | None = None,
    ) -> CollaborationResult:
        """
        Execute collaboration based on mode.
        
        Args:
            mode: Collaboration mode
            cwd: Working directory
            prompt: Main prompt (for parallel/chain/single modes)
            codex_task: Codex-specific task (for role mode)
            gemini_task: Gemini-specific task (for role mode)
            generator: Which model generates in chain mode
            single_model: Which model to use in single mode
            session_id: Resume existing session
        """
        if mode == CollaborationMode.PARALLEL:
            return self._parallel_analyze(prompt or "", cwd)
        elif mode == CollaborationMode.CHAIN:
            return self._chain_verify(prompt or "", cwd, generator)
        elif mode == CollaborationMode.ROLE_BASED:
            return self._role_based(cwd, codex_task, gemini_task)
        elif mode == CollaborationMode.SINGLE:
            return self._single_execute(
                prompt or "", cwd, single_model, session_id
            )
        else:
            raise ValueError(f"Unknown collaboration mode: {mode}")
    
    def _parallel_analyze(self, prompt: str, cwd: Path) -> CollaborationResult:
        """
        Parallel mode: Both models analyze simultaneously.
        Merge results, identify agreements as high-confidence.
        
        Best for: Code review, security audit, bug analysis
        """
        codex_resp = self.codex.execute(prompt, cwd)
        gemini_resp = self.gemini.execute(prompt, cwd)
        
        # Calculate confidence based on both responses
        if codex_resp.success and gemini_resp.success:
            merged = self._merge_analyses(codex_resp.content, gemini_resp.content)
            confidence = self._calculate_agreement(codex_resp, gemini_resp)
        elif codex_resp.success:
            merged = f"[Codex Only]\n{codex_resp.content}"
            confidence = 0.6
        elif gemini_resp.success:
            merged = f"[Gemini Only]\n{gemini_resp.content}"
            confidence = 0.6
        else:
            merged = "Both models failed to produce output."
            confidence = 0.0
        
        return CollaborationResult(
            mode="parallel",
            task_description=prompt[:200],
            codex_response=codex_resp,
            gemini_response=gemini_resp,
            merged_analysis=merged,
            confidence=confidence,
            recommendations=self._extract_recommendations(merged),
        )
    
    def _chain_verify(
        self, prompt: str, cwd: Path, generator: str = "codex"
    ) -> CollaborationResult:
        """
        Chain mode: One model generates, another verifies.
        
        Best for: Algorithm implementation, code generation
        """
        if generator == "codex":
            # Codex generates, Gemini verifies
            gen_resp = self.codex.execute(prompt, cwd)
            if gen_resp.success:
                verify_prompt = self._build_verification_prompt(gen_resp.content)
                verify_resp = self.gemini.execute(verify_prompt, cwd)
            else:
                verify_resp = None
        else:
            # Gemini generates, Codex verifies
            gen_resp = self.gemini.execute(prompt, cwd)
            if gen_resp.success:
                verify_prompt = self._build_verification_prompt(gen_resp.content)
                verify_resp = self.codex.execute(verify_prompt, cwd)
            else:
                verify_resp = None
        
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
            codex_response=gen_resp if generator == "codex" else verify_resp,
            gemini_response=verify_resp if generator == "codex" else gen_resp,
            merged_analysis=merged,
            confidence=confidence,
            recommendations=self._extract_recommendations(merged),
        )
    
    def _role_based(
        self,
        cwd: Path,
        codex_task: str | None,
        gemini_task: str | None,
    ) -> CollaborationResult:
        """
        Role-based mode: Divide tasks by model specialty.
        
        Codex: Code generation, implementation, fixing
        Gemini: Explanation, documentation, analysis
        """
        codex_resp = None
        gemini_resp = None
        
        if codex_task:
            codex_resp = self.codex.execute(codex_task, cwd)
        
        if gemini_task:
            gemini_resp = self.gemini.execute(gemini_task, cwd)
        
        # Merge outputs
        parts = []
        if codex_resp and codex_resp.success:
            parts.append(f"## Codex Output\n\n{codex_resp.content}")
        if gemini_resp and gemini_resp.success:
            parts.append(f"## Gemini Output\n\n{gemini_resp.content}")
        
        merged = "\n\n---\n\n".join(parts) if parts else "No successful outputs."
        
        # Calculate confidence
        success_count = sum([
            1 for r in [codex_resp, gemini_resp]
            if r and r.success
        ])
        confidence = success_count / 2.0 if (codex_task or gemini_task) else 0.0
        
        task_desc = f"Codex: {codex_task or 'N/A'} | Gemini: {gemini_task or 'N/A'}"
        
        return CollaborationResult(
            mode="role_based",
            task_description=task_desc[:200],
            codex_response=codex_resp,
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
        else:
            resp = self.gemini.execute(prompt, cwd, session_id=session_id)
            return CollaborationResult(
                mode="single",
                task_description=prompt[:200],
                gemini_response=resp,
                merged_analysis=resp.content if resp.success else resp.error or "",
                confidence=1.0 if resp.success else 0.0,
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
            "primary_agent": primary_agent,
            "review_agent": review_agent,
            "fallback_agent": fallback_agent,
            "quality_gates": quality_gates,
        }

    def _resolve_runtime_agent(
        self,
        preferred_agent: str,
        fallback_chain: list[str],
        exclude_agent: str | None = None,
    ) -> tuple[str, list[str]]:
        notes: list[str] = []
        seen: set[str] = set()
        candidates = [preferred_agent, *fallback_chain, "codex", "gemini"]
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

    def _execute_runtime_agent(self, agent_name: str, prompt: str, cwd: Path) -> BridgeResponse:
        if agent_name == "codex":
            return self.codex.execute(prompt, cwd)
        if agent_name == "gemini":
            return self.gemini.execute(prompt, cwd)
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
            "quality_gates": quality_gates,
        }

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
5. Explicitly check quality_gates and mark each as PASS/WARN/FAIL.
6. If required input is missing, list it under "Missing Inputs" and continue with placeholders.

MCP evidence snapshot:
{self._format_mcp_evidence(mcp_evidence)}

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
        draft_output: str,
    ) -> str:
        return f"""Review the draft for this canonical research workflow task.

Task packet (JSON):
{json.dumps(task_packet, ensure_ascii=False, indent=2)}

Primary draft:
{draft_output}

MCP evidence snapshot:
{self._format_mcp_evidence(mcp_evidence)}

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

    def task_run(
        self,
        task_id: str,
        paper_type: str,
        topic: str,
        cwd: Path,
        venue: str | None = None,
        context: str | None = None,
        mcp_strict: bool = False,
    ) -> CollaborationResult:
        """Run task-level orchestration using capability map and contract."""
        normalized_task = task_id.strip().upper()
        normalized_topic = self._normalize_topic(topic)
        routing_notes: list[str] = []

        try:
            agent_plan = self._load_task_agent_plan(normalized_task)
            artifact_root, required_outputs = self._load_task_outputs(normalized_task)
        except (FileNotFoundError, ValueError) as exc:
            error_resp = BridgeResponse.from_error("orchestrator", str(exc))
            return CollaborationResult(
                mode="task-run",
                task_description=f"{normalized_task} {paper_type} {normalized_topic}"[:200],
                merged_analysis=str(exc),
                confidence=0.0,
                recommendations=[],
                codex_response=error_resp if "codex" in str(exc).lower() else None,
                gemini_response=error_resp if "gemini" in str(exc).lower() else None,
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
            quality_gates=agent_plan["quality_gates"],
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
                codex_response=error_resp,
            )

        primary_runtime, primary_notes = self._resolve_runtime_agent(
            preferred_agent=agent_plan["primary_agent"],
            fallback_chain=[agent_plan["fallback_agent"]],
        )
        routing_notes.extend(primary_notes)

        draft_prompt = self._build_task_draft_prompt(packet, mcp_evidence, context)
        draft_resp = self._execute_runtime_agent(primary_runtime, draft_prompt, cwd)
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
                fallback_resp = self._execute_runtime_agent(fallback_runtime, draft_prompt, cwd)
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
                draft_resp.content,
            )
            review_resp = self._execute_runtime_agent(review_runtime, review_prompt, cwd)
        else:
            routing_notes.append("Skipped independent review because draft generation failed.")

        codex_resp = None
        gemini_resp = None
        for runtime_agent, response in (
            (draft_runtime, draft_resp),
            (review_runtime, review_resp),
            (fallback_runtime, fallback_resp),
        ):
            if not response:
                continue
            if runtime_agent == "codex" and codex_resp is None:
                codex_resp = response
            if runtime_agent == "gemini" and gemini_resp is None:
                gemini_resp = response

        merged_parts = [
            "## Task Packet",
            json.dumps(packet, ensure_ascii=False, indent=2),
            "",
            "## MCP Evidence",
            self._format_mcp_evidence(mcp_evidence),
            "",
            "## Routing",
            "\n".join(f"- {item}" for item in routing_notes) if routing_notes else "- Direct mapping used.",
            "",
            f"## Draft ({draft_runtime})",
            draft_resp.content if draft_resp.success else f"[FAILED] {draft_resp.error}",
        ]
        if review_resp:
            merged_parts.extend(
                [
                    "",
                    f"## Review ({review_runtime})",
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
        merged = "\n".join(merged_parts)

        if draft_resp.success and review_resp and review_resp.success and review_runtime != draft_runtime:
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
    parallel = subparsers.add_parser("parallel", help="Both models analyze simultaneously")
    parallel.add_argument("--prompt", required=True, help="Analysis prompt")
    parallel.add_argument("--cwd", required=True, type=Path, help="Working directory")
    
    # Chain mode
    chain = subparsers.add_parser("chain", help="One generates, other verifies")
    chain.add_argument("--prompt", required=True, help="Generation prompt")
    chain.add_argument("--cwd", required=True, type=Path, help="Working directory")
    chain.add_argument("--generator", choices=["codex", "gemini"], default="codex")
    
    # Role-based mode
    role = subparsers.add_parser("role", help="Task division by specialty")
    role.add_argument("--cwd", required=True, type=Path, help="Working directory")
    role.add_argument("--codex-task", help="Task for Codex")
    role.add_argument("--gemini-task", help="Task for Gemini")
    
    # Single mode
    single = subparsers.add_parser("single", help="Single model execution")
    single.add_argument("--prompt", required=True, help="Task prompt")
    single.add_argument("--cwd", required=True, type=Path, help="Working directory")
    single.add_argument("--model", choices=["codex", "gemini"], default="codex")
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
    elif args.mode == "task-run":
        result = orchestrator.task_run(
            task_id=args.task_id,
            paper_type=args.paper_type,
            topic=args.topic,
            cwd=args.cwd,
            venue=args.venue,
            context=args.context,
            mcp_strict=args.mcp_strict,
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
            gemini_task=getattr(args, "gemini_task", None),
            generator=getattr(args, "generator", "codex"),
            single_model=getattr(args, "model", "codex"),
            session_id=getattr(args, "session_id", None),
        )
    
    print(result.to_json())

if __name__ == "__main__":
    main()
