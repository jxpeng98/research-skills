#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
VERSION=""
FROM_TAG=""
ALLOW_DIRTY=0
SKIP_BUMP=0
PACKAGE_VERSION=""
SKILL_VERSION=""
REPO_TAG=""
PRE_ARGS=()
PYPI_ARGS=()

is_prerelease_tag() {
  [[ "$1" == *beta* || "$1" =~ b[0-9]+ ]]
}

usage() {
  cat <<'EOF'
Usage:
  ./scripts/release_ready.sh --version <version> [options]

Description:
  Prepare a local publish-ready release state by chaining:
    1) version bump + metadata sync
    2) release preflight (validator + tests + smoke + beta note draft or stable changelog check)
    3) package preflight (build + twine + install smoke)

Options:
  --version <v>        Required version input (for example 0.2.0, 0.2.0b1, or v0.2.0-beta.1).
  --from-tag <tag>     Optional baseline tag passed into release note generation.
  --skip-bump          Skip version sync and start from preflight/package checks.
  --allow-dirty        Allow existing local changes before running release prep.
  --skip-smoke         Pass through to release preflight.
  --maintainer-smoke   Pass through to release preflight.
  --no-strict          Pass through to release preflight validator.
  --skip-note-gen      Pass through to release preflight.
  --note-overwrite     Pass through to release preflight.
  --no-build           Pass through to PyPI preflight.
  --no-install-smoke   Pass through to PyPI preflight.
  --keep-dist          Pass through to PyPI preflight.
  -h, --help           Show this message.
EOF
}

normalize_field() {
  local field="$1"
  python3 "${ROOT_DIR}/scripts/sync_versions.py" "${VERSION}" --print-field "${field}"
}

status_path_from_line() {
  local line="$1"
  local path="${line:3}"
  if [[ "$path" == *" -> "* ]]; then
    path="${path##* -> }"
  fi
  printf '%s\n' "$path"
}

is_expected_release_path() {
  local path="$1"
  case "$path" in
    pyproject.toml|research_skills/__init__.py|research-paper-workflow/VERSION|skills/registry.yaml)
      return 0
      ;;
    skills/*)
      return 0
      ;;
  esac
  if is_prerelease_tag "$REPO_TAG"; then
    [[ "$path" == "release/${REPO_TAG}.md" ]] && return 0
  else
    [[ "$path" == "CHANGELOG.md" ]] && return 0
  fi
  return 1
}

ensure_clean_worktree() {
  local dirty
  dirty="$(git status --short || true)"
  if [[ -z "$dirty" ]]; then
    return 0
  fi

  if [[ "$ALLOW_DIRTY" -eq 1 ]]; then
    echo "[release-ready] warning: working tree is dirty, continuing because --allow-dirty was set"
    printf '%s\n' "$dirty"
    return 0
  fi

  if [[ "$SKIP_BUMP" -eq 1 ]]; then
    local unexpected=()
    local path=""
    while IFS= read -r line; do
      [[ -z "$line" ]] && continue
      path="$(status_path_from_line "$line")"
      if ! is_expected_release_path "$path"; then
        unexpected+=("$path")
      fi
    done <<< "$dirty"

    if [[ ${#unexpected[@]} -eq 0 ]]; then
      echo "[release-ready] detected existing release-prep changes, continuing with --skip-bump"
      printf '%s\n' "$dirty"
      return 0
    fi
  fi

  echo "[release-ready] working tree must be clean before release prep" >&2
  printf '%s\n' "$dirty" >&2
  echo "[release-ready] rerun with --allow-dirty only if you intentionally want to include unrelated changes" >&2
  exit 1
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      [[ $# -ge 2 ]] || { echo "[release-ready] missing value for --version" >&2; exit 2; }
      VERSION="$2"
      shift 2
      ;;
    --from-tag)
      [[ $# -ge 2 ]] || { echo "[release-ready] missing value for --from-tag" >&2; exit 2; }
      FROM_TAG="$2"
      shift 2
      ;;
    --skip-bump)
      SKIP_BUMP=1
      shift
      ;;
    --allow-dirty)
      ALLOW_DIRTY=1
      shift
      ;;
    --skip-smoke|--maintainer-smoke|--no-strict|--skip-note-gen|--note-overwrite)
      PRE_ARGS+=("$1")
      shift
      ;;
    --no-build|--no-install-smoke|--keep-dist)
      PYPI_ARGS+=("$1")
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[release-ready] unknown option: $1" >&2
      usage
      exit 2
      ;;
  esac
done

[[ -n "$VERSION" ]] || { echo "[release-ready] --version is required" >&2; usage; exit 2; }

cd "$ROOT_DIR"

PACKAGE_VERSION="$(normalize_field package_version)"
SKILL_VERSION="$(normalize_field skill_version)"
REPO_TAG="$(normalize_field repo_version)"

ensure_clean_worktree

if [[ "$SKIP_BUMP" -eq 0 ]]; then
  echo "[release-ready] syncing release versions"
  ./scripts/bump-version.sh "$VERSION"
else
  echo "[release-ready] version sync skipped"
fi

PRE_ARGS+=(--tag "$REPO_TAG")
if [[ -n "$FROM_TAG" ]]; then
  PRE_ARGS+=(--from-tag "$FROM_TAG")
fi

echo "[release-ready] release preflight"
./scripts/release_automation.sh pre "${PRE_ARGS[@]}"

echo "[release-ready] package preflight"
bash ./scripts/pypi_preflight.sh "${PYPI_ARGS[@]}"

echo "[release-ready] publish-ready state confirmed"
echo "[release-ready] normalized versions"
echo "  - package_version: ${PACKAGE_VERSION}"
echo "  - skill_version:   ${SKILL_VERSION}"
echo "  - repo_tag:        ${REPO_TAG}"

status_snapshot="$(git status --short || true)"
if [[ -n "$status_snapshot" ]]; then
  echo "[release-ready] working tree snapshot"
  printf '%s\n' "$status_snapshot"
fi

echo "[release-ready] next steps"
if is_prerelease_tag "$REPO_TAG"; then
  echo "  git add pyproject.toml research_skills/__init__.py research-paper-workflow/VERSION skills/registry.yaml skills release/${REPO_TAG}.md"
else
  echo "  git add pyproject.toml research_skills/__init__.py research-paper-workflow/VERSION skills/registry.yaml skills CHANGELOG.md"
fi
echo "  git commit -m 'chore: prepare release ${PACKAGE_VERSION}'"
echo "  # optional: run the 'Publish to TestPyPI' workflow and validate install before tagging"
echo "  git tag -a ${REPO_TAG} -m \"research-skills release\""
echo "  git push origin main --tags"
echo "  ./scripts/release_automation.sh post --tag ${REPO_TAG} --create-release"
