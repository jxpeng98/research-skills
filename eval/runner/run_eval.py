#!/usr/bin/env python3
"""Simple eval runner for research skills golden tests.

Usage:
    python eval/runner/run_eval.py eval/cases/sr-social-media-mental-health.yaml

This runner validates that skill outputs match expected structure.
It does NOT execute skills — it checks existing outputs against expectations.
"""
import sys
import os
import yaml
import re


def load_case(path: str) -> dict:
    with open(path) as f:
        return yaml.safe_load(f)


def check_artifact_exists(base_dir: str, artifact_path: str) -> bool:
    full = os.path.join(base_dir, artifact_path)
    return os.path.exists(full)


def check_must_contain(content: str, patterns: list) -> list:
    """Check if content contains required patterns. Supports 'or' patterns."""
    failures = []
    for pattern in patterns:
        if " or " in str(pattern):
            alternatives = [p.strip().strip('"').strip("'") for p in str(pattern).split(" or ")]
            if not any(alt.lower() in content.lower() for alt in alternatives):
                failures.append(f"Missing: {pattern}")
        else:
            clean = str(pattern).strip('"').strip("'")
            if clean.lower() not in content.lower():
                failures.append(f"Missing: {clean}")
    return failures


def run_case(case_path: str, output_dir: str = None):
    case = load_case(case_path)
    print(f"\n{'='*60}")
    print(f"Eval Case: {case['case_id']}")
    print(f"Pipeline:  {case['pipeline']}")
    print(f"{'='*60}")

    if output_dir is None:
        topic_slug = re.sub(r'[^a-z0-9]+', '_', case['input']['topic'].lower())[:40]
        output_dir = f"RESEARCH/{topic_slug}"

    total = 0
    passed = 0
    failed = 0

    for skill_id, expected in case.get('expected_outputs', {}).items():
        total += 1
        artifact = expected.get('artifact', '')
        exists = check_artifact_exists(output_dir, artifact)

        if not exists:
            print(f"  [{skill_id}] SKIP — artifact not found: {artifact}")
            continue

        artifact_path = os.path.join(output_dir, artifact)
        if os.path.isdir(artifact_path):
            # For directory artifacts, check any file inside
            files = [f for f in os.listdir(artifact_path) if f.endswith(('.md', '.py', '.r', '.R'))]
            if files:
                with open(os.path.join(artifact_path, files[0])) as f:
                    content = f.read()
            else:
                print(f"  [{skill_id}] SKIP — directory empty: {artifact}")
                continue
        else:
            with open(artifact_path) as f:
                content = f.read()

        must_contain = expected.get('must_contain', [])
        failures = check_must_contain(content, must_contain)

        if failures:
            failed += 1
            print(f"  [{skill_id}] FAIL")
            for f in failures:
                print(f"    ✗ {f}")
        else:
            passed += 1
            print(f"  [{skill_id}] PASS")

    print(f"\n{'─'*40}")
    print(f"Results: {passed}/{total} passed, {failed} failed, {total - passed - failed} skipped")
    return failed == 0


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python eval/runner/run_eval.py <case.yaml> [output_dir]")
        sys.exit(1)

    case_path = sys.argv[1]
    output_dir = sys.argv[2] if len(sys.argv) > 2 else None
    success = run_case(case_path, output_dir)
    sys.exit(0 if success else 1)
