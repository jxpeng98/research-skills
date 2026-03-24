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
echo "  ./scripts/release_ready.sh --version ${NEW_VERSION} --skip-bump"
echo "  # or manually continue with git add/commit + release preflight + package preflight"
