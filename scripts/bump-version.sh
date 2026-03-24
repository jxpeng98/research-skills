#!/usr/bin/env bash
set -euo pipefail

# Bump release versions across package metadata, portable skill version,
# and all skill frontmatter/registry entries from one source version.
#
# Usage:
#   ./scripts/bump-version.sh <new-version>
#
# Examples:
#   ./scripts/bump-version.sh 0.1.0b7
#   ./scripts/bump-version.sh 0.2.0
#   ./scripts/bump-version.sh v0.2.0-beta.1

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
NEW_VERSION="${1:-}"

if [[ -z "$NEW_VERSION" ]]; then
  echo "Usage: $0 <new-version>"
  echo "  e.g. $0 0.1.0b7"
  exit 2
fi

python3 "${ROOT_DIR}/scripts/sync_versions.py" "${NEW_VERSION}" --root "${ROOT_DIR}"

echo ""
echo "[bump-version] Synced release metadata from ${NEW_VERSION}"
echo ""
echo "Next steps:"
echo "  git add pyproject.toml research_skills/__init__.py research-paper-workflow/VERSION skills/registry.yaml skills"
echo "  git commit -m 'chore: bump version to ${NEW_VERSION}'"
echo "  git tag <repo-version-from-sync-versions-output>"
echo "  git push origin main --tags"
