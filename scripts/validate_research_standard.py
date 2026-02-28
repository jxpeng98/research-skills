#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

EXPECTED_PAPER_TYPES = {"empirical", "systematic-review", "methods", "theory"}
EXPECTED_STAGE_IDS = {stage for stage in "ABCDEFGHI"}
EXPECTED_TASK_IDS = {
    "A1", "A1_5", "A2", "A3", "A4", "A5",
    "B1", "B1_5", "B2", "B3", "B4", "B5", "B6",
    "C1", "C1_5", "C2", "C3", "C3_5", "C4", "C5",
    "D1", "D2", "D3",
    "E1", "E2", "E3", "E3_5", "E4", "E5",
    "F1", "F2", "F3", "F4", "F5", "F6",
    "G1", "G2", "G3", "G4",
    "H1", "H2", "H2_5",
    "I1", "I2", "I3", "I4",
}
EXPECTED_QUALITY_GATES = {"Q1", "Q2", "Q3", "Q4"}
EXPECTED_AGENTS = {"codex", "claude", "gemini"}
EXPECTED_PACKET_FIELDS = {
    "task_id",
    "paper_type",
    "topic",
    "required_outputs",
    "required_skills",
    "required_skill_cards",
    "quality_gates",
}
EXPECTED_EXECUTION_CHAIN = [
    "plan",
    "mcp-evidence",
    "primary-agent-draft",
    "review-agent-check",
    "validator-gate",
]
WORKFLOW_TASK_EXPECTATIONS = {
    ".agent/workflows/paper.md": EXPECTED_TASK_IDS,
    ".agent/workflows/academic-write.md": {"F2"},
    ".agent/workflows/build-framework.md": {"A3"},
    ".agent/workflows/code-build.md": {"I1", "I2", "I3"},
    ".agent/workflows/ethics-check.md": {"D1"},
    ".agent/workflows/find-gap.md": {"A4"},
    ".agent/workflows/lit-review.md": {"B1"},
    ".agent/workflows/paper-read.md": {"B2"},
    ".agent/workflows/paper-write.md": {"F1", "F3", "F4", "F5"},
    ".agent/workflows/rebuttal.md": {"H2"},
    ".agent/workflows/study-design.md": {"C1", "C2", "C3", "C4", "C5"},
    ".agent/workflows/submission-prep.md": {"H1"},
    ".agent/workflows/synthesize.md": {"E1", "E2", "E3", "E4", "E5"},
}
RELEASE_NOTE_FILE_PATTERN = re.compile(r"^v(\d+)\.(\d+)\.(\d+)-beta\.(\d+)\.md$")


@dataclass
class ValidationReport:
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)
    passed: int = 0

    def check(self, condition: bool, pass_msg: str, fail_msg: str) -> None:
        if condition:
            self.passed += 1
            print(f"[PASS] {pass_msg}")
            return
        self.errors.append(fail_msg)
        print(f"[FAIL] {fail_msg}")

    def warn(self, condition: bool, pass_msg: str, warn_msg: str) -> None:
        if condition:
            self.passed += 1
            print(f"[PASS] {pass_msg}")
            return
        self.warnings.append(warn_msg)
        print(f"[WARN] {warn_msg}")


def read_text(root: Path, relative_path: str, report: ValidationReport) -> str:
    target = root / relative_path
    if not target.exists():
        report.errors.append(f"Missing file: {relative_path}")
        print(f"[FAIL] Missing file: {relative_path}")
        return ""
    try:
        content = target.read_text(encoding="utf-8")
    except OSError as exc:
        report.errors.append(f"Failed to read {relative_path}: {exc}")
        print(f"[FAIL] Failed to read {relative_path}: {exc}")
        return ""
    report.passed += 1
    print(f"[PASS] Read file: {relative_path}")
    return content


def extract_top_level_section(content: str, key: str) -> str:
    match = re.search(rf"^{re.escape(key)}:\s*\n", content, flags=re.MULTILINE)
    if not match:
        return ""
    start = match.end()
    tail = content[start:]
    next_key = re.search(r"^[A-Za-z0-9_]+:\s*", tail, flags=re.MULTILINE)
    if not next_key:
        return tail
    return tail[: next_key.start()]


def extract_nested_section(block: str, key: str, indent: int) -> str:
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


def parse_yaml_scalar(block: str, key: str, indent: int) -> str:
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


def parse_yaml_list(section: str, item_indent: int) -> list[str]:
    values = []
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


def select_latest_release_note(root: Path) -> Path | None:
    release_dir = root / "release"
    if not release_dir.exists():
        return None
    candidates: list[tuple[tuple[int, int, int, int], Path]] = []
    for path in release_dir.iterdir():
        if not path.is_file():
            continue
        match = RELEASE_NOTE_FILE_PATTERN.match(path.name)
        if not match:
            continue
        version = tuple(int(match.group(index)) for index in range(1, 5))
        candidates.append((version, path))
    if not candidates:
        return None
    candidates.sort(key=lambda item: item[0], reverse=True)
    return candidates[0][1]


def parse_frontmatter(content: str) -> tuple[dict[str, str], str]:
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


def ids_to_text(values: set[str]) -> str:
    return ", ".join(sorted(values))


def validate_contract(root: Path, report: ValidationReport) -> None:
    content = read_text(root, "standards/research-workflow-contract.yaml", report)
    if not content:
        return

    contract_version = re.search(
        r'^contract_version:\s*"([^"]+)"\s*$',
        content,
        flags=re.MULTILINE,
    )
    report.check(
        contract_version is not None,
        "Contract version exists",
        "Contract version missing in standards/research-workflow-contract.yaml",
    )
    if contract_version is not None:
        report.warn(
            contract_version.group(1) == "1.0.0",
            "Contract version is 1.0.0",
            f"Contract version is {contract_version.group(1)} (expected 1.0.0)",
        )

    paper_section = extract_top_level_section(content, "paper_types")
    paper_types = {
        found
        for found in re.findall(
            r'^\s*-\s*id:\s*"([a-z-]+)"\s*$',
            paper_section,
            flags=re.MULTILINE,
        )
    }
    report.check(
        paper_types == EXPECTED_PAPER_TYPES,
        "Paper type set matches standard",
        (
            "Paper types mismatch. Missing: "
            f"{ids_to_text(EXPECTED_PAPER_TYPES - paper_types) or 'none'}; "
            f"Extra: {ids_to_text(paper_types - EXPECTED_PAPER_TYPES) or 'none'}"
        ),
    )

    stages_section = extract_top_level_section(content, "stages")
    stage_ids = {
        found
        for found in re.findall(
            r'^\s*-\s*id:\s*"([A-I])"\s*$',
            stages_section,
            flags=re.MULTILINE,
        )
    }
    report.check(
        stage_ids == EXPECTED_STAGE_IDS,
        "Stage set A-I matches standard",
        (
            "Stages mismatch. Missing: "
            f"{ids_to_text(EXPECTED_STAGE_IDS - stage_ids) or 'none'}; "
            f"Extra: {ids_to_text(stage_ids - EXPECTED_STAGE_IDS) or 'none'}"
        ),
    )

    task_section = extract_top_level_section(content, "task_catalog")
    task_ids = {
        found
        for found in re.findall(
            r"^\s{2}([A-I][0-9_]+):\s*$",
            task_section,
            flags=re.MULTILINE,
        )
    }
    report.check(
        task_ids == EXPECTED_TASK_IDS,
        "Task ID catalog matches standard",
        (
            "Task IDs mismatch. Missing: "
            f"{ids_to_text(EXPECTED_TASK_IDS - task_ids) or 'none'}; "
            f"Extra: {ids_to_text(task_ids - EXPECTED_TASK_IDS) or 'none'}"
        ),
    )

    for task_id, block in re.findall(
        r"^\s{2}([A-I][0-9_]+):\n((?:^\s{4}.*\n?)+)",
        task_section,
        flags=re.MULTILINE,
    ):
        stage_match = re.search(
            r'^\s{4}stage:\s*"([A-I])"\s*$',
            block,
            flags=re.MULTILINE,
        )
        report.check(
            stage_match is not None,
            f"{task_id} has stage field",
            f"{task_id} missing stage field",
        )
        if stage_match is not None:
            report.check(
                stage_match.group(1) == task_id[0],
                f"{task_id} stage prefix matches",
                f"{task_id} stage {stage_match.group(1)} mismatches task prefix {task_id[0]}",
            )
        has_outputs = bool(re.search(r"^\s{4}outputs:\s*$", block, flags=re.MULTILINE))
        has_output_items = bool(
            re.search(r'^\s{6}-\s*"[^"]+"\s*$', block, flags=re.MULTILINE)
        )
        report.check(
            has_outputs and has_output_items,
            f"{task_id} has non-empty outputs",
            f"{task_id} missing outputs or output paths",
        )

    quality_section = extract_top_level_section(content, "quality_gates")
    quality_ids = {
        found
        for found in re.findall(
            r'^\s*-\s*id:\s*"([A-Z][0-9]+)"\s*$',
            quality_section,
            flags=re.MULTILINE,
        )
    }
    report.check(
        quality_ids == EXPECTED_QUALITY_GATES,
        "Quality gates set matches standard",
        (
            "Quality gate mismatch. Missing: "
            f"{ids_to_text(EXPECTED_QUALITY_GATES - quality_ids) or 'none'}; "
            f"Extra: {ids_to_text(quality_ids - EXPECTED_QUALITY_GATES) or 'none'}"
        ),
    )


def validate_portable_skill(root: Path, report: ValidationReport) -> None:
    skill_content = read_text(root, "research-paper-workflow/SKILL.md", report)
    if skill_content:
        frontmatter, _body = parse_frontmatter(skill_content)
        key_set = set(frontmatter.keys())
        report.check(
            key_set == {"name", "description"},
            "SKILL.md frontmatter keys are name/description only",
            f"SKILL.md frontmatter keys must be name+description only; found: {ids_to_text(key_set)}",
        )
        report.check(
            frontmatter.get("name") == "research-paper-workflow",
            "SKILL.md name matches folder",
            (
                "SKILL.md name must be research-paper-workflow; found: "
                f"{frontmatter.get('name', '<missing>')}"
            ),
        )
        for reference_path in (
            "references/workflow-contract.md",
            "references/platform-routing.md",
            "references/coverage-matrix.md",
        ):
            report.check(
                reference_path in skill_content,
                f"SKILL.md references {reference_path}",
                f"SKILL.md missing reference link: {reference_path}",
            )
            target = root / "research-paper-workflow" / reference_path
            report.check(
                target.exists(),
                f"{reference_path} exists",
                f"Missing file: research-paper-workflow/{reference_path}",
            )

    yaml_content = read_text(root, "research-paper-workflow/agents/openai.yaml", report)
    if yaml_content:
        required_keys = (
            "interface:",
            "display_name:",
            "short_description:",
            "default_prompt:",
        )
        for key in required_keys:
            report.check(
                key in yaml_content,
                f"openai.yaml includes {key.strip(':')}",
                f"openai.yaml missing required key: {key.strip(':')}",
            )
        prompt = re.search(r'^\s*default_prompt:\s*"([^"]+)"\s*$', yaml_content, re.MULTILINE)
        report.check(
            prompt is not None,
            "openai.yaml default_prompt is quoted",
            "openai.yaml default_prompt missing or not quoted",
        )
        if prompt is not None:
            report.check(
                "$research-paper-workflow" in prompt.group(1),
                "openai.yaml default_prompt invokes $research-paper-workflow",
                "openai.yaml default_prompt must include $research-paper-workflow",
            )


def validate_mcp_agent_map(root: Path, report: ValidationReport) -> None:
    map_content = read_text(root, "standards/mcp-agent-capability-map.yaml", report)
    if not map_content:
        return

    map_version = re.search(
        r'^map_version:\s*"([^"]+)"\s*$',
        map_content,
        flags=re.MULTILINE,
    )
    report.check(
        map_version is not None,
        "Capability map version exists",
        "Map version missing in standards/mcp-agent-capability-map.yaml",
    )
    if map_version is not None:
        report.warn(
            map_version.group(1) == "1.0.0",
            "Capability map version is 1.0.0",
            f"Capability map version is {map_version.group(1)} (expected 1.0.0)",
        )

    coordination_section = extract_top_level_section(map_content, "coordination_contract")
    packet_section = extract_nested_section(
        coordination_section,
        "required_packet_fields",
        indent=2,
    )
    packet_fields = set(parse_yaml_list(packet_section, item_indent=4))
    report.check(
        packet_fields == EXPECTED_PACKET_FIELDS,
        "Coordination required_packet_fields match standard",
        (
            "Coordination packet fields mismatch. Missing: "
            f"{ids_to_text(EXPECTED_PACKET_FIELDS - packet_fields) or 'none'}; "
            f"Extra: {ids_to_text(packet_fields - EXPECTED_PACKET_FIELDS) or 'none'}"
        ),
    )
    chain_section = extract_nested_section(
        coordination_section,
        "fixed_execution_chain",
        indent=2,
    )
    execution_chain = parse_yaml_list(chain_section, item_indent=4)
    report.check(
        execution_chain == EXPECTED_EXECUTION_CHAIN,
        "Coordination fixed_execution_chain matches standard",
        (
            "Execution chain mismatch. Expected: "
            f"{', '.join(EXPECTED_EXECUTION_CHAIN)}; "
            f"Found: {', '.join(execution_chain)}"
        ),
    )

    mcp_section = extract_top_level_section(map_content, "mcp_registry")
    mcp_list = parse_yaml_list(mcp_section, item_indent=2)
    mcp_registry = set(mcp_list)
    report.check(
        len(mcp_registry) > 0,
        "MCP registry is non-empty",
        "MCP registry must include at least one MCP capability",
    )
    report.check(
        len(mcp_registry) == len(mcp_list),
        "MCP registry has unique entries",
        "MCP registry has duplicate entries",
    )
    report.check(
        "filesystem" in mcp_registry,
        "MCP registry includes filesystem",
        "MCP registry must include filesystem",
    )

    agent_section = extract_top_level_section(map_content, "agent_registry")
    agent_list = parse_yaml_list(agent_section, item_indent=2)
    agent_registry = set(agent_list)
    report.check(
        agent_registry.issuperset(EXPECTED_AGENTS),
        "Agent registry includes codex/claude/gemini",
        (
            "Agent registry missing required agents: "
            f"{ids_to_text(EXPECTED_AGENTS - agent_registry) or 'none'}"
        ),
    )
    report.check(
        len(agent_registry) == len(agent_list),
        "Agent registry has unique entries",
        "Agent registry has duplicate entries",
    )

    skill_section = extract_top_level_section(map_content, "skill_registry")
    skill_list = parse_yaml_list(skill_section, item_indent=2)
    skill_registry = set(skill_list)
    report.check(
        len(skill_registry) > 0,
        "Skill registry is non-empty",
        "Skill registry must include at least one skill",
    )
    report.check(
        len(skill_registry) == len(skill_list),
        "Skill registry has unique entries",
        "Skill registry has duplicate entries",
    )
    for skill_name in sorted(skill_registry):
        report.check(
            (root / "skills" / f"{skill_name}.md").exists(),
            f"Skill file exists for {skill_name}",
            f"Missing skill file for registry entry: skills/{skill_name}.md",
        )

    skill_catalog_section = extract_top_level_section(map_content, "skill_catalog")
    catalog_ids = {
        found
        for found in re.findall(
            r"^\s{2}([a-z0-9-]+):\s*$",
            skill_catalog_section,
            flags=re.MULTILINE,
        )
    }
    report.check(
        catalog_ids == skill_registry,
        "Skill catalog covers all registered skills",
        (
            "Skill catalog mismatch. Missing: "
            f"{ids_to_text(skill_registry - catalog_ids) or 'none'}; "
            f"Extra: {ids_to_text(catalog_ids - skill_registry) or 'none'}"
        ),
    )
    for skill_name, block in re.findall(
        r"^\s{2}([a-z0-9-]+):\n((?:^\s{4}.*\n?)+)",
        skill_catalog_section,
        flags=re.MULTILINE,
    ):
        skill_file = parse_yaml_scalar(block, "file", indent=4)
        category = parse_yaml_scalar(block, "category", indent=4)
        focus = parse_yaml_scalar(block, "focus", indent=4)
        default_outputs = parse_yaml_list(
            extract_nested_section(block, "default_outputs", indent=4),
            item_indent=6,
        )
        report.check(
            bool(skill_file),
            f"Skill catalog entry has file: {skill_name}",
            f"Skill catalog entry missing file: {skill_name}",
        )
        if skill_file:
            report.check(
                skill_file.startswith("skills/"),
                f"Skill catalog file uses skills/ prefix: {skill_name}",
                f"Skill catalog file should be under skills/: {skill_name} -> {skill_file}",
            )
            report.check(
                (root / skill_file).exists(),
                f"Skill catalog file exists: {skill_name}",
                f"Skill catalog file path missing: {skill_name} -> {skill_file}",
            )
        report.check(
            bool(category),
            f"Skill catalog entry has category: {skill_name}",
            f"Skill catalog entry missing category: {skill_name}",
        )
        report.check(
            bool(focus),
            f"Skill catalog entry has focus: {skill_name}",
            f"Skill catalog entry missing focus: {skill_name}",
        )
        report.check(
            len(default_outputs) > 0,
            f"Skill catalog entry has default_outputs: {skill_name}",
            f"Skill catalog entry missing default_outputs: {skill_name}",
        )

    skill_map_section = extract_top_level_section(map_content, "task_skill_mapping")
    skill_map_ids = {
        found
        for found in re.findall(
            r"^\s{2}([A-I][0-9_]+):\s*$",
            skill_map_section,
            flags=re.MULTILINE,
        )
    }
    report.check(
        skill_map_ids == EXPECTED_TASK_IDS,
        "Task skill mapping covers all canonical Task IDs",
        (
            "Task skill mapping mismatch. Missing: "
            f"{ids_to_text(EXPECTED_TASK_IDS - skill_map_ids) or 'none'}; "
            f"Extra: {ids_to_text(skill_map_ids - EXPECTED_TASK_IDS) or 'none'}"
        ),
    )
    for task_id, block in re.findall(
        r"^\s{2}([A-I][0-9_]+):\n((?:^\s{4}.*\n?)+)",
        skill_map_section,
        flags=re.MULTILINE,
    ):
        required_skills_section = extract_nested_section(block, "required_skills", indent=4)
        required_skills = set(parse_yaml_list(required_skills_section, item_indent=6))
        report.check(
            len(required_skills) > 0,
            f"{task_id} has required_skills list",
            f"{task_id} missing required_skills entries",
        )
        report.check(
            required_skills.issubset(skill_registry),
            f"{task_id} required_skills are registered",
            (
                f"{task_id} references unknown skills: "
                f"{ids_to_text(required_skills - skill_registry) or 'none'}"
            ),
        )

    task_section = extract_top_level_section(map_content, "task_execution")
    task_ids = {
        found
        for found in re.findall(
            r"^\s{2}([A-I][0-9_]+):\s*$",
            task_section,
            flags=re.MULTILINE,
        )
    }
    report.check(
        task_ids == EXPECTED_TASK_IDS,
        "Capability map covers all canonical Task IDs",
        (
            "Capability map task coverage mismatch. Missing: "
            f"{ids_to_text(EXPECTED_TASK_IDS - task_ids) or 'none'}; "
            f"Extra: {ids_to_text(task_ids - EXPECTED_TASK_IDS) or 'none'}"
        ),
    )

    for task_id, block in re.findall(
        r"^\s{2}([A-I][0-9_]+):\n((?:^\s{4}.*\n?)+)",
        task_section,
        flags=re.MULTILINE,
    ):
        required_mcp_section = extract_nested_section(block, "required_mcp", indent=4)
        required_mcp = set(parse_yaml_list(required_mcp_section, item_indent=6))
        report.check(
            len(required_mcp) > 0,
            f"{task_id} has required_mcp list",
            f"{task_id} missing required_mcp entries",
        )
        report.check(
            required_mcp.issubset(mcp_registry),
            f"{task_id} required_mcp are registered",
            (
                f"{task_id} references unknown MCP capabilities: "
                f"{ids_to_text(required_mcp - mcp_registry) or 'none'}"
            ),
        )

        for field_name in ("primary_agent", "review_agent", "fallback_agent"):
            value = parse_yaml_scalar(block, field_name, indent=4)
            report.check(
                bool(value),
                f"{task_id} has {field_name}",
                f"{task_id} missing {field_name}",
            )
            if value:
                report.check(
                    value in agent_registry,
                    f"{task_id} {field_name} is registered",
                    f"{task_id} {field_name} must be in agent_registry: {value}",
                )

        primary_agent = parse_yaml_scalar(block, "primary_agent", indent=4)
        review_agent = parse_yaml_scalar(block, "review_agent", indent=4)
        report.warn(
            bool(primary_agent and review_agent and primary_agent != review_agent),
            f"{task_id} primary/review agents are separated",
            f"{task_id} uses same primary/review agent; prefer independent review",
        )

        quality_section = extract_nested_section(block, "quality_gates", indent=4)
        quality_gates = set(parse_yaml_list(quality_section, item_indent=6))
        report.check(
            len(quality_gates) > 0,
            f"{task_id} has quality_gates list",
            f"{task_id} missing quality_gates entries",
        )
        report.check(
            quality_gates.issubset(EXPECTED_QUALITY_GATES),
            f"{task_id} quality_gates are canonical",
            (
                f"{task_id} has unknown quality gates: "
                f"{ids_to_text(quality_gates - EXPECTED_QUALITY_GATES) or 'none'}"
            ),
        )


def validate_cross_file_consistency(root: Path, report: ValidationReport) -> None:
    markdown_contract = read_text(
        root,
        "research-paper-workflow/references/workflow-contract.md",
        report,
    )
    if markdown_contract:
        markdown_task_ids = {
            found
            for found in re.findall(
                r"^\|\s*`([A-I][0-9_]+)`\s*\|",
                markdown_contract,
                flags=re.MULTILINE,
            )
        }
        report.check(
            markdown_task_ids == EXPECTED_TASK_IDS,
            "Reference workflow-contract.md task table is complete",
            (
                "workflow-contract.md task table mismatch. Missing: "
                f"{ids_to_text(EXPECTED_TASK_IDS - markdown_task_ids) or 'none'}; "
                f"Extra: {ids_to_text(markdown_task_ids - EXPECTED_TASK_IDS) or 'none'}"
            ),
        )

    paper_workflow_content = read_text(root, ".agent/workflows/paper.md", report)
    if paper_workflow_content:
        paper_ids = {
            found for found in re.findall(r"\*\*([A-I][0-9_]+)\b", paper_workflow_content)
        }
        report.check(
            paper_ids == EXPECTED_TASK_IDS,
            "paper.md menu covers all canonical Task IDs",
            (
                "paper.md Task ID coverage mismatch. Missing: "
                f"{ids_to_text(EXPECTED_TASK_IDS - paper_ids) or 'none'}; "
                f"Extra: {ids_to_text(paper_ids - EXPECTED_TASK_IDS) or 'none'}"
            ),
        )
        report.check(
            "quality gate applies (`Q1`-`Q4`)" in paper_workflow_content,
            "paper.md includes quality gate check",
            "paper.md missing Q1-Q4 quality gate check step",
        )

    routing_content = read_text(
        root,
        "research-paper-workflow/references/platform-routing.md",
        report,
    )
    if routing_content:
        for token in (
            "## Claude Code",
            "## Codex",
            "## Gemini",
            "$research-paper-workflow",
            "Task {ID}",
        ):
            report.check(
                token in routing_content,
                f"platform-routing.md includes {token}",
                f"platform-routing.md missing: {token}",
            )

    matrix_content = read_text(
        root,
        "research-paper-workflow/references/coverage-matrix.md",
        report,
    )
    if matrix_content:
        for token in ("Empirical", "Systematic Review", "Methods", "Theory"):
            report.check(
                token in matrix_content,
                f"coverage-matrix.md includes {token}",
                f"coverage-matrix.md missing {token} column",
            )
        capability_rows = [
            row
            for row in matrix_content.splitlines()
            if row.startswith("| ") and row.count("|") >= 5 and not row.startswith("|---")
        ]
        report.warn(
            len(capability_rows) >= 8,
            "coverage-matrix.md has broad capability rows",
            "coverage-matrix.md appears too short; expected at least 8 capability rows",
        )

    for relative_path, expected_ids in WORKFLOW_TASK_EXPECTATIONS.items():
        content = read_text(root, relative_path, report)
        if not content:
            continue
        report.check(
            "standards/research-workflow-contract.yaml" in content,
            f"{relative_path} references canonical contract",
            f"{relative_path} missing canonical contract reference",
        )
        for task_id in sorted(expected_ids):
            report.check(
                re.search(rf"\b{task_id}\b", content) is not None,
                f"{relative_path} includes {task_id}",
                f"{relative_path} missing task ID {task_id}",
            )


def validate_docs(root: Path, report: ValidationReport) -> None:
    for relative_path in ("README.md", "README_CN.md", "CLAUDE.md"):
        content = read_text(root, relative_path, report)
        if not content:
            continue
        report.check(
            "standards/research-workflow-contract.yaml" in content,
            f"{relative_path} mentions canonical contract",
            f"{relative_path} missing canonical contract mention",
        )
        report.check(
            "standards/mcp-agent-capability-map.yaml" in content,
            f"{relative_path} mentions MCP-agent map",
            f"{relative_path} should mention standards/mcp-agent-capability-map.yaml",
        )
        report.warn(
            "scripts/validate_research_standard.py" in content,
            f"{relative_path} includes validator command",
            f"{relative_path} should document scripts/validate_research_standard.py",
        )
        report.warn(
            "task-run" in content,
            f"{relative_path} mentions task-run orchestration",
            f"{relative_path} should mention task-run for capability-map orchestration",
        )
        report.warn(
            "guides/agent-skill-collaboration.md" in content,
            f"{relative_path} references collaboration guide",
            f"{relative_path} should reference guides/agent-skill-collaboration.md",
        )
        report.warn(
            "scripts/install_research_skill.sh" in content,
            f"{relative_path} includes installer command",
            f"{relative_path} should document scripts/install_research_skill.sh",
        )


def validate_orchestrator(root: Path, report: ValidationReport) -> None:
    content = read_text(root, "bridges/orchestrator.py", report)
    if not content:
        return
    report.check(
        "task-run" in content,
        "orchestrator.py includes task-run CLI mode",
        "orchestrator.py missing task-run CLI mode",
    )
    report.check(
        "mcp-agent-capability-map.yaml" in content,
        "orchestrator.py references capability map",
        "orchestrator.py should reference mcp-agent-capability-map.yaml",
    )
    report.check(
        "task_skill_mapping" in content,
        "orchestrator.py references task_skill_mapping",
        "orchestrator.py should reference task_skill_mapping",
    )
    report.check(
        "required_skills" in content,
        "orchestrator.py injects required_skills",
        "orchestrator.py should inject required_skills in task-run prompts",
    )
    report.check(
        "MCPConnector" in content,
        "orchestrator.py uses MCPConnector",
        "orchestrator.py should use MCPConnector for mcp-evidence collection",
    )
    report.check(
        "ClaudeBridge" in content,
        "orchestrator.py uses ClaudeBridge",
        "orchestrator.py should use ClaudeBridge for claude runtime",
    )
    report.check(
        re.search(r"\bdef _collect_mcp_evidence\(", content) is not None,
        "orchestrator.py includes _collect_mcp_evidence",
        "orchestrator.py missing _collect_mcp_evidence",
    )
    report.check(
        re.search(r"\bdef _collect_skill_context\(", content) is not None,
        "orchestrator.py includes _collect_skill_context",
        "orchestrator.py missing _collect_skill_context",
    )
    report.check(
        "required_skill_cards" in content,
        "orchestrator.py injects required_skill_cards",
        "orchestrator.py should inject required_skill_cards in task-run prompts",
    )
    report.check(
        "--mcp-strict" in content,
        "orchestrator.py exposes --mcp-strict",
        "orchestrator.py should expose --mcp-strict for MCP availability gating",
    )
    report.check(
        "--skills-strict" in content,
        "orchestrator.py exposes --skills-strict",
        "orchestrator.py should expose --skills-strict for skill availability gating",
    )
    report.check(
        "--triad" in content,
        "orchestrator.py exposes --triad",
        "orchestrator.py should expose --triad for third-agent audit",
    )
    report.check(
        "--summarizer" in content,
        "orchestrator.py exposes --summarizer for parallel synthesis",
        "orchestrator.py should expose --summarizer for post-parallel synthesis",
    )
    report.check(
        re.search(r"subparsers\.add_parser\(\s*[\"']doctor[\"']", content, flags=re.DOTALL)
        is not None,
        "orchestrator.py includes doctor preflight mode",
        "orchestrator.py should include doctor preflight mode",
    )
    report.check(
        re.search(r"\bdef doctor\(", content) is not None,
        "orchestrator.py includes doctor method",
        "orchestrator.py missing doctor method",
    )
    report.check(
        "--profile-file" in content,
        "orchestrator.py exposes --profile-file for run-level customization",
        "orchestrator.py should expose --profile-file for profile bundle injection",
    )
    report.check(
        "--summarizer-profile" in content,
        "orchestrator.py exposes --summarizer-profile",
        "orchestrator.py should expose --summarizer-profile in parallel mode",
    )
    for token in ("--profile", "--draft-profile", "--review-profile", "--triad-profile"):
        report.check(
            token in content,
            f"orchestrator.py exposes {token}",
            f"orchestrator.py missing {token} CLI option",
        )
    for token in (
        "DEFAULT_AGENT_PROFILES",
        "_load_profile_bundle",
        "_resolve_task_profile_names",
        "_profile_runtime_options",
        "_build_profile_directive",
    ):
        report.check(
            token in content,
            f"orchestrator.py includes {token}",
            f"orchestrator.py missing {token}",
        )
    report.check(
        "RUNTIME_AGENTS = {\"codex\", \"claude\", \"gemini\"}" in content,
        "orchestrator.py runtime agents include codex/claude/gemini",
        "orchestrator.py should include codex/claude/gemini in runtime agents",
    )
    report.check(
        "ThreadPoolExecutor" in content and "as_completed" in content,
        "orchestrator.py uses concurrent triad execution",
        "orchestrator.py should use concurrent execution for parallel mode",
    )
    report.check(
        "_build_parallel_synthesis_prompt" in content,
        "orchestrator.py includes synthesis prompt builder for parallel mode",
        "orchestrator.py missing synthesis prompt builder for parallel mode",
    )
    report.check(
        "_calculate_parallel_confidence" in content,
        "orchestrator.py includes parallel confidence calculation",
        "orchestrator.py missing parallel confidence calculation",
    )
    report.check(
        re.search(r"\bdef task_run\(", content) is not None,
        "orchestrator.py includes task_run method",
        "orchestrator.py missing task_run method",
    )


def validate_guides(root: Path, report: ValidationReport) -> None:
    content = read_text(root, "guides/agent-skill-collaboration.md", report)
    if not content:
        return
    for token in ("Task ID", "required_skills", "task-run", "--summarizer", "--profile-file", "doctor"):
        report.check(
            token in content,
            f"collaboration guide includes {token}",
            f"guides/agent-skill-collaboration.md missing key token: {token}",
        )
    report.check(
        "RESEARCH_MCP_" in content,
        "collaboration guide documents external MCP env contract",
        "guides/agent-skill-collaboration.md should include RESEARCH_MCP_ env contract",
    )

    install_content = read_text(root, "guides/install-multi-client.md", report)
    if not install_content:
        return
    for token in (
        "install_research_skill.sh",
        "--target all",
        "--mode copy|link",
        "CODEX_HOME",
        "CLAUDE_CODE_HOME",
        "GEMINI_HOME",
        "bridges.orchestrator doctor",
    ):
        report.check(
            token in install_content,
            f"install guide includes {token}",
            f"guides/install-multi-client.md missing token: {token}",
        )


def validate_mcp_connector(root: Path, report: ValidationReport) -> None:
    content = read_text(root, "bridges/mcp_connectors.py", report)
    if not content:
        return
    for token in ("class MCPConnector", "class MCPEvidence", "RESEARCH_MCP_", "filesystem"):
        report.check(
            token in content,
            f"mcp_connectors.py includes {token}",
            f"bridges/mcp_connectors.py missing token: {token}",
        )


def validate_claude_bridge(root: Path, report: ValidationReport) -> None:
    content = read_text(root, "bridges/claude_bridge.py", report)
    if not content:
        return
    for token in (
        "class ClaudeBridge",
        "model_type = ModelType.CLAUDE",
        "\"claude\"",
        "--output-format",
        "stream-json",
    ):
        report.check(
            token in content,
            f"claude_bridge.py includes {token}",
            f"bridges/claude_bridge.py missing token: {token}",
        )


def validate_base_bridge(root: Path, report: ValidationReport) -> None:
    content = read_text(root, "bridges/base_bridge.py", report)
    if not content:
        return
    for token in (
        "DEFAULT_TIMEOUT_SECONDS",
        "non_interactive",
        "require_api_key",
        "AUTH_ENV_BY_MODEL",
        "CI",
        "TERM",
    ):
        report.check(
            token in content,
            f"base_bridge.py includes {token}",
            f"bridges/base_bridge.py missing token: {token}",
        )


def validate_tests(root: Path, report: ValidationReport) -> None:
    content = read_text(root, "tests/test_orchestrator_workflows.py", report)
    if not content:
        return
    for token in (
        "MockOrchestrator",
        "CollaborationMode.PARALLEL",
        "task_run(",
        "profile_file",
        "Unknown agent profile",
    ):
        report.check(
            token in content,
            f"test_orchestrator_workflows.py includes {token}",
            f"tests/test_orchestrator_workflows.py missing token: {token}",
        )


def validate_profile_bundle_template(root: Path, report: ValidationReport) -> None:
    relative_path = "standards/agent-profiles.example.json"
    content = read_text(root, relative_path, report)
    if not content:
        return
    try:
        payload = json.loads(content)
    except json.JSONDecodeError as exc:
        report.check(
            False,
            "",
            f"{relative_path} is invalid JSON: {exc}",
        )
        return

    report.check(
        isinstance(payload, dict),
        f"{relative_path} root is JSON object",
        f"{relative_path} root should be JSON object",
    )
    if not isinstance(payload, dict):
        return

    profiles = payload.get("profiles")
    report.check(
        isinstance(profiles, dict) and bool(profiles),
        f"{relative_path} includes non-empty profiles",
        f"{relative_path} missing non-empty profiles object",
    )
    if isinstance(profiles, dict):
        for profile_name in ("default", "strict-review", "rapid-draft"):
            report.warn(
                profile_name in profiles,
                f"{relative_path} includes {profile_name}",
                f"{relative_path} should include {profile_name} profile",
            )
        for name, cfg in profiles.items():
            report.check(
                isinstance(cfg, dict),
                f"{relative_path} profile '{name}' is object",
                f"{relative_path} profile '{name}' should be object",
            )
            if not isinstance(cfg, dict):
                continue
            runtime_options = cfg.get("runtime_options", {})
            report.warn(
                isinstance(runtime_options, dict),
                f"{relative_path} profile '{name}' runtime_options is object",
                f"{relative_path} profile '{name}' should define runtime_options object",
            )
            if isinstance(runtime_options, dict):
                unknown_agents = set(runtime_options) - EXPECTED_AGENTS
                report.check(
                    not unknown_agents,
                    f"{relative_path} profile '{name}' runtime_options targets known agents",
                    (
                        f"{relative_path} profile '{name}' has unknown runtime agent keys: "
                        f"{ids_to_text(unknown_agents) or 'none'}"
                    ),
                )

    task_overrides = payload.get("task_overrides", {})
    report.check(
        isinstance(task_overrides, dict),
        f"{relative_path} task_overrides is object",
        f"{relative_path} task_overrides should be JSON object",
    )


def validate_ci_workflow(root: Path, report: ValidationReport) -> None:
    content = read_text(root, ".github/workflows/ci.yml", report)
    if not content:
        return
    for token in (
        "actions/checkout",
        "actions/setup-python",
        "python -m py_compile",
        "bash -n scripts/generate_release_notes.sh",
        "bash -n scripts/release_preflight.sh",
        "bash -n scripts/release_postflight.sh",
        "bash -n scripts/release_automation.sh",
        "bash -n scripts/install_research_skill.sh",
        "Run standardized pre-release gates",
    ):
        report.check(
            token in content,
            f"ci.yml includes {token}",
            f".github/workflows/ci.yml missing token: {token}",
        )

    release_workflow = read_text(root, ".github/workflows/release-automation.yml", report)
    if release_workflow:
        for token in (
            "workflow_dispatch",
            "inputs:",
            "Run release automation",
            "release_automation.sh",
            "create_release",
            "skip_note_gen",
            "note_overwrite",
            "from_tag",
        ):
            report.check(
                token in release_workflow,
                f"release-automation.yml includes {token}",
                f".github/workflows/release-automation.yml missing token: {token}",
            )


def validate_release_artifacts(root: Path, report: ValidationReport) -> None:
    smoke_content = read_text(root, "scripts/run_beta_smoke.sh", report)
    if smoke_content:
        for token in (
            "bridges.orchestrator doctor",
            "bridges.orchestrator parallel",
            "bridges.orchestrator task-run",
            "smoke-fast",
        ):
            report.check(
                token in smoke_content,
                f"run_beta_smoke.sh includes {token}",
                f"scripts/run_beta_smoke.sh missing token: {token}",
            )

    preflight_content = read_text(root, "scripts/release_preflight.sh", report)
    if preflight_content:
        for token in (
            "validate_research_standard.py",
            "tests.test_orchestrator_workflows",
            "./scripts/run_beta_smoke.sh",
            "generate_release_notes.sh",
            "--skip-note-gen",
            "--from-tag",
            "--update-existing",
            "--validator-result",
            "--unittest-result",
            "--smoke-result",
            "release note evidence update",
            "--skip-smoke",
            "--tag",
        ):
            report.check(
                token in preflight_content,
                f"release_preflight.sh includes {token}",
                f"scripts/release_preflight.sh missing token: {token}",
            )

    note_gen_content = read_text(root, "scripts/generate_release_notes.sh", report)
    if note_gen_content:
        for token in (
            "--tag <tag>",
            "--from-tag <tag>",
            "release/${TAG}.md",
            "## Highlights (Draft)",
            "## Validation Evidence",
            "## Publish Steps",
            "--update-existing",
            "--validator-result",
            "--unittest-result",
            "--smoke-result",
            "updated evidence lines",
        ):
            report.check(
                token in note_gen_content,
                f"generate_release_notes.sh includes {token}",
                f"scripts/generate_release_notes.sh missing token: {token}",
            )

    postflight_content = read_text(root, "scripts/release_postflight.sh", report)
    if postflight_content:
        for token in (
            "git ls-remote --heads origin",
            "actions/runs?branch",
            "release/templates/beta-acceptance-template.md",
            "--create-release",
            "gh release view",
        ):
            report.check(
                token in postflight_content,
                f"release_postflight.sh includes {token}",
                f"scripts/release_postflight.sh missing token: {token}",
            )

    automation_content = read_text(root, "scripts/release_automation.sh", report)
    if automation_content:
        for token in (
            "release_preflight.sh",
            "release_postflight.sh",
            "<pre|post|full>",
            "--from-tag",
            "--skip-note-gen",
        ):
            report.check(
                token in automation_content,
                f"release_automation.sh includes {token}",
                f"scripts/release_automation.sh missing token: {token}",
            )

    installer_content = read_text(root, "scripts/install_research_skill.sh", report)
    if installer_content:
        for token in (
            "--target <codex|claude|gemini|all>",
            "--project-dir",
            "--doctor",
            "research-paper-workflow",
            ".agent/workflows",
            "research-skills.md",
        ):
            report.check(
                token in installer_content,
                f"install_research_skill.sh includes {token}",
                f"scripts/install_research_skill.sh missing token: {token}",
            )

    template_content = read_text(
        root, "release/templates/beta-acceptance-template.md", report
    )
    if template_content:
        for token in ("{{TAG}}", "{{DATE}}", "{{COMMIT}}", "{{CI_STATUS}}"):
            report.check(
                token in template_content,
                f"beta-acceptance-template.md includes {token}",
                f"release/templates/beta-acceptance-template.md missing token: {token}",
            )

    automation_doc = read_text(root, "release/automation.md", report)
    if automation_doc:
        for token in (
            "release_preflight.sh",
            "release_postflight.sh",
            "release_automation.sh",
            "pre --tag",
            "post --tag",
        ):
            report.check(
                token in automation_doc,
                f"release/automation.md includes {token}",
                f"release/automation.md missing token: {token}",
            )

    latest_note = select_latest_release_note(root)
    report.check(
        latest_note is not None,
        "release includes at least one beta release note",
        "release/ missing beta release note files (expected vX.Y.Z-beta.N.md)",
    )
    if latest_note is not None:
        relative_note_path = str(latest_note.relative_to(root))
        notes_content = read_text(root, relative_note_path, report)
        for token in (
            "Release Notes",
            "Validation Evidence",
            "Publish Steps",
            "rollback.md",
        ):
            report.check(
                token in notes_content,
                f"{latest_note.name} includes {token}",
                f"{relative_note_path} missing token: {token}",
            )

    rollback_content = read_text(root, "release/rollback.md", report)
    if rollback_content:
        for token in (
            "Git Tag Rollback",
            "Commit-Level Rollback",
            "Recovery Validation",
            "validate_research_standard.py --strict",
        ):
            report.check(
                token in rollback_content,
                f"rollback.md includes {token}",
                f"release/rollback.md missing token: {token}",
            )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate cross-model research workflow standardization consistency."
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=Path.cwd(),
        help="Repository root path (default: current directory).",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Treat warnings as failures.",
    )
    args = parser.parse_args()

    root = args.root.resolve()
    report = ValidationReport()

    print(f"Validating research standard in: {root}")
    validate_contract(root, report)
    validate_mcp_agent_map(root, report)
    validate_portable_skill(root, report)
    validate_cross_file_consistency(root, report)
    validate_mcp_connector(root, report)
    validate_claude_bridge(root, report)
    validate_base_bridge(root, report)
    validate_tests(root, report)
    validate_orchestrator(root, report)
    validate_profile_bundle_template(root, report)
    validate_ci_workflow(root, report)
    validate_release_artifacts(root, report)
    validate_guides(root, report)
    validate_docs(root, report)

    total_failed = len(report.errors)
    total_warn = len(report.warnings)
    print(
        f"\nSummary: {report.passed} passed, {total_failed} failed, {total_warn} warnings"
    )
    if report.errors:
        return 1
    if args.strict and report.warnings:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
