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
    "H1", "H2", "H2_5", "H3", "H4",
    "I1", "I2", "I3", "I4", "I5", "I6", "I7", "I8",
}
EXPECTED_QUALITY_GATES = {"Q1", "Q2", "Q3", "Q4"}
EXPECTED_AGENTS = {"codex", "claude", "gemini"}
EXPECTED_FUNCTIONAL_AGENTS = {
    "research-orchestrator",
    "literature-agent",
    "theory-agent",
    "methodology-agent",
    "data-agent",
    "analysis-agent",
    "writing-agent",
    "publication-agent",
}
EXPECTED_SKILL_STAGES = {
    "A_framing",
    "B_literature",
    "C_design",
    "D_ethics",
    "E_synthesis",
    "F_writing",
    "G_compliance",
    "H_submission",
    "I_code",
    "Z_cross_cutting",
}
EXPECTED_ROUTING_STAGE_IDS = EXPECTED_STAGE_IDS | {"J"}
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
    ".agent/workflows/proofread.md": {"J1", "J2", "J3", "J4"},
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


@dataclass
class SkillRegistryEntry:
    id: str
    stage: str
    file: str
    inputs: list[str]
    outputs: list[str]


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


def strip_quotes(value: str) -> str:
    value = value.strip()
    if (value.startswith('"') and value.endswith('"')) or (
        value.startswith("'") and value.endswith("'")
    ):
        return value[1:-1]
    return value


def parse_inline_yaml_list(block: str, key: str, indent: int) -> list[str]:
    match = re.search(
        rf"^\s{{{indent}}}{re.escape(key)}:\s*\[(.*?)\]\s*$",
        block,
        flags=re.MULTILINE | re.DOTALL,
    )
    if not match:
        return []
    raw = match.group(1).strip()
    if not raw:
        return []
    return [strip_quotes(item) for item in re.split(r"\s*,\s*", raw) if item.strip()]


def iter_yaml_list_blocks(section: str, item_indent: int) -> list[tuple[str, str]]:
    pattern = re.compile(
        rf"^\s{{{item_indent}}}-\s*id:\s*([A-Za-z0-9_-]+)\s*$",
        flags=re.MULTILINE,
    )
    matches = list(pattern.finditer(section))
    blocks: list[tuple[str, str]] = []
    for index, match in enumerate(matches):
        start = match.start()
        end = matches[index + 1].start() if index + 1 < len(matches) else len(section)
        blocks.append((match.group(1), section[start:end]))
    return blocks


def parse_artifact_types(content: str) -> list[str]:
    return re.findall(
        r'^\s*-\s*name:\s*([A-Za-z][A-Za-z0-9]+)\s*$',
        content,
        flags=re.MULTILINE,
    )


def parse_skill_registry_entries(content: str) -> dict[str, SkillRegistryEntry]:
    section = extract_top_level_section(content, "skills")
    entries: dict[str, SkillRegistryEntry] = {}
    for skill_id, block in iter_yaml_list_blocks(section, item_indent=2):
        stage = parse_yaml_scalar(block, "stage", indent=4)
        file_path = parse_yaml_scalar(block, "file", indent=4)
        inputs = parse_inline_yaml_list(block, "inputs", indent=4)
        outputs = parse_inline_yaml_list(block, "outputs", indent=4)
        entries[skill_id] = SkillRegistryEntry(
            id=skill_id,
            stage=stage,
            file=file_path,
            inputs=inputs,
            outputs=outputs,
        )
    return entries


def parse_map_skill_registry(content: str) -> set[str]:
    section = extract_top_level_section(content, "skill_registry")
    return set(parse_yaml_list(section, item_indent=2))


def parse_map_skill_catalog_files(content: str) -> dict[str, str]:
    section = extract_top_level_section(content, "skill_catalog")
    files: dict[str, str] = {}
    for skill_name, block in re.findall(
        r"^\s{2}([a-z0-9-]+):\n((?:^\s{4}.*\n?)+)",
        section,
        flags=re.MULTILINE,
    ):
        files[skill_name] = parse_yaml_scalar(block, "file", indent=4)
    return files


def parse_task_skill_mapping(content: str) -> dict[str, set[str]]:
    section = extract_top_level_section(content, "task_skill_mapping")
    task_map: dict[str, set[str]] = {}
    for task_id, block in re.findall(
        r"^\s{2}([A-I][0-9_]+):\n((?:^\s{4}.*\n?)+)",
        section,
        flags=re.MULTILINE,
    ):
        task_map[task_id] = set(
            parse_yaml_list(extract_nested_section(block, "required_skills", indent=4), item_indent=6)
        )
    return task_map


def parse_task_functional_routing(content: str) -> tuple[dict[str, str], dict[str, str]]:
    routing_section = extract_top_level_section(content, "task_functional_routing")
    defaults_section = extract_nested_section(routing_section, "defaults_by_stage", indent=2)
    routing_defaults = {
        stage: owner
        for stage, owner in re.findall(
            r'^\s{4}([A-Z]):\s*"?(.*?)"?\s*$',
            defaults_section,
            flags=re.MULTILINE,
        )
    }
    overrides_section = extract_nested_section(routing_section, "overrides", indent=2)
    routing_overrides = {
        task_id: owner
        for task_id, owner in re.findall(
            r'^\s{4}([A-I][0-9_]+):\s*"?(.*?)"?\s*$',
            overrides_section,
            flags=re.MULTILINE,
        )
    }
    return routing_defaults, routing_overrides


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

    dependency_section = extract_top_level_section(content, "dependency_catalog")
    report.check(
        bool(dependency_section),
        "Dependency catalog exists in contract",
        "dependency_catalog missing in standards/research-workflow-contract.yaml",
    )
    if dependency_section:
        dependency_ids = {
            found
            for found in re.findall(
                r"^\s{2}([A-I][0-9_]+):\s*$",
                dependency_section,
                flags=re.MULTILINE,
            )
        }
        report.check(
            dependency_ids == EXPECTED_TASK_IDS,
            "Dependency catalog covers all canonical Task IDs",
            (
                "Dependency catalog mismatch. Missing: "
                f"{ids_to_text(EXPECTED_TASK_IDS - dependency_ids) or 'none'}; "
                f"Extra: {ids_to_text(dependency_ids - EXPECTED_TASK_IDS) or 'none'}"
            ),
        )
        prereq_all_graph: dict[str, list[str]] = {task_id: [] for task_id in EXPECTED_TASK_IDS}
        for dep_task_id, dep_block in re.findall(
            r"^\s{2}([A-I][0-9_]+):\n((?:^\s{4}.*\n?)+)",
            dependency_section,
            flags=re.MULTILINE,
        ):
            prereq_all = parse_yaml_list(
                extract_nested_section(dep_block, "prerequisites_all", indent=4),
                item_indent=6,
            )
            prereq_all_graph[dep_task_id] = prereq_all

            for field_name in (
                "prerequisites_all",
                "prerequisites_any",
                "recommended_prerequisites",
                "recommended_next",
            ):
                field_values = set(
                    parse_yaml_list(
                        extract_nested_section(dep_block, field_name, indent=4),
                        item_indent=6,
                    )
                )
                report.check(
                    field_values.issubset(EXPECTED_TASK_IDS),
                    f"{dep_task_id} dependency {field_name} references canonical tasks",
                    (
                        f"{dep_task_id} dependency {field_name} references unknown tasks: "
                        f"{ids_to_text(field_values - EXPECTED_TASK_IDS) or 'none'}"
                    ),
                )

        visiting: set[str] = set()
        visited: set[str] = set()

        def dfs(node: str) -> None:
            if node in visited:
                return
            if node in visiting:
                raise ValueError(f"cycle detected near task: {node}")
            visiting.add(node)
            for prereq in prereq_all_graph.get(node, []):
                if prereq in prereq_all_graph:
                    dfs(prereq)
            visiting.remove(node)
            visited.add(node)

        cycle_error = ""
        try:
            for node in sorted(EXPECTED_TASK_IDS):
                dfs(node)
        except ValueError as exc:
            cycle_error = str(exc)
        report.check(
            not cycle_error,
            "Dependency catalog prerequisites_all are acyclic",
            f"Dependency catalog prerequisites_all contains cycle: {cycle_error}",
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


def validate_skill_registry(root: Path, report: ValidationReport) -> None:
    artifact_content = read_text(root, "schemas/artifact-types.yaml", report)
    artifact_types = parse_artifact_types(artifact_content) if artifact_content else []
    artifact_type_set = set(artifact_types)
    report.check(
        bool(artifact_types),
        "Artifact type vocabulary is non-empty",
        "schemas/artifact-types.yaml must define at least one artifact type",
    )
    report.check(
        len(artifact_type_set) == len(artifact_types),
        "Artifact type vocabulary has unique names",
        "schemas/artifact-types.yaml contains duplicate artifact type names",
    )

    registry_content = read_text(root, "skills/registry.yaml", report)
    if not registry_content:
        return

    registry_entries = parse_skill_registry_entries(registry_content)
    report.check(
        bool(registry_entries),
        "skills/registry.yaml contains skill entries",
        "skills/registry.yaml must contain at least one skill entry",
    )

    seen_files: set[str] = set()
    for skill_id in sorted(registry_entries):
        entry = registry_entries[skill_id]
        report.check(
            entry.stage in EXPECTED_SKILL_STAGES,
            f"Registry stage is canonical for {skill_id}",
            f"skills/registry.yaml stage must be canonical for {skill_id}: {entry.stage}",
        )
        report.check(
            bool(entry.file),
            f"Registry file path exists for {skill_id}",
            f"skills/registry.yaml missing file path for {skill_id}",
        )
        if entry.file:
            report.check(
                entry.file.startswith("skills/"),
                f"Registry file path stays under skills/ for {skill_id}",
                f"skills/registry.yaml file path must stay under skills/ for {skill_id}: {entry.file}",
            )
            report.check(
                entry.file not in seen_files,
                f"Registry file path is unique for {skill_id}",
                f"skills/registry.yaml duplicates file path: {entry.file}",
            )
            seen_files.add(entry.file)
            report.check(
                (root / entry.file).exists(),
                f"Registry file exists on disk for {skill_id}",
                f"skills/registry.yaml points to missing file for {skill_id}: {entry.file}",
            )
            report.check(
                re.search(rf"^skills/{re.escape(entry.stage)}/{re.escape(skill_id)}\.md$", entry.file)
                is not None,
                f"Registry file path matches stage/id for {skill_id}",
                (
                    "skills/registry.yaml file path must match stage/id convention for "
                    f"{skill_id}: {entry.file}"
                ),
            )

        report.check(
            set(entry.inputs).issubset(artifact_type_set),
            f"Registry inputs use known artifact types for {skill_id}",
            (
                f"skills/registry.yaml inputs for {skill_id} reference unknown artifact types: "
                f"{ids_to_text(set(entry.inputs) - artifact_type_set) or 'none'}"
            ),
        )
        report.check(
            set(entry.outputs).issubset(artifact_type_set),
            f"Registry outputs use known artifact types for {skill_id}",
            (
                f"skills/registry.yaml outputs for {skill_id} reference unknown artifact types: "
                f"{ids_to_text(set(entry.outputs) - artifact_type_set) or 'none'}"
            ),
        )

        skill_path = root / entry.file
        if not skill_path.exists():
            continue
        frontmatter, _body = parse_frontmatter(skill_path.read_text(encoding="utf-8"))
        report.check(
            frontmatter.get("id") == skill_id,
            f"Skill frontmatter id matches registry for {skill_id}",
            (
                f"{entry.file} frontmatter id mismatch: "
                f"{frontmatter.get('id', '<missing>')} vs registry {skill_id}"
            ),
        )
        report.check(
            frontmatter.get("stage") == entry.stage,
            f"Skill frontmatter stage matches registry for {skill_id}",
            (
                f"{entry.file} frontmatter stage mismatch: "
                f"{frontmatter.get('stage', '<missing>')} vs registry {entry.stage}"
            ),
        )


def validate_single_skill_source_of_truth(root: Path, report: ValidationReport) -> None:
    forbidden = sorted(
        path.relative_to(root)
        for pattern in ("skill.yaml", "skill.yml")
        for path in root.rglob(pattern)
        if path.is_file()
    )
    report.check(
        not forbidden,
        "Repository has no hand-written skill.yaml/skill.yml files",
        (
            "Found hand-written skill truth files that would create a third source of truth: "
            f"{', '.join(str(path) for path in forbidden)}"
        ),
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

    functional_agent_section = extract_top_level_section(map_content, "functional_agent_registry")
    functional_agent_list = parse_yaml_list(functional_agent_section, item_indent=2)
    functional_agent_registry = set(functional_agent_list)
    report.check(
        functional_agent_registry.issuperset(EXPECTED_FUNCTIONAL_AGENTS),
        "Functional agent registry includes required baseline agents",
        (
            "Functional agent registry missing required agents: "
            f"{ids_to_text(EXPECTED_FUNCTIONAL_AGENTS - functional_agent_registry) or 'none'}"
        ),
    )
    report.check(
        len(functional_agent_registry) == len(functional_agent_list),
        "Functional agent registry has unique entries",
        "Functional agent registry has duplicate entries",
    )

    functional_agents_section = extract_top_level_section(map_content, "functional_agents")
    functional_agent_blocks = {
        found: block
        for found, block in re.findall(
            r"^\s{2}([a-z0-9-]+):\n((?:^\s{4}.*\n?)+)",
            functional_agents_section,
            flags=re.MULTILINE,
        )
    }
    report.check(
        set(functional_agent_blocks) == functional_agent_registry,
        "functional_agents section covers all registered functional agents",
        (
            "functional_agents mismatch. Missing: "
            f"{ids_to_text(functional_agent_registry - set(functional_agent_blocks)) or 'none'}; "
            f"Extra: {ids_to_text(set(functional_agent_blocks) - functional_agent_registry) or 'none'}"
        ),
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

    registry_content = read_text(root, "skills/registry.yaml", report)
    registry_entries = parse_skill_registry_entries(registry_content) if registry_content else {}
    report.check(
        skill_registry == set(registry_entries),
        "Capability-map skill registry matches skills/registry.yaml",
        (
            "Capability-map skill registry mismatch vs skills/registry.yaml. Missing: "
            f"{ids_to_text(set(registry_entries) - skill_registry) or 'none'}; "
            f"Extra: {ids_to_text(skill_registry - set(registry_entries)) or 'none'}"
        ),
    )
    for skill_name in sorted(skill_registry):
        pass # The existence of the file is validated via skill_catalog below

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
        if skill_name in registry_entries:
            report.check(
                skill_file == registry_entries[skill_name].file,
                f"Skill catalog file matches registry for {skill_name}",
                (
                    f"Skill catalog file mismatch for {skill_name}: "
                    f"{skill_file} vs registry {registry_entries[skill_name].file}"
                ),
            )

    for functional_agent_name in sorted(functional_agent_registry):
        block = functional_agent_blocks.get(functional_agent_name, "")
        agent_file = parse_yaml_scalar(block, "file", indent=4)
        focus = parse_yaml_scalar(block, "focus", indent=4)
        mapped_role = parse_yaml_scalar(block, "mapped_role", indent=4)
        owns_stages = set(
            parse_yaml_list(extract_nested_section(block, "owns_stages", indent=4), item_indent=6)
        )
        report.check(
            bool(agent_file),
            f"Functional agent has file: {functional_agent_name}",
            f"functional_agents entry missing file: {functional_agent_name}",
        )
        if agent_file:
            report.check(
                agent_file.startswith("roles/"),
                f"Functional agent file stays under roles/: {functional_agent_name}",
                f"functional_agents file should stay under roles/: {functional_agent_name} -> {agent_file}",
            )
            report.check(
                (root / agent_file).exists(),
                f"Functional agent role file exists: {functional_agent_name}",
                f"functional_agents file missing on disk: {functional_agent_name} -> {agent_file}",
            )
        report.check(
            bool(focus),
            f"Functional agent has focus: {functional_agent_name}",
            f"functional_agents entry missing focus: {functional_agent_name}",
        )
        report.check(
            bool(owns_stages),
            f"Functional agent owns stages: {functional_agent_name}",
            f"functional_agents entry missing owns_stages: {functional_agent_name}",
        )
        report.check(
            owns_stages.issubset(EXPECTED_ROUTING_STAGE_IDS),
            f"Functional agent owns canonical stages: {functional_agent_name}",
            (
                f"functional_agents entry for {functional_agent_name} references unknown stages: "
                f"{ids_to_text(owns_stages - EXPECTED_ROUTING_STAGE_IDS) or 'none'}"
            ),
        )

        if agent_file and (root / agent_file).exists():
            role_content = read_text(root, agent_file, report)
            role_id = parse_yaml_scalar(role_content, "id", indent=0)
            preferred_skills = set(
                parse_yaml_list(extract_top_level_section(role_content, "preferred_skills"), item_indent=2)
            )
            expected_role_id = mapped_role or functional_agent_name
            report.check(
                bool(role_id),
                f"Role file declares id: {functional_agent_name}",
                f"{agent_file} missing top-level id",
            )
            if role_id:
                report.check(
                    role_id == expected_role_id,
                    f"Role file id matches functional mapping: {functional_agent_name}",
                    (
                        f"{agent_file} id mismatch for {functional_agent_name}: "
                        f"{role_id} vs expected {expected_role_id}"
                    ),
                )
            report.check(
                preferred_skills.issubset(skill_registry),
                f"Role preferred_skills are registered for {functional_agent_name}",
                (
                    f"{agent_file} references unknown preferred_skills: "
                    f"{ids_to_text(preferred_skills - skill_registry) or 'none'}"
                ),
            )

    routing_defaults, routing_overrides = parse_task_functional_routing(map_content)
    report.check(
        set(routing_defaults) == EXPECTED_ROUTING_STAGE_IDS,
        "task_functional_routing defaults cover canonical stages",
        (
            "task_functional_routing defaults mismatch. Missing: "
            f"{ids_to_text(EXPECTED_ROUTING_STAGE_IDS - set(routing_defaults)) or 'none'}; "
            f"Extra: {ids_to_text(set(routing_defaults) - EXPECTED_ROUTING_STAGE_IDS) or 'none'}"
        ),
    )
    report.check(
        set(routing_defaults.values()).issubset(functional_agent_registry),
        "task_functional_routing defaults use registered functional agents",
        (
            "task_functional_routing defaults reference unknown functional agents: "
            f"{ids_to_text(set(routing_defaults.values()) - functional_agent_registry) or 'none'}"
        ),
    )

    report.check(
        set(routing_overrides).issubset(EXPECTED_TASK_IDS),
        "task_functional_routing overrides reference canonical Task IDs",
        (
            "task_functional_routing overrides reference unknown Task IDs: "
            f"{ids_to_text(set(routing_overrides) - EXPECTED_TASK_IDS) or 'none'}"
        ),
    )
    report.check(
        set(routing_overrides.values()).issubset(functional_agent_registry),
        "task_functional_routing overrides use registered functional agents",
        (
            "task_functional_routing overrides reference unknown functional agents: "
            f"{ids_to_text(set(routing_overrides.values()) - functional_agent_registry) or 'none'}"
        ),
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
        resolved_owner = routing_overrides.get(task_id) or routing_defaults.get(task_id[0])
        report.check(
            bool(resolved_owner),
            f"{task_id} resolves to a functional owner",
            f"{task_id} has no functional owner in task_functional_routing",
        )
        if resolved_owner:
            report.check(
                resolved_owner in functional_agent_registry,
                f"{task_id} functional owner is registered",
                f"{task_id} resolves to unknown functional owner: {resolved_owner}",
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


def validate_pipelines(root: Path, report: ValidationReport) -> None:
    registry_content = read_text(root, "skills/registry.yaml", report)
    artifact_content = read_text(root, "schemas/artifact-types.yaml", report)
    map_content = read_text(root, "standards/mcp-agent-capability-map.yaml", report)
    if not registry_content or not artifact_content or not map_content:
        return

    registry_entries = parse_skill_registry_entries(registry_content)
    artifact_types = set(parse_artifact_types(artifact_content))
    task_skill_map = parse_task_skill_mapping(map_content)
    routing_defaults, routing_overrides = parse_task_functional_routing(map_content)
    functional_registry = parse_yaml_list(
        extract_top_level_section(map_content, "functional_agent_registry"),
        item_indent=2,
    )
    functional_agent_registry = set(functional_registry)

    pipeline_dir = root / "pipelines"
    pipeline_paths = sorted(pipeline_dir.glob("*.yaml"))
    report.check(
        bool(pipeline_paths),
        "pipelines/ contains YAML pipeline definitions",
        "pipelines/ must contain at least one .yaml pipeline definition",
    )

    for pipeline_path in pipeline_paths:
        relative_path = str(pipeline_path.relative_to(root))
        content = read_text(root, relative_path, report)
        if not content:
            continue

        pipeline_id = parse_yaml_scalar(content, "id", indent=0)
        pipeline_paper_type = parse_yaml_scalar(content, "paper_type", indent=0)
        report.check(
            bool(pipeline_id),
            f"{relative_path} declares pipeline id",
            f"{relative_path} missing top-level id",
        )
        if pipeline_id:
            report.check(
                pipeline_path.stem == pipeline_id,
                f"{relative_path} filename matches pipeline id",
                f"{relative_path} filename must match top-level id {pipeline_id}",
            )
        report.check(
            pipeline_paper_type in EXPECTED_PAPER_TYPES,
            f"{relative_path} paper_type is canonical",
            f"{relative_path} has non-canonical paper_type: {pipeline_paper_type or '<missing>'}",
        )

        steps_section = extract_top_level_section(content, "steps")
        step_blocks = iter_yaml_list_blocks(steps_section, item_indent=2)
        report.check(
            bool(step_blocks),
            f"{relative_path} contains step blocks",
            f"{relative_path} must contain at least one pipeline step",
        )

        step_ids = [step_id for step_id, _block in step_blocks]
        report.check(
            len(step_ids) == len(set(step_ids)),
            f"{relative_path} step ids are unique",
            f"{relative_path} has duplicate pipeline step ids",
        )
        step_id_set = set(step_ids)

        for step_id, block in step_blocks:
            skill_name = parse_yaml_scalar(block, "skill", indent=4)
            task_id = parse_yaml_scalar(block, "task_id", indent=4)
            outputs = parse_inline_yaml_list(block, "outputs", indent=4)
            depends_on = parse_inline_yaml_list(block, "depends_on", indent=4)
            owner = parse_yaml_scalar(block, "owner_functional_agent", indent=4)

            report.check(
                bool(skill_name),
                f"{relative_path} step {step_id} has skill",
                f"{relative_path} step {step_id} missing skill",
            )
            if skill_name:
                report.check(
                    skill_name in registry_entries,
                    f"{relative_path} step {step_id} skill is registered",
                    (
                        f"{relative_path} step {step_id} references unknown skill: "
                        f"{skill_name}"
                    ),
                )

            report.check(
                task_id in EXPECTED_TASK_IDS,
                f"{relative_path} step {step_id} task_id is canonical",
                (
                    f"{relative_path} step {step_id} references unknown task_id: "
                    f"{task_id or '<missing>'}"
                ),
            )

            report.check(
                len(outputs) > 0,
                f"{relative_path} step {step_id} declares outputs",
                f"{relative_path} step {step_id} missing outputs list",
            )
            report.check(
                set(outputs).issubset(artifact_types),
                f"{relative_path} step {step_id} outputs use known artifact types",
                (
                    f"{relative_path} step {step_id} references unknown artifact types: "
                    f"{ids_to_text(set(outputs) - artifact_types) or 'none'}"
                ),
            )

            if skill_name in registry_entries:
                registry_outputs = set(registry_entries[skill_name].outputs)
                report.check(
                    set(outputs).issubset(registry_outputs),
                    f"{relative_path} step {step_id} outputs match registry skill outputs",
                    (
                        f"{relative_path} step {step_id} outputs are not declared by "
                        f"{skill_name}: {ids_to_text(set(outputs) - registry_outputs) or 'none'}"
                    ),
                )

            report.check(
                set(depends_on).issubset(step_id_set),
                f"{relative_path} step {step_id} depends_on references known steps",
                (
                    f"{relative_path} step {step_id} depends_on unknown step ids: "
                    f"{ids_to_text(set(depends_on) - step_id_set) or 'none'}"
                ),
            )

            if task_id in task_skill_map and skill_name:
                report.check(
                    skill_name in task_skill_map[task_id],
                    f"{relative_path} step {step_id} skill is allowed for {task_id}",
                    (
                        f"{relative_path} step {step_id} uses skill {skill_name} which is not "
                        f"listed in task_skill_mapping.{task_id}"
                    ),
                )

            resolved_owner = routing_overrides.get(task_id) or routing_defaults.get(task_id[:1])
            report.check(
                bool(owner),
                f"{relative_path} step {step_id} declares owner_functional_agent",
                f"{relative_path} step {step_id} missing owner_functional_agent",
            )
            if owner:
                report.check(
                    owner in functional_agent_registry,
                    f"{relative_path} step {step_id} owner_functional_agent is registered",
                    (
                        f"{relative_path} step {step_id} owner_functional_agent is unknown: "
                        f"{owner}"
                    ),
                )
                if resolved_owner:
                    report.check(
                        owner == resolved_owner,
                        f"{relative_path} step {step_id} owner_functional_agent matches task routing",
                        (
                            f"{relative_path} step {step_id} owner_functional_agent mismatch: "
                            f"{owner} vs expected {resolved_owner} for task {task_id}"
                        ),
                    )

    for relative_path, expected_ids in WORKFLOW_TASK_EXPECTATIONS.items():
        content = read_text(root, relative_path, report)
        if not content:
            continue
        report.check(
            "research-paper-workflow" in content,
            f"{relative_path} references global skill",
            f"{relative_path} missing global skill reference",
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
            "guides/advanced/agent-skill-collaboration.md" in content,
            f"{relative_path} references collaboration guide",
            f"{relative_path} should reference guides/advanced/agent-skill-collaboration.md",
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
    for token in ("--profile", "--draft-profile", "--review-profile", "--triad-profile", "--role"):
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
    content = read_text(root, "guides/advanced/agent-skill-collaboration.md", report)
    if not content:
        return
    for token in ("Task ID", "required_skills", "task-run", "--summarizer", "--profile-file", "doctor"):
        report.check(
            token in content,
            f"collaboration guide includes {token}",
            f"guides/advanced/agent-skill-collaboration.md missing key token: {token}",
        )
    report.check(
        "RESEARCH_MCP_" in content,
        "collaboration guide documents external MCP env contract",
        "guides/advanced/agent-skill-collaboration.md should include RESEARCH_MCP_ env contract",
    )

    install_content = read_text(root, "guides/basic/install-multi-client.md", report)
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
            f"guides/basic/install-multi-client.md missing token: {token}",
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
        "ConfigError",
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
    validate_skill_registry(root, report)
    validate_single_skill_source_of_truth(root, report)
    validate_mcp_agent_map(root, report)
    validate_portable_skill(root, report)
    validate_cross_file_consistency(root, report)
    validate_pipelines(root, report)
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
