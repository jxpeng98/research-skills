#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TAG=""

usage() {
  cat <<'EOF'
Usage:
  ./scripts/verify_release_tag_version.sh --tag <tag>

Description:
  Verify that a Git release tag matches the package and workflow versions in the repo.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      [[ $# -ge 2 ]] || { echo "[verify-release-tag] missing value for --tag" >&2; exit 2; }
      TAG="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[verify-release-tag] unknown option: $1" >&2
      usage
      exit 2
      ;;
  esac
done

[[ -n "$TAG" ]] || { echo "[verify-release-tag] --tag is required" >&2; usage; exit 2; }

cd "$ROOT_DIR"

expected_repo_tag="$(python3 scripts/sync_versions.py "$TAG" --print-field repo_version)"
expected_package_version="$(python3 scripts/sync_versions.py "$TAG" --print-field package_version)"
expected_skill_version="$(python3 scripts/sync_versions.py "$TAG" --print-field skill_version)"

if [[ "$expected_repo_tag" != "$TAG" ]]; then
  echo "[verify-release-tag] normalized tag mismatch: expected $expected_repo_tag from input $TAG" >&2
  exit 1
fi

actual_package_version="$(python3 - <<'PY'
import re
from pathlib import Path

content = Path("pyproject.toml").read_text(encoding="utf-8")
match = re.search(r'^version = "([^"]+)"$', content, re.MULTILINE)
if not match:
    raise SystemExit("missing version in pyproject.toml")
print(match.group(1))
PY
)"

actual_init_version="$(python3 - <<'PY'
import re
from pathlib import Path

content = Path("research_skills/__init__.py").read_text(encoding="utf-8")
match = re.search(r'^__version__ = "([^"]+)"$', content, re.MULTILINE)
if not match:
    raise SystemExit("missing __version__ in research_skills/__init__.py")
print(match.group(1))
PY
)"

actual_skill_version="$(python3 - <<'PY'
import re
from pathlib import Path

content = Path("skills/registry.yaml").read_text(encoding="utf-8")
match = re.search(r'^\s*version: "([^"]+)"$', content, re.MULTILINE)
if not match:
    raise SystemExit("missing version in skills/registry.yaml")
print(match.group(1))
PY
)"

actual_workflow_version="$(tr -d '\r\n' < research-paper-workflow/VERSION)"

[[ "$actual_package_version" == "$expected_package_version" ]] || {
  echo "[verify-release-tag] pyproject version mismatch: tag=$TAG expects $expected_package_version, found $actual_package_version" >&2
  exit 1
}

[[ "$actual_init_version" == "$expected_package_version" ]] || {
  echo "[verify-release-tag] research_skills/__init__.py mismatch: tag=$TAG expects $expected_package_version, found $actual_init_version" >&2
  exit 1
}

[[ "$actual_skill_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_skill_version" >&2
  exit 1
}

[[ "$actual_workflow_version" == "$expected_repo_tag" ]] || {
  echo "[verify-release-tag] research-paper-workflow/VERSION mismatch: tag=$TAG expects $expected_repo_tag, found $actual_workflow_version" >&2
  exit 1
}

echo "[verify-release-tag] tag and repo versions are aligned: $TAG"
