#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


VERSION_PATTERN = re.compile(
    r"^(?:v)?(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)(?:(?:-beta\.|b)(?P<beta>\d+))?$"
)


def parse_version(raw: str) -> tuple[str, str, str]:
    match = VERSION_PATTERN.fullmatch(raw.strip())
    if not match:
        raise ValueError(
            "unsupported version format. Use stable `X.Y.Z` or beta `X.Y.ZbN` / `vX.Y.Z-beta.N`."
        )

    major = int(match.group("major"))
    minor = int(match.group("minor"))
    patch = int(match.group("patch"))
    beta_raw = match.group("beta")

    if beta_raw is None:
        skill_version = f"{major}.{minor}.{patch}"
        package_version = skill_version
        repo_version = f"v{skill_version}"
        return package_version, skill_version, repo_version

    beta = int(beta_raw)
    package_version = f"{major}.{minor}.{patch}b{beta}"
    skill_version = f"{major}.{minor}.{patch}-beta.{beta}"
    repo_version = f"v{skill_version}"
    return package_version, skill_version, repo_version


def replace_pattern(path: Path, pattern: re.Pattern[str], replacement: str) -> bool:
    original = path.read_text(encoding="utf-8")
    updated, count = pattern.subn(replacement, original)
    if count == 0:
        raise ValueError(f"no matching version field found in {path}")
    if updated != original:
        path.write_text(updated, encoding="utf-8")
        return True
    return False


def sync_versions(root: Path, raw_version: str) -> list[Path]:
    package_version, skill_version, repo_version = parse_version(raw_version)
    changed: list[Path] = []

    replacements: list[tuple[Path, re.Pattern[str], str]] = [
        (
            root / "pyproject.toml",
            re.compile(r'^version = "[^"]+"$', re.MULTILINE),
            f'version = "{package_version}"',
        ),
        (
            root / "research_skills" / "__init__.py",
            re.compile(r'^__version__ = "[^"]+"$', re.MULTILINE),
            f'__version__ = "{package_version}"',
        ),
        (
            root / "skills" / "registry.yaml",
            re.compile(r'^(\s*version: )"[^"]+"$', re.MULTILINE),
            rf'\g<1>"{skill_version}"',
        ),
    ]

    for path, pattern, replacement in replacements:
        if replace_pattern(path, pattern, replacement):
            changed.append(path)

    version_file = root / "research-paper-workflow" / "VERSION"
    original_repo_version = version_file.read_text(encoding="utf-8").strip()
    if original_repo_version != repo_version:
        version_file.write_text(repo_version + "\n", encoding="utf-8")
        changed.append(version_file)

    skill_pattern = re.compile(r'^version: "[^"]+"$', re.MULTILINE)
    for path in sorted((root / "skills").rglob("*.md")):
        if replace_pattern(path, skill_pattern, f'version: "{skill_version}"'):
            changed.append(path)

    return changed


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Sync package, portable skill, and skill metadata versions from one release version."
    )
    parser.add_argument("version", help="Stable or beta version, e.g. 0.2.0 or 0.2.0b1")
    parser.add_argument(
        "--root",
        default=Path(__file__).resolve().parents[1],
        type=Path,
        help="Repository root (defaults to current repo root)",
    )
    args = parser.parse_args(argv)

    root = args.root.resolve()
    changed = sync_versions(root, args.version)
    package_version, skill_version, repo_version = parse_version(args.version)

    print("[sync-versions] normalized versions")
    print(f"  - package_version: {package_version}")
    print(f"  - skill_version:   {skill_version}")
    print(f"  - repo_version:    {repo_version}")
    print(f"  - changed_files:   {len(changed)}")
    for path in changed:
        print(f"    - {path.relative_to(root)}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ValueError as exc:
        print(f"[sync-versions] {exc}", file=sys.stderr)
        raise SystemExit(2)
