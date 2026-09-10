#!/usr/bin/env python3
"""Simple eval runner for qiongli golden tests.

Usage:
    python evals/runner/run_eval.py evals/cases/sr-social-media-mental-health.yaml

This runner validates that skill outputs match expected structure.
It does NOT execute skills — it checks existing outputs against expectations.
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import re
import sys
import tempfile
import xml.etree.ElementTree as ET
from collections import Counter
from pathlib import Path
from typing import Any

import yaml


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tooling.scripts.audit_citation_risk import (  # noqa: E402
    CITABLE_EVIDENCE_TYPES,
    audit_citation_integrity,
)
from tooling.scripts.validate_capability_contract import (  # noqa: E402
    _load_json,
    validate_instance,
)


SUPPORTED_ASSERTIONS = {
    "contains_all",
    "contains_any",
    "schema",
    "field_constraint",
    "count_conservation",
    "cross_artifact_consistency",
    "locator_syntax",
    "citation_identity",
    "file_digest",
}
ASSERTION_KEYS = {
    "contains_all": {"type", "values"},
    "contains_any": {"type", "values"},
    "schema": {"type", "schema"},
    "field_constraint": {"type", "field", "allowed_values"},
    "count_conservation": {"type", "total", "parts"},
    "cross_artifact_consistency": {
        "type",
        "field",
        "other_artifact",
        "other_field",
        "relation",
    },
    "locator_syntax": {"type", "field"},
    "citation_identity": {"type", "bibliography"},
    "file_digest": {"type", "sha256"},
}
SUPPORTED_ARTIFACT_SUFFIXES = (".md", ".py", ".r")
SUPPORTED_SCHEMA_CONSTRAINTS = {
    "$ref",
    "oneOf",
    "const",
    "enum",
    "type",
    "required",
    "properties",
    "additionalProperties",
    "minProperties",
    "minItems",
    "maxItems",
    "uniqueItems",
    "items",
    "minLength",
    "maxLength",
    "pattern",
    "format",
    "minimum",
    "maximum",
}
LOCATOR_PATTERN = re.compile(
    r"(?:p\.\s*\d+|pp\.\s*\d+\s*[-–]\s*\d+|"
    r"[A-Za-z0-9][A-Za-z0-9_.-]*:\S(?:.*\S)?)",
    flags=re.IGNORECASE,
)
RECEIPT_VERSION = "1.0"
TRUTH_COUNTERS = (
    "required_missing",
    "executed_assertions",
    "failed_assertions",
    "blocked_assertions",
    "unknown_validation_types",
)
PORTABLE_MESSAGES = {
    "assertion-passed": "Assertion passed.",
    "assertion-config-invalid": "Assertion configuration is invalid.",
    "assertion-type-unknown": "Assertion type is unknown.",
    "assertion-evidence-unavailable": "Assertion evidence is unavailable.",
    "case-load-failed": "Eval case could not be loaded.",
    "case-contract-invalid": "Eval case contract is invalid.",
    "schema-version-unsupported": "Eval case schema version is unsupported.",
    "expected-outputs-invalid": "Expected outputs contract is invalid.",
    "no-assertions-executed": "No assertions executed.",
    "output-contract-invalid": "Expected output contract is invalid.",
    "artifact-path-invalid": "Artifact path is invalid.",
    "required-artifact-missing": "Required artifact is missing.",
    "required-artifact-empty": "Required artifact is empty.",
    "optional-artifact-missing": "Optional artifact is missing.",
    "optional-artifact-empty": "Optional artifact is empty.",
    "artifact-unreadable": "Artifact is unreadable.",
    "artifact-kind-invalid": "Artifact is not a regular file.",
}


def load_case(path: str | Path) -> object:
    with Path(path).open(encoding="utf-8") as handle:
        return yaml.safe_load(handle)


def _clean_string(value: object, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{label} must be a non-empty string")
    return value.strip()


def _clean_string_list(value: object, label: str) -> list[str]:
    if not isinstance(value, list) or not value:
        raise ValueError(f"{label} must be a non-empty list of strings")
    cleaned = [_clean_string(item, label) for item in value]
    if len(cleaned) != len(set(cleaned)):
        raise ValueError(f"{label} must not contain duplicates")
    return cleaned


def _normalize_assertion(
    assertion: object, index: int
) -> tuple[dict[str, Any] | None, str | None, bool]:
    if not isinstance(assertion, dict):
        return None, f"assertion {index} must be an object", False

    raw_type = assertion.get("type")
    if not isinstance(raw_type, str) or not raw_type.strip():
        return None, f"assertion {index} requires a type", False
    assertion_type = raw_type.strip()
    if assertion_type not in SUPPORTED_ASSERTIONS:
        return None, f"assertion {index} has unknown type: {assertion_type}", True

    expected_keys = ASSERTION_KEYS[assertion_type]
    missing = expected_keys - assertion.keys()
    extra = assertion.keys() - expected_keys
    if missing or extra:
        details = []
        if missing:
            details.append("missing " + ", ".join(sorted(missing)))
        if extra:
            details.append("unexpected " + ", ".join(sorted(map(str, extra))))
        return None, f"assertion {index} fields: {'; '.join(details)}", False

    try:
        normalized: dict[str, Any] = {"type": assertion_type}
        if assertion_type in {"contains_all", "contains_any"}:
            normalized["values"] = _clean_string_list(assertion["values"], "values")
        elif assertion_type == "schema":
            normalized["schema"] = _clean_string(assertion["schema"], "schema")
        elif assertion_type == "field_constraint":
            normalized["field"] = _clean_string(assertion["field"], "field")
            normalized["allowed_values"] = _clean_string_list(
                assertion["allowed_values"], "allowed_values"
            )
        elif assertion_type == "count_conservation":
            normalized["total"] = _clean_string(assertion["total"], "total")
            normalized["parts"] = _clean_string_list(assertion["parts"], "parts")
            if normalized["total"] in normalized["parts"]:
                raise ValueError("total must not also appear in parts")
        elif assertion_type == "cross_artifact_consistency":
            for key in ("field", "other_field"):
                value = assertion[key]
                normalized[key] = _clean_string_list(value, key) if isinstance(value, list) else [_clean_string(value, key)]
            if len(normalized["field"]) != len(normalized["other_field"]):
                raise ValueError("cross-artifact fields must have equal lengths")
            normalized["other_artifact"] = _clean_string(
                assertion["other_artifact"], "other_artifact"
            )
            relation = _clean_string(assertion["relation"], "relation")
            if relation not in {"equal", "subset"}:
                raise ValueError("relation must be equal or subset")
            normalized["relation"] = relation
        elif assertion_type == "locator_syntax":
            normalized["field"] = _clean_string(assertion["field"], "field")
        elif assertion_type == "citation_identity":
            normalized["bibliography"] = _clean_string(
                assertion["bibliography"], "bibliography"
            )
        else:
            digest = _clean_string(assertion["sha256"], "sha256").lower()
            if re.fullmatch(r"[0-9a-f]{64}", digest) is None:
                raise ValueError(
                    "sha256 must contain exactly 64 hexadecimal characters"
                )
            normalized["sha256"] = digest
    except ValueError as exc:
        return None, f"assertion {index}: {exc}", False
    return normalized, None, False


def _resolve_relative(root: Path, reference: str) -> Path:
    relative = Path(reference)
    if relative.is_absolute():
        raise ValueError(f"path must be relative: {reference}")
    resolved_root = root.resolve()
    resolved = (resolved_root / relative).resolve()
    if not resolved.is_relative_to(resolved_root):
        raise ValueError(f"path escapes its root: {reference}")
    return resolved


def _require_file(path: Path, label: str) -> Path:
    if not path.is_file():
        raise ValueError(f"{label} not found: {path.name}")
    return path


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _read_csv_rows(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        raw_fields = reader.fieldnames
        if not raw_fields:
            raise ValueError(f"CSV has no header: {path.name}")
        fields = [_clean_string(field, "CSV header") for field in raw_fields]
        if len(fields) != len(set(fields)):
            raise ValueError(f"CSV has duplicate headers: {path.name}")

        rows: list[dict[str, str]] = []
        for row_number, row in enumerate(reader, start=2):
            if None in row or any(value is None for value in row.values()):
                raise ValueError(
                    f"CSV row {row_number} has the wrong number of fields"
                )
            rows.append(
                {
                    field: str(row[raw_field]).strip()
                    for raw_field, field in zip(raw_fields, fields, strict=True)
                }
            )
    if not rows:
        raise ValueError(f"CSV has no data rows: {path.name}")
    return rows


def _read_csv_column(
    path: Path, field: str, *, allow_blank: bool = False
) -> list[str]:
    rows = _read_csv_rows(path)
    if field not in rows[0]:
        raise ValueError(f"CSV field not found: {field}")
    values = [row[field] for row in rows]
    if allow_blank:
        values = [value for value in values if value]
        if not values:
            raise ValueError(f"CSV field has no applicable values: {field}")
    elif any(not value for value in values):
        raise ValueError(f"CSV field contains an empty value: {field}")
    return values


def _read_csv_keys(path: Path, fields: list[str], *, allow_blank: bool = False) -> Counter:
    rows = _read_csv_rows(path)
    if any(field not in rows[0] for field in fields):
        raise ValueError("CSV key field not found")
    keys = [tuple(row[field] for field in fields) for row in rows]
    if not allow_blank and any(not value for key in keys for value in key):
        raise ValueError("CSV key contains an empty value")
    return Counter(keys)


def _extract_count(content: str, label: str) -> int:
    pattern = re.compile(
        rf"^\s*(?:[-*+]\s+)?(?:\*\*)?{re.escape(label)}\s*:\s*"
        rf"n\s*=\s*(\d+)\s*(?:\*\*)?\s*$",
        flags=re.IGNORECASE | re.MULTILINE,
    )
    matches = pattern.findall(content)
    if len(matches) != 1:
        raise ValueError(f"count label must occur exactly once: {label}")
    return int(matches[0])


def _load_structured_artifact(path: Path) -> object:
    suffix = path.suffix.casefold()
    if suffix == ".json":
        return _load_json(path)
    if suffix in {".yaml", ".yml"}:
        return yaml.safe_load(_read_text(path))
    if suffix == ".csv":
        return _read_csv_rows(path)
    raise ValueError("schema assertions require a JSON, YAML or CSV artifact")


def _check_assertion(
    artifact_path: Path,
    assertion: dict[str, Any],
    *,
    case_root: Path,
    output_root: Path,
) -> list[str]:
    assertion_type = assertion["type"]
    if assertion_type in {"contains_all", "contains_any"}:
        folded = _read_text(artifact_path).casefold()
        values = assertion["values"]
        if assertion_type == "contains_all":
            return [
                f"Missing: {value}"
                for value in values
                if value.casefold() not in folded
            ]
        if any(value.casefold() in folded for value in values):
            return []
        return [f"Missing any of: {', '.join(values)}"]

    if assertion_type == "schema":
        schema_path = _require_file(
            _resolve_relative(case_root, assertion["schema"]), "schema"
        )
        schema = _load_json(schema_path)
        if not isinstance(schema, dict):
            raise ValueError("schema must be a JSON object")
        if not SUPPORTED_SCHEMA_CONSTRAINTS.intersection(schema):
            raise ValueError("schema has no supported constraints")
        value = _load_structured_artifact(artifact_path)
        return [
            f"Schema: {failure}" for failure in validate_instance(value, schema)
        ]

    if assertion_type == "field_constraint":
        values = _read_csv_column(artifact_path, assertion["field"])
        allowed = set(assertion["allowed_values"])
        invalid = sorted({value for value in values if value not in allowed})
        if not invalid:
            return []
        return [
            f"Disallowed {assertion['field']} value(s): {', '.join(invalid)}"
        ]

    if assertion_type == "count_conservation":
        content = _read_text(artifact_path)
        total = _extract_count(content, assertion["total"])
        parts = [
            _extract_count(content, label) for label in assertion["parts"]
        ]
        if total == sum(parts):
            return []
        return [
            f"Count mismatch: {assertion['total']}={total}, parts sum={sum(parts)}"
        ]

    if assertion_type == "cross_artifact_consistency":
        primary = _read_csv_keys(artifact_path, assertion["field"])
        other_path = _require_file(
            _resolve_relative(output_root, assertion["other_artifact"]),
            "other artifact",
        )
        relation = assertion["relation"]
        other = _read_csv_keys(other_path, assertion["other_field"],
                               allow_blank=relation == "subset" and len(assertion["other_field"]) > 1)
        matches = (
            primary == other if relation == "equal" else not (primary - other)
        )
        if matches:
            return []
        return [f"Cross-artifact {relation} relation failed"]

    if assertion_type == "locator_syntax":
        values = _read_csv_column(
            artifact_path, assertion["field"], allow_blank=True
        )
        invalid = sorted(
            {
                value
                for value in values
                if LOCATOR_PATTERN.fullmatch(value) is None
            }
        )
        if not invalid:
            return []
        return [f"Invalid locator(s): {', '.join(invalid)}"]

    if assertion_type == "citation_identity":
        rows = _read_csv_rows(artifact_path)
        required_fields = {"evidence_type", "source_id"}
        if not required_fields.issubset(rows[0]):
            missing = ", ".join(
                sorted(required_fields - rows[0].keys())
            )
            raise ValueError(f"citation ledger missing field(s): {missing}")
        if not any(
            row["evidence_type"] in CITABLE_EVIDENCE_TYPES for row in rows
        ):
            raise ValueError(
                "citation ledger has no citable paper/theory rows"
            )
        bibliography = _require_file(
            _resolve_relative(output_root, assertion["bibliography"]),
            "bibliography",
        )
        result = audit_citation_integrity(artifact_path, bibliography)
        return [
            f"Citation identity: {error}" for error in result.errors
        ]

    actual = hashlib.sha256(artifact_path.read_bytes()).hexdigest()
    if actual == assertion["sha256"]:
        return []
    return [
        f"SHA-256 mismatch: expected {assertion['sha256']}, got {actual}"
    ]


def _portable_message(reason_code: str) -> str:
    if reason_code in PORTABLE_MESSAGES:
        return PORTABLE_MESSAGES[reason_code]
    if reason_code.endswith("-failed"):
        return "Assertion failed."
    return "Evaluation blocked."


def _portable_evidence(root: Path, path: Path, role: str) -> dict[str, str] | None:
    try:
        resolved_root = root.resolve()
        resolved_path = path.resolve()
        relative = resolved_path.relative_to(resolved_root)
    except (OSError, RuntimeError, ValueError):
        return None

    evidence = {"role": role, "path": relative.as_posix()}
    try:
        if resolved_path.is_file():
            evidence["sha256"] = hashlib.sha256(resolved_path.read_bytes()).hexdigest()
    except OSError:
        pass
    return evidence


def _reference_evidence(
    assertion: dict[str, Any], *, case_root: Path, output_root: Path
) -> list[dict[str, str]]:
    references = {
        "schema": ("schema", case_root, "schema"),
        "cross_artifact_consistency": (
            "other_artifact",
            output_root,
            "other-artifact",
        ),
        "citation_identity": ("bibliography", output_root, "bibliography"),
    }
    reference = references.get(assertion["type"])
    if reference is None:
        return []
    field, root, role = reference
    try:
        path = _resolve_relative(root, assertion[field])
    except (KeyError, ValueError):
        return []
    evidence = _portable_evidence(root, path, role)
    return [evidence] if evidence is not None else []


def _new_result(case_file: Path) -> dict[str, Any]:
    return {
        "receipt_version": RECEIPT_VERSION,
        "case": {
            "id": "unknown",
            "pipeline": None,
            "schema_version": None,
            "status": "blocked",
            "reason_code": "case-contract-invalid",
        },
        "summary": {counter: 0 for counter in TRUTH_COUNTERS},
        "assertions": [],
        "_case_file": case_file,
        "_display_lines": [],
        "_output_order": {},
        "_success": False,
    }


def _add_outcome(
    result: dict[str, Any],
    *,
    output_id: str,
    index: int | None,
    assertion_type: str,
    status: str,
    reason_code: str,
    evidence: list[dict[str, str]] | None = None,
) -> None:
    result["assertions"].append(
        {
            "output_id": output_id,
            "index": index,
            "type": assertion_type,
            "status": status,
            "reason_code": reason_code,
            "message": _portable_message(reason_code),
            "evidence": evidence or [],
        }
    )


def _block_case(result: dict[str, Any], reason_code: str) -> dict[str, Any]:
    result["summary"]["blocked_assertions"] = 1
    result["case"]["status"] = "blocked"
    result["case"]["reason_code"] = reason_code
    case_file = result["_case_file"]
    evidence = _portable_evidence(case_file.parent, case_file, "case")
    _add_outcome(
        result,
        output_id="case",
        index=None,
        assertion_type="case-contract",
        status="blocked",
        reason_code=reason_code,
        evidence=[evidence] if evidence is not None else [],
    )
    return result


def _artifact_outcomes(
    result: dict[str, Any],
    output_id: str,
    assertions: list[tuple[int, dict[str, Any]]],
    *,
    status: str,
    reason_code: str,
    evidence: list[dict[str, str]],
) -> None:
    for index, assertion in assertions:
        _add_outcome(
            result,
            output_id=output_id,
            index=index,
            assertion_type=assertion["type"],
            status=status,
            reason_code=reason_code,
            evidence=evidence,
        )


def _finish_result(result: dict[str, Any]) -> dict[str, Any]:
    summary = result["summary"]
    if summary["executed_assertions"] == 0 and not any(
        outcome["status"] in {"fail", "blocked"}
        for outcome in result["assertions"]
    ):
        case_file = result["_case_file"]
        evidence = _portable_evidence(case_file.parent, case_file, "case")
        _add_outcome(
            result,
            output_id="case",
            index=None,
            assertion_type="case-contract",
            status="blocked",
            reason_code="no-assertions-executed",
            evidence=[evidence] if evidence is not None else [],
        )

    output_order = result["_output_order"]
    result["assertions"].sort(
        key=lambda outcome: (
            len(output_order)
            if outcome["type"] == "case-contract"
            else output_order.get(outcome["output_id"], len(output_order)),
            -1 if outcome["index"] is None else outcome["index"],
        )
    )

    success = (
        summary["required_missing"] == 0
        and summary["executed_assertions"] > 0
        and summary["failed_assertions"] == 0
        and summary["blocked_assertions"] == 0
        and summary["unknown_validation_types"] == 0
    )
    result["_success"] = success
    if success:
        result["case"]["status"] = "pass"
        result["case"]["reason_code"] = "case-passed"
    elif summary["unknown_validation_types"]:
        result["case"]["status"] = "blocked"
        result["case"]["reason_code"] = "assertion-type-unknown"
    elif summary["blocked_assertions"]:
        result["case"]["status"] = "blocked"
        result["case"]["reason_code"] = "assertion-blocked"
    elif summary["executed_assertions"] == 0:
        result["case"]["status"] = "blocked"
        result["case"]["reason_code"] = "no-assertions-executed"
    elif summary["required_missing"]:
        result["case"]["status"] = "fail"
        result["case"]["reason_code"] = "required-artifact-missing"
    else:
        result["case"]["status"] = "fail"
        result["case"]["reason_code"] = "assertion-failed"
    return result


def _evaluate_case(
    case_path: str | Path, output_dir: str | Path | None = None
) -> dict[str, Any]:
    case_file = Path(case_path)
    result = _new_result(case_file)
    lines = result["_display_lines"]
    try:
        case = load_case(case_file)
    except (OSError, UnicodeError, yaml.YAMLError) as exc:
        lines.extend(("", f"[FAIL] Unable to load eval case: {exc}"))
        return _block_case(result, "case-load-failed")

    if not isinstance(case, dict):
        lines.extend(("", "[FAIL] Eval case must be a YAML object"))
        return _block_case(result, "case-contract-invalid")

    case_id = case.get("case_id")
    pipeline = case.get("pipeline")
    if not isinstance(case_id, str) or not case_id.strip():
        lines.extend(("", "[FAIL] Eval case requires a non-empty case_id"))
        return _block_case(result, "case-contract-invalid")
    if not isinstance(pipeline, str) or not pipeline.strip():
        result["case"]["id"] = case_id.strip()
        lines.extend(
            ("", f"[FAIL] Eval case {case_id} requires a non-empty pipeline")
        )
        return _block_case(result, "case-contract-invalid")

    case_id = case_id.strip()
    pipeline = pipeline.strip()
    schema_version = case.get("schema_version")
    result["case"].update(
        {
            "id": case_id,
            "pipeline": pipeline,
            "schema_version": schema_version if isinstance(schema_version, str) else None,
        }
    )
    lines.extend(("", "=" * 60, f"Eval Case: {case_id}", f"Pipeline:  {pipeline}", "=" * 60))

    if schema_version != "1.0":
        lines.append(
            "  [case] BLOCKED — unsupported schema_version: "
            f"{schema_version!r}"
        )
        return _block_case(result, "schema-version-unsupported")

    expected_outputs = case.get("expected_outputs")
    if not isinstance(expected_outputs, dict) or not expected_outputs:
        lines.append(
            "  [case] BLOCKED — expected_outputs must be a non-empty object"
        )
        return _block_case(result, "expected-outputs-invalid")

    case_input = case.get("input")
    topic = case_input.get("topic") if isinstance(case_input, dict) else None
    if not isinstance(topic, str) or not topic.strip():
        lines.append(
            "  [case] BLOCKED — input.topic must be a non-empty string"
        )
        return _block_case(result, "case-contract-invalid")

    if output_dir is None:
        topic_slug = re.sub(r"[^a-z0-9]+", "_", topic.lower())[:40]
        output_dir = Path("RESEARCH") / topic_slug

    output_root = Path(output_dir).resolve()
    case_root = case_file.resolve().parent
    total = len(expected_outputs)
    passed = 0
    failed = 0
    skipped = 0
    required_missing = 0
    executed_assertions = 0
    failed_assertions = 0
    blocked_assertions = 0
    unknown_validation_types = 0

    for output_position, (skill_id, expected) in enumerate(expected_outputs.items()):
        output_id = str(skill_id)
        result["_output_order"].setdefault(output_id, output_position)
        if not isinstance(expected, dict):
            failed += 1
            blocked_assertions += 1
            _add_outcome(
                result,
                output_id=output_id,
                index=None,
                assertion_type="output-contract",
                status="blocked",
                reason_code="output-contract-invalid",
            )
            lines.append(f"  [{skill_id}] BLOCKED — expected output must be an object")
            continue

        fatal_errors: list[str] = []
        blocked_reasons: list[str] = []
        artifact = expected.get("artifact")
        required = expected.get("required")
        raw_assertions = expected.get("assertions")

        if "must_contain" in expected or "validation" in expected:
            blocked_reasons.append(
                "legacy must_contain/validation fields are unsupported"
            )
            blocked_assertions += 1
            _add_outcome(
                result,
                output_id=output_id,
                index=None,
                assertion_type="output-contract",
                status="blocked",
                reason_code="output-contract-invalid",
            )
        if not isinstance(artifact, str) or not artifact.strip():
            fatal_errors.append(
                "artifact must be a non-empty relative path"
            )
            blocked_assertions += 1
            _add_outcome(
                result,
                output_id=output_id,
                index=None,
                assertion_type="output-contract",
                status="blocked",
                reason_code="output-contract-invalid",
            )
        else:
            artifact = artifact.strip()
        if type(required) is not bool:
            fatal_errors.append("required must be true or false")
            blocked_assertions += 1
            _add_outcome(
                result,
                output_id=output_id,
                index=None,
                assertion_type="output-contract",
                status="blocked",
                reason_code="output-contract-invalid",
            )

        assertions: list[tuple[int, dict[str, Any]]] = []
        if not isinstance(raw_assertions, list) or not raw_assertions:
            fatal_errors.append("assertions must be a non-empty list")
            blocked_assertions += 1
            _add_outcome(
                result,
                output_id=output_id,
                index=None,
                assertion_type="output-contract",
                status="blocked",
                reason_code="output-contract-invalid",
            )
        else:
            for index, assertion in enumerate(raw_assertions):
                normalized, error, unknown = _normalize_assertion(
                    assertion, index
                )
                if error is not None:
                    blocked_reasons.append(error)
                    blocked_assertions += 1
                    unknown_validation_types += int(unknown)
                    assertion_type = (
                        assertion.get("type")
                        if isinstance(assertion, dict)
                        and assertion.get("type") in SUPPORTED_ASSERTIONS
                        else "unknown"
                    )
                    _add_outcome(
                        result,
                        output_id=output_id,
                        index=index,
                        assertion_type=assertion_type,
                        status="blocked",
                        reason_code=(
                            "assertion-type-unknown"
                            if unknown
                            else "assertion-config-invalid"
                        ),
                    )
                elif normalized is not None:
                    assertions.append((index, normalized))

        if fatal_errors:
            blocked_assertions += len(assertions)
            failed += 1
            _artifact_outcomes(
                result,
                output_id,
                assertions,
                status="blocked",
                reason_code="output-contract-invalid",
                evidence=[],
            )
            lines.append(f"  [{skill_id}] BLOCKED")
            for error in fatal_errors + blocked_reasons:
                lines.append(f"    x {error}")
            continue

        try:
            artifact_path = _resolve_relative(output_root, artifact)
        except ValueError as exc:
            blocked_assertions += len(assertions)
            failed += 1
            _artifact_outcomes(
                result,
                output_id,
                assertions,
                status="blocked",
                reason_code="artifact-path-invalid",
                evidence=[],
            )
            lines.append(f"  [{skill_id}] BLOCKED")
            for error in blocked_reasons + [str(exc)]:
                lines.append(f"    x {error}")
            continue

        if not artifact_path.exists():
            evidence = _portable_evidence(output_root, artifact_path, "artifact")
            portable_evidence = [evidence] if evidence is not None else []
            if blocked_reasons:
                failed += 1
                _artifact_outcomes(
                    result,
                    output_id,
                    assertions,
                    status="blocked",
                    reason_code="assertion-evidence-unavailable",
                    evidence=portable_evidence,
                )
                lines.append(f"  [{skill_id}] BLOCKED")
                for error in blocked_reasons:
                    lines.append(f"    x {error}")
            elif required:
                failed += 1
                required_missing += 1
                _artifact_outcomes(
                    result,
                    output_id,
                    assertions,
                    status="fail",
                    reason_code="required-artifact-missing",
                    evidence=portable_evidence,
                )
                lines.append(
                    f"  [{skill_id}] FAIL — required artifact not found: "
                    f"{artifact}"
                )
            else:
                skipped += 1
                _artifact_outcomes(
                    result,
                    output_id,
                    assertions,
                    status="skip",
                    reason_code="optional-artifact-missing",
                    evidence=portable_evidence,
                )
                lines.append(
                    f"  [{skill_id}] SKIP — optional artifact not found: "
                    f"{artifact}"
                )
            continue

        if artifact_path.is_dir():
            try:
                files = sorted(
                    path
                    for path in artifact_path.iterdir()
                    if path.is_file()
                    and path.suffix.casefold() in SUPPORTED_ARTIFACT_SUFFIXES
                    and path.resolve().is_relative_to(output_root)
                )
            except OSError as exc:
                blocked_assertions += len(assertions)
                failed += 1
                evidence = _portable_evidence(output_root, artifact_path, "artifact")
                _artifact_outcomes(
                    result,
                    output_id,
                    assertions,
                    status="blocked",
                    reason_code="artifact-unreadable",
                    evidence=[evidence] if evidence is not None else [],
                )
                lines.append(f"  [{skill_id}] BLOCKED — artifact unreadable: {exc}")
                continue
            if not files:
                evidence = _portable_evidence(output_root, artifact_path, "artifact")
                portable_evidence = [evidence] if evidence is not None else []
                if required:
                    failed += 1
                    required_missing += 1
                    _artifact_outcomes(
                        result,
                        output_id,
                        assertions,
                        status="fail",
                        reason_code="required-artifact-empty",
                        evidence=portable_evidence,
                    )
                    lines.append(
                        f"  [{skill_id}] FAIL — required artifact "
                        f"directory empty: {artifact}"
                    )
                else:
                    skipped += 1
                    _artifact_outcomes(
                        result,
                        output_id,
                        assertions,
                        status="skip",
                        reason_code="optional-artifact-empty",
                        evidence=portable_evidence,
                    )
                    lines.append(
                        f"  [{skill_id}] SKIP — optional artifact "
                        f"directory empty: {artifact}"
                    )
                continue
            artifact_path = files[0].resolve()
        elif not artifact_path.is_file():
            blocked_assertions += len(assertions)
            failed += 1
            evidence = _portable_evidence(output_root, artifact_path, "artifact")
            _artifact_outcomes(
                result,
                output_id,
                assertions,
                status="blocked",
                reason_code="artifact-kind-invalid",
                evidence=[evidence] if evidence is not None else [],
            )
            lines.append(
                f"  [{skill_id}] BLOCKED — artifact is not a regular "
                f"file: {artifact}"
            )
            continue

        try:
            artifact_empty = artifact_path.stat().st_size == 0
        except OSError as exc:
            blocked_assertions += len(assertions)
            failed += 1
            evidence = _portable_evidence(output_root, artifact_path, "artifact")
            _artifact_outcomes(
                result,
                output_id,
                assertions,
                status="blocked",
                reason_code="artifact-unreadable",
                evidence=[evidence] if evidence is not None else [],
            )
            lines.append(f"  [{skill_id}] BLOCKED — artifact unreadable: {exc}")
            continue
        if required and artifact_empty:
            failed += 1
            required_missing += 1
            evidence = _portable_evidence(output_root, artifact_path, "artifact")
            _artifact_outcomes(
                result,
                output_id,
                assertions,
                status="fail",
                reason_code="required-artifact-empty",
                evidence=[evidence] if evidence is not None else [],
            )
            lines.append(
                f"  [{skill_id}] FAIL — required artifact is empty: "
                f"{artifact}"
            )
            continue

        failures: list[str] = []
        for index, assertion in assertions:
            evidence = _portable_evidence(output_root, artifact_path, "artifact")
            portable_evidence = [evidence] if evidence is not None else []
            portable_evidence.extend(
                _reference_evidence(
                    assertion, case_root=case_root, output_root=output_root
                )
            )
            try:
                assertion_failures = _check_assertion(
                    artifact_path,
                    assertion,
                    case_root=case_root,
                    output_root=output_root,
                )
            except (
                OSError,
                UnicodeError,
                ValueError,
                TypeError,
                csv.Error,
                yaml.YAMLError,
                re.error,
            ) as exc:
                blocked_assertions += 1
                blocked_reasons.append(
                    f"assertion {index} ({assertion['type']}) "
                    f"unavailable: {exc}"
                )
                _add_outcome(
                    result,
                    output_id=output_id,
                    index=index,
                    assertion_type=assertion["type"],
                    status="blocked",
                    reason_code="assertion-evidence-unavailable",
                    evidence=portable_evidence,
                )
                continue
            executed_assertions += 1
            if assertion_failures:
                failed_assertions += 1
                failures.extend(assertion_failures)
                _add_outcome(
                    result,
                    output_id=output_id,
                    index=index,
                    assertion_type=assertion["type"],
                    status="fail",
                    reason_code=assertion["type"].replace("_", "-") + "-failed",
                    evidence=portable_evidence,
                )
            else:
                _add_outcome(
                    result,
                    output_id=output_id,
                    index=index,
                    assertion_type=assertion["type"],
                    status="pass",
                    reason_code="assertion-passed",
                    evidence=portable_evidence,
                )

        if blocked_reasons:
            failed += 1
            lines.append(f"  [{skill_id}] BLOCKED")
            for error in blocked_reasons + failures:
                lines.append(f"    x {error}")
        elif failures:
            failed += 1
            lines.append(f"  [{skill_id}] FAIL")
            for failure in failures:
                lines.append(f"    x {failure}")
        else:
            passed += 1
            lines.append(f"  [{skill_id}] PASS")

    lines.extend(("", "-" * 40))
    lines.append(
        f"Results: {passed}/{total} passed, {failed} failed, "
        f"{skipped} skipped"
    )
    lines.append(
        "Truth: "
        f"required_missing={required_missing}, "
        f"executed_assertions={executed_assertions}, "
        f"failed_assertions={failed_assertions}, "
        f"blocked_assertions={blocked_assertions}, "
        f"unknown_validation_types={unknown_validation_types}"
    )
    if executed_assertions == 0:
        lines.append("  [case] BLOCKED — no assertions executed")
    result["summary"].update(
        {
            "required_missing": required_missing,
            "executed_assertions": executed_assertions,
            "failed_assertions": failed_assertions,
            "blocked_assertions": blocked_assertions,
            "unknown_validation_types": unknown_validation_types,
        }
    )
    return _finish_result(result)


def _print_result(result: dict[str, Any]) -> None:
    print("\n".join(result["_display_lines"]))


def _public_receipt(result: dict[str, Any]) -> dict[str, Any]:
    return {
        "receipt_version": result["receipt_version"],
        "case": result["case"],
        "summary": result["summary"],
        "assertions": result["assertions"],
    }


def _render_json_receipt(result: dict[str, Any]) -> bytes:
    return (
        json.dumps(
            _public_receipt(result),
            allow_nan=False,
            ensure_ascii=False,
            indent=2,
            sort_keys=True,
        )
        + "\n"
    ).encode("utf-8")


def _add_properties(parent: ET.Element, values: list[tuple[str, object]]) -> None:
    properties = ET.SubElement(parent, "properties")
    for name, value in values:
        ET.SubElement(
            properties,
            "property",
            {"name": name, "value": "" if value is None else str(value)},
        )


def _render_junit_receipt(result: dict[str, Any]) -> bytes:
    outcomes = result["assertions"]
    counts = Counter(outcome["status"] for outcome in outcomes)
    suite = ET.Element(
        "testsuite",
        {
            "name": "qiongli-eval",
            "tests": str(len(outcomes)),
            "failures": str(counts["fail"]),
            "errors": str(counts["blocked"]),
            "skipped": str(counts["skip"]),
        },
    )
    case = result["case"]
    summary = result["summary"]
    _add_properties(
        suite,
        [
            ("receipt_version", result["receipt_version"]),
            ("case_id", case["id"]),
            ("pipeline", case["pipeline"]),
            ("schema_version", case["schema_version"]),
            ("status", case["status"]),
            ("reason_code", case["reason_code"]),
            *((counter, summary[counter]) for counter in TRUTH_COUNTERS),
        ],
    )
    for outcome in outcomes:
        index = "contract" if outcome["index"] is None else str(outcome["index"])
        testcase = ET.SubElement(
            suite,
            "testcase",
            {
                "name": f"{outcome['output_id']}[{index}]:{outcome['type']}",
                "classname": str(case["id"]),
            },
        )
        values: list[tuple[str, object]] = [
            ("output_id", outcome["output_id"]),
            ("index", outcome["index"]),
            ("type", outcome["type"]),
            ("status", outcome["status"]),
            ("reason_code", outcome["reason_code"]),
        ]
        for evidence_index, evidence in enumerate(outcome["evidence"]):
            for field in ("role", "path", "sha256"):
                if field in evidence:
                    values.append(
                        (f"evidence.{evidence_index}.{field}", evidence[field])
                    )
        _add_properties(testcase, values)
        child_attributes = {
            "message": outcome["message"],
            "type": outcome["reason_code"],
        }
        if outcome["status"] == "fail":
            ET.SubElement(testcase, "failure", child_attributes)
        elif outcome["status"] == "blocked":
            ET.SubElement(testcase, "error", child_attributes)
        elif outcome["status"] == "skip":
            ET.SubElement(testcase, "skipped", {"message": outcome["message"]})

    ET.indent(suite, space="  ")
    return ET.tostring(
        suite,
        encoding="utf-8",
        xml_declaration=True,
        short_empty_elements=True,
    ) + b"\n"


def _atomic_write(path: str | Path, data: bytes) -> None:
    target = Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    temporary: Path | None = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="wb",
            dir=target.parent,
            prefix=f".{target.name}.",
            suffix=".tmp",
            delete=False,
        ) as handle:
            temporary = Path(handle.name)
            handle.write(data)
            handle.flush()
        temporary.replace(target)
    except OSError:
        if temporary is not None:
            try:
                temporary.unlink(missing_ok=True)
            except OSError:
                pass
        raise


def run_case(
    case_path: str | Path, output_dir: str | Path | None = None
) -> bool:
    result = _evaluate_case(case_path, output_dir)
    _print_result(result)
    return bool(result["_success"])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate existing outputs for one qiongli eval case."
    )
    parser.add_argument("case", help="Path to the eval case YAML")
    parser.add_argument("output_dir", nargs="?", help="Directory containing outputs")
    parser.add_argument("--json-receipt", help="Write a deterministic JSON receipt")
    parser.add_argument("--junit-receipt", help="Write a deterministic JUnit receipt")
    arguments = parser.parse_args(argv)

    if (
        arguments.json_receipt is not None
        and arguments.junit_receipt is not None
        and Path(arguments.json_receipt).resolve()
        == Path(arguments.junit_receipt).resolve()
    ):
        parser.error("JSON and JUnit receipt destinations must be different")

    result = _evaluate_case(arguments.case, arguments.output_dir)
    _print_result(result)
    try:
        if arguments.json_receipt is not None:
            _atomic_write(arguments.json_receipt, _render_json_receipt(result))
        if arguments.junit_receipt is not None:
            _atomic_write(arguments.junit_receipt, _render_junit_receipt(result))
    except (OSError, TypeError, ValueError) as exc:
        print(f"[FAIL] Unable to write eval receipt: {exc}")
        return 1
    return 0 if result["_success"] else 1


if __name__ == "__main__":
    sys.exit(main())
