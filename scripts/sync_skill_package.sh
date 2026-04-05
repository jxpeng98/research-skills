#!/usr/bin/env bash
# sync_skill_package.sh — populate research-paper-workflow/ with bundled assets
# Run before install/release to ensure the skill package is self-contained.
#
# Usage:
#   ./scripts/sync_skill_package.sh [--dry-run]
#
# Copies the following into research-paper-workflow/:
#   skills/          → research-paper-workflow/skills/
#   skills-core.md   → research-paper-workflow/skills-core.md
#   templates/       → research-paper-workflow/templates/
#   standards/       → research-paper-workflow/standards/
#   roles/           → research-paper-workflow/roles/
#
# These paths are .gitignore'd — they are generated artifacts, not source of truth.
# The canonical source of truth remains the repo-root directories.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PKG_DIR="$ROOT_DIR/research-paper-workflow"

DRY_RUN=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    *) echo "Unknown argument: $arg" >&2; exit 1 ;;
  esac
done

# ── Sync targets ─────────────────────────────────────────────────────────────

SYNC_DIRS=(
  "skills"
  "templates"
  "standards"
  "roles"
)

SYNC_FILES=(
  "skills-core.md"
)

# Exclude files that are project-bootstrap templates, not research output templates
EXCLUDE_FILES=(
  "CLAUDE.project.md"
)

# ── Helpers ──────────────────────────────────────────────────────────────────

sync_dir() {
  local src="$ROOT_DIR/$1"
  local dest="$PKG_DIR/$1"
  if [[ ! -d "$src" ]]; then
    echo "  [skip] $1 (source not found)"
    return
  fi
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "  [dry-run] sync $src/ → $dest/"
    return
  fi
  # Clean copy: remove stale, copy fresh, prune excludes
  rm -rf "$dest"
  cp -a "$src" "$dest"
  # Remove excluded files
  for excl in "${EXCLUDE_FILES[@]}"; do
    find "$dest" -name "$excl" -delete 2>/dev/null || true
  done
  # Remove system junk
  find "$dest" -name '.DS_Store' -delete 2>/dev/null || true
  find "$dest" -name '__pycache__' -type d -exec rm -rf {} + 2>/dev/null || true
  echo "  [ok] $1/"
}

sync_file() {
  local src="$ROOT_DIR/$1"
  local dest="$PKG_DIR/$1"
  if [[ ! -f "$src" ]]; then
    echo "  [skip] $1 (source not found)"
    return
  fi
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "  [dry-run] cp $src → $dest"
    return
  fi
  cp "$src" "$dest"
  echo "  [ok] $1"
}

# ── Main ─────────────────────────────────────────────────────────────────────

echo "Syncing skill package: $PKG_DIR"

for dir in "${SYNC_DIRS[@]}"; do
  sync_dir "$dir"
done

for file in "${SYNC_FILES[@]}"; do
  sync_file "$file"
done

echo "[done] Skill package is self-contained."
