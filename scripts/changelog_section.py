#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


HEADING_RE = re.compile(r"^## \[(?P<version>[^\]]+)\](?P<suffix>.*)$")


def extract_section(content: str, version: str) -> str | None:
    lines = content.splitlines()
    start = None
    end = None

    for index, line in enumerate(lines):
        match = HEADING_RE.match(line)
        if not match:
            continue
        if match.group("version").strip() == version:
            start = index
            break

    if start is None:
        return None

    for index in range(start + 1, len(lines)):
        if HEADING_RE.match(lines[index]):
            end = index
            break

    section_lines = lines[start:end]
    text = "\n".join(section_lines).strip() + "\n"
    return text


def main() -> int:
    parser = argparse.ArgumentParser(description="Extract a version section from CHANGELOG.md.")
    parser.add_argument("--version", required=True, help="Version label inside the changelog header, for example 0.4.0")
    parser.add_argument("--input", default="CHANGELOG.md", help="Path to changelog file")
    parser.add_argument("--output", help="Optional output file path")
    parser.add_argument("--check", action="store_true", help="Only verify the section exists")
    args = parser.parse_args()

    changelog_path = Path(args.input)
    if not changelog_path.exists():
        print(f"[changelog] missing changelog file: {changelog_path}", file=sys.stderr)
        return 1

    content = changelog_path.read_text(encoding="utf-8")
    section = extract_section(content, args.version)
    if section is None:
        print(f"[changelog] missing version section: {args.version}", file=sys.stderr)
        return 1

    if args.check:
        print(f"[changelog] found version section: {args.version}")
        return 0

    if args.output:
        Path(args.output).write_text(section, encoding="utf-8")
        print(f"[changelog] wrote section {args.version} to {args.output}")
        return 0

    sys.stdout.write(section)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
