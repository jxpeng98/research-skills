#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import sys

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from research_skills.skill_docs import generate_skill_reference_docs


def main() -> int:
    root = REPO_ROOT
    generated = generate_skill_reference_docs(root)
    for relative_path, content in generated.items():
        target = root / relative_path
        target.write_text(content, encoding="utf-8")
        print(f"[WRITE] {relative_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
