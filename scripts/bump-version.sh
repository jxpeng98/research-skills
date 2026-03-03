#!/usr/bin/env bash
set -euo pipefail

# Bump version in both pyproject.toml and research_skills/__init__.py
#
# Usage:
#   ./scripts/bump-version.sh <new-version>
#
# Examples:
#   ./scripts/bump-version.sh 0.1.0b7
#   ./scripts/bump-version.sh 0.2.0
#   ./scripts/bump-version.sh 1.0.0

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
NEW_VERSION="${1:-}"

if [[ -z "$NEW_VERSION" ]]; then
  echo "Usage: $0 <new-version>"
  echo "  e.g. $0 0.1.0b7"
  exit 2
fi

PYPROJECT="${ROOT_DIR}/pyproject.toml"
INIT_PY="${ROOT_DIR}/research_skills/__init__.py"

# --- Update pyproject.toml ---
if [[ "$(uname)" == "Darwin" ]]; then
  sed -i '' "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" "$PYPROJECT"
else
  sed -i "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" "$PYPROJECT"
fi

# --- Update __init__.py ---
if [[ "$(uname)" == "Darwin" ]]; then
  sed -i '' "s/^__version__ = \".*\"/__version__ = \"${NEW_VERSION}\"/" "$INIT_PY"
else
  sed -i "s/^__version__ = \".*\"/__version__ = \"${NEW_VERSION}\"/" "$INIT_PY"
fi

echo "[bump-version] Updated to ${NEW_VERSION}"
echo "  - ${PYPROJECT}"
echo "  - ${INIT_PY}"
echo ""
echo "Next steps:"
echo "  git add pyproject.toml research_skills/__init__.py"
echo "  git commit -m 'chore: bump version to ${NEW_VERSION}'"
echo "  git tag v${NEW_VERSION}"
echo "  git push origin main --tags"
