#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path


EXPECTED_TASK_ID = re.compile(r"^[A-K][0-9_]+$")


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


def normalize_topic(topic: str) -> str:
    normalized = re.sub(r"[^a-z0-9]+", "-", topic.strip().lower())
    return normalized.strip("-")


def read_contract(repo_root: Path) -> str:
    contract_path = repo_root / "standards" / "research-workflow-contract.yaml"
    return contract_path.read_text(encoding="utf-8")


def load_artifact_root(contract: str) -> str:
    artifacts = extract_top_level_section(contract, "artifacts")
    return parse_yaml_scalar(artifacts, "root", indent=2) or "RESEARCH/[topic]/"


def load_task_outputs(contract: str, task_id: str) -> list[str]:
    task_section = extract_top_level_section(contract, "task_catalog")
    match = re.search(
        rf"^\s{{2}}{re.escape(task_id)}:\n((?:^\s{{4}}.*\n?)+)",
        task_section,
        flags=re.MULTILINE,
    )
    if not match:
        raise ValueError(f"Task ID not found in contract task_catalog: {task_id}")
    outputs_section = extract_nested_section(match.group(1), "outputs", indent=4)
    outputs = parse_yaml_list(outputs_section, item_indent=6)
    if not outputs:
        raise ValueError(f"Task {task_id} has no outputs configured in contract")
    return outputs


def load_task_dependencies(contract: str, task_id: str) -> dict[str, list[str]]:
    dependency_section = extract_top_level_section(contract, "dependency_catalog")
    if not dependency_section:
        return {
            "prerequisites_all": [],
            "prerequisites_any": [],
            "recommended_prerequisites": [],
            "recommended_next": [],
        }
    match = re.search(
        rf"^\s{{2}}{re.escape(task_id)}:\n((?:^\s{{4}}.*\n?)+)",
        dependency_section,
        flags=re.MULTILINE,
    )
    if not match:
        return {
            "prerequisites_all": [],
            "prerequisites_any": [],
            "recommended_prerequisites": [],
            "recommended_next": [],
        }
    block = match.group(1)
    return {
        "prerequisites_all": parse_yaml_list(
            extract_nested_section(block, "prerequisites_all", indent=4),
            item_indent=6,
        ),
        "prerequisites_any": parse_yaml_list(
            extract_nested_section(block, "prerequisites_any", indent=4),
            item_indent=6,
        ),
        "recommended_prerequisites": parse_yaml_list(
            extract_nested_section(block, "recommended_prerequisites", indent=4),
            item_indent=6,
        ),
        "recommended_next": parse_yaml_list(
            extract_nested_section(block, "recommended_next", indent=4),
            item_indent=6,
        ),
    }


def build_task_plan(contract: str, task_id: str) -> dict[str, object]:
    visited: set[str] = set()
    visiting: set[str] = set()
    ordered: list[str] = []

    def dfs(node: str) -> None:
        if node in visited:
            return
        if node in visiting:
            raise ValueError(f"Cycle detected in prerequisites_all near task: {node}")
        visiting.add(node)
        spec = load_task_dependencies(contract, node)
        for prereq in spec.get("prerequisites_all", []):
            if EXPECTED_TASK_ID.match(prereq):
                dfs(prereq)
        visiting.remove(node)
        visited.add(node)
        ordered.append(node)

    dfs(task_id)

    any_of_requirements: list[dict[str, object]] = []
    for node in ordered:
        spec = load_task_dependencies(contract, node)
        any_of = [x for x in spec.get("prerequisites_any", []) if EXPECTED_TASK_ID.match(x)]
        if any_of:
            any_of_requirements.append({"task": node, "any_of": any_of})

    return {
        "task_id": task_id,
        "requires_all_order": ordered,
        "root_dependencies": load_task_dependencies(contract, task_id),
        "any_of_requirements": any_of_requirements,
    }


def is_nonempty_text(path: Path) -> bool:
    try:
        content = path.read_text(encoding="utf-8")
    except OSError:
        return False
    return bool(content.strip())


def check_csv_basic(path: Path, report: ValidationReport) -> None:
    try:
        with path.open("r", encoding="utf-8", newline="") as handle:
            reader = csv.reader(handle)
            rows = list(reader)
    except OSError as exc:
        report.errors.append(f"Failed to read CSV {path}: {exc}")
        print(f"[FAIL] Failed to read CSV {path}: {exc}")
        return
    report.warn(
        len(rows) >= 2,
        f"{path.name} has at least 1 data row",
        f"{path.name} should include header + at least 1 data row",
    )
    if not rows:
        return
    header = [cell.strip().lower() for cell in rows[0] if cell.strip()]
    report.warn(
        len(header) >= 2,
        f"{path.name} header has >=2 columns",
        f"{path.name} header should have >=2 columns",
    )


def check_bibtex_basic(path: Path, report: ValidationReport) -> None:
    try:
        content = path.read_text(encoding="utf-8")
    except OSError as exc:
        report.errors.append(f"Failed to read BibTeX {path}: {exc}")
        print(f"[FAIL] Failed to read BibTeX {path}: {exc}")
        return
    keys = re.findall(r"@\w+\s*\{\s*([^,\s]+)\s*,", content)
    report.warn(
        bool(keys),
        f"{path.name} contains BibTeX entries",
        f"{path.name} should contain at least 1 BibTeX entry",
    )
    duplicates = sorted({key for key in keys if keys.count(key) > 1})
    report.check(
        not duplicates,
        f"{path.name} has no duplicate citekeys",
        f"{path.name} has duplicate citekeys: {', '.join(duplicates)}",
    )


def check_claims_evidence_map(path: Path, report: ValidationReport) -> None:
    try:
        content = path.read_text(encoding="utf-8")
    except OSError as exc:
        report.errors.append(f"Failed to read claims-evidence map {path}: {exc}")
        print(f"[FAIL] Failed to read claims-evidence map {path}: {exc}")
        return
    table_rows = [
        line
        for line in content.splitlines()
        if line.strip().startswith("|") and line.count("|") >= 3
    ]
    report.warn(
        len(table_rows) >= 3,
        f"{path.name} contains a populated table",
        f"{path.name} should include a claim-evidence table with >=1 data row",
    )


def check_prisma_flow(path: Path, report: ValidationReport) -> None:
    try:
        content = path.read_text(encoding="utf-8")
    except OSError as exc:
        report.errors.append(f"Failed to read PRISMA flow {path}: {exc}")
        print(f"[FAIL] Failed to read PRISMA flow {path}: {exc}")
        return
    numbers = re.findall(r"\b\d+\b", content)
    report.warn(
        bool(numbers),
        f"{path.name} includes numeric counts",
        f"{path.name} should include numeric counts for PRISMA tracking",
    )


def validate_task_outputs(
    contract: str,
    project_root: Path,
    task_id: str,
    report: ValidationReport,
) -> None:
    outputs = load_task_outputs(contract, task_id)
    for rel_path in outputs:
        rel_norm = rel_path.strip()
        if not rel_norm:
            continue
        target = project_root / rel_norm
        if rel_norm.endswith("/"):
            report.check(
                target.exists() and target.is_dir(),
                f"{task_id} output dir exists: {rel_norm}",
                f"{task_id} missing output dir: {rel_norm}",
            )
            if target.exists() and target.is_dir():
                try:
                    has_children = any(target.iterdir())
                except OSError:
                    has_children = False
                report.warn(
                    has_children,
                    f"{task_id} output dir not empty: {rel_norm}",
                    f"{task_id} output dir appears empty: {rel_norm}",
                )
            continue

        report.check(
            target.exists() and target.is_file(),
            f"{task_id} output file exists: {rel_norm}",
            f"{task_id} missing output file: {rel_norm}",
        )
        if target.exists() and target.is_file():
            report.warn(
                is_nonempty_text(target),
                f"{task_id} output file non-empty: {rel_norm}",
                f"{task_id} output file is empty/whitespace: {rel_norm}",
            )


def validate_content_quality(project_root: Path, report: ValidationReport) -> None:
    # Heuristic, deterministic checks (warnings by default).
    checks = {
        "search_results.csv": check_csv_basic,
        "bibliography.bib": check_bibtex_basic,
        "manuscript/claims_evidence_map.md": check_claims_evidence_map,
        "screening/prisma_flow.md": check_prisma_flow,
    }
    for rel_path, checker in checks.items():
        target = project_root / rel_path
        if target.exists() and target.is_file():
            checker(target, report)


def task_outputs_exist(contract: str, project_root: Path, task_id: str) -> tuple[bool, list[str]]:
    try:
        outputs = load_task_outputs(contract, task_id)
    except ValueError:
        return False, ["<task not found in contract>"]
    missing: list[str] = []
    for rel_path in outputs:
        rel_norm = rel_path.strip()
        if not rel_norm:
            continue
        target = project_root / rel_norm
        if not target.exists():
            missing.append(rel_norm)
    return not missing, missing


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Deterministic validator for RESEARCH/[topic]/ artifacts against the workflow contract."
    )
    parser.add_argument("--cwd", required=True, type=Path, help="Project working directory")
    parser.add_argument("--topic", required=True, help="Topic slug/name (maps to RESEARCH/[topic]/)")
    parser.add_argument("--task-id", required=True, help="Canonical Task ID to validate (e.g. H1)")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Treat warnings as failures (use for submission gates)",
    )
    parser.add_argument(
        "--json",
        dest="json_output",
        action="store_true",
        help="Emit a JSON summary at the end (in addition to logs)",
    )
    args = parser.parse_args()

    report = ValidationReport()
    repo_root = Path(__file__).resolve().parents[1]
    cwd = args.cwd if args.cwd.is_absolute() else (Path.cwd() / args.cwd)
    task_id = args.task_id.strip().upper()
    topic = normalize_topic(args.topic)

    report.check(
        bool(EXPECTED_TASK_ID.match(task_id)),
        "Task ID is canonical format",
        f"Task ID must match pattern [A-K][0-9_]+; got: {args.task_id}",
    )
    contract = read_contract(repo_root)
    artifact_root = load_artifact_root(contract)
    project_root = cwd / artifact_root.replace("[topic]", topic)
    report.check(
        project_root.exists() and project_root.is_dir(),
        f"Project root exists: {project_root}",
        f"Project root missing: {project_root}",
    )
    if report.errors:
        return finalize(report, strict=args.strict, json_output=args.json_output)

    try:
        plan = build_task_plan(contract, task_id)
    except ValueError as exc:
        report.errors.append(str(exc))
        print(f"[FAIL] {exc}")
        return finalize(report, strict=args.strict, json_output=args.json_output)

    ordered = [str(x) for x in plan.get("requires_all_order", [])]
    report.check(
        ordered and ordered[-1] == task_id,
        "Task plan resolved prerequisites_all chain",
        "Failed to resolve task plan prerequisites_all chain",
    )

    missing_any_of: list[str] = []
    for item in plan.get("any_of_requirements", []):
        node = str(item.get("task", "")).strip()
        options = [str(x).strip() for x in item.get("any_of", []) if str(x).strip()]
        satisfied_by = None
        for opt in options:
            ok, _missing = task_outputs_exist(contract, project_root, opt)
            if ok:
                satisfied_by = opt
                break
        report.check(
            bool(satisfied_by),
            f"{node} prerequisites_any satisfied",
            f"{node} prerequisites_any missing (need any of: {', '.join(options)})",
        )
        if not satisfied_by:
            missing_any_of.append(node)

    for node in ordered:
        validate_task_outputs(contract, project_root, node, report)

    root_deps = plan.get("root_dependencies", {})
    recommended_prereq = [
        str(x).strip()
        for x in root_deps.get("recommended_prerequisites", [])
        if str(x).strip()
    ]
    for rec in recommended_prereq:
        try:
            rec_outputs = load_task_outputs(contract, rec)
        except ValueError:
            continue
        missing = [rel for rel in rec_outputs if not (project_root / rel).exists()]
        report.warn(
            not missing,
            f"Recommended prerequisite complete: {rec}",
            f"Recommended prerequisite incomplete: {rec} (missing {', '.join(missing)})",
        )

    validate_content_quality(project_root, report)

    return finalize(report, strict=args.strict, json_output=args.json_output)


def finalize(report: ValidationReport, strict: bool, json_output: bool) -> int:
    blocked_by_warnings = bool(strict and report.warnings)
    effective_errors = list(report.errors)
    if blocked_by_warnings:
        effective_errors.extend(f"[strict-warning] {warning}" for warning in report.warnings)
    verdict = "BLOCK" if effective_errors else "PASS"
    exit_code = 1 if verdict == "BLOCK" else 0

    print("")
    print("=== Project Artifact Validator Summary ===")
    print(f"Verdict: {verdict}")
    print(f"Passed checks: {report.passed}")
    print(f"Warnings: {len(report.warnings)}")
    print(f"Errors: {len(report.errors)}")
    if strict:
        print(f"Strict mode: {'ON (warnings are blocking)' if blocked_by_warnings else 'ON'}")

    if json_output:
        payload = {
            "verdict": verdict,
            "passed": report.passed,
            "strict": strict,
            "blocked_by_warnings": blocked_by_warnings,
            "warnings": report.warnings,
            "errors": report.errors,
            "effective_errors": effective_errors,
        }
        print("")
        print(json.dumps(payload, ensure_ascii=False, indent=2))

    return exit_code


if __name__ == "__main__":
    sys.exit(main())
