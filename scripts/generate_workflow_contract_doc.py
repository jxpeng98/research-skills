#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import sys

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from research_skills.workflow_contract_doc import generate_workflow_contract_reference


def main() -> int:
    target = REPO_ROOT / "research-paper-workflow" / "references" / "workflow-contract.md"
    target.write_text(generate_workflow_contract_reference(REPO_ROOT), encoding="utf-8")
    print(f"[WRITE] {target.relative_to(REPO_ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
