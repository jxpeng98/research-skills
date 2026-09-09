#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION=""
FROM_TAG=""
ALLOW_DIRTY=0
SKIP_BUMP=0
PACKAGE_VERSION=""
SKILL_VERSION=""
REPO_TAG=""
RELEASE_LINE=""
RELEASE_CHANNEL=""
PRE_ARGS=()
PYPI_ARGS=()
RELEASE_STAGING_DIR=""
RELEASE_STAGING_AUTO=0
CLI_GITHUB=0

cleanup() {
  if [[ "$RELEASE_STAGING_AUTO" -eq 1 && -n "$RELEASE_STAGING_DIR" && -d "$RELEASE_STAGING_DIR" ]]; then
    rm -rf "$RELEASE_STAGING_DIR"
  fi
}

trap cleanup EXIT

release_field() {
  local version="$1"
  local field="$2"
  python3 "${ROOT_DIR}/scripts/release_version.py" "$version" --print-field "$field"
}

is_prerelease_tag() {
  [[ "$(release_field "$1" channel)" != "stable" ]]
}

usage() {
  cat <<'EOF'
Usage:
  ./scripts/release_ready.sh --version <version> [options]

Description:
  Prepare a local publish-ready release state by chaining:
    1) version bump + metadata sync
    2) release preflight (legacy package gates or native alpha dry-run gates)
    3) local plugin install acceptance in an isolated sandbox
    4) package preflight (build + twine + install smoke)

  Native 2.x without --cli-github retains the desktop diagnostic lane.
  --cli-github instead qualifies standalone executable/npm/wheel GitHub assets;
  all readiness modes are local and never publish.

Options:
  --cli-github        Qualify standalone CLI GitHub Release assets on the current target.
                      No App or Community Alpha signing/promotion dependency.
  --version <v>        Required version input (for example 0.2.0, v0.2.0-beta.1,
                       or v2.0.0-alpha.3).
  --from-tag <tag>     Optional baseline tag passed into release note generation.
  --skip-bump          Skip version sync and start from preflight/package checks.
  --allow-dirty        Allow existing local changes before running release prep.
  --skip-smoke         Pass through to release preflight.
  --maintainer-smoke   Pass through to release preflight.
  --no-strict          Pass through to release preflight validator.
  --skip-note-gen      Pass through to release preflight.
  --note-overwrite     Pass through to release preflight.
  --staging-dir <dir>  Staging root used for materialized distribution checks.
  --no-build           Pass through to PyPI preflight.
  --no-install-smoke   Pass through to PyPI preflight.
  --keep-dist          Pass through to PyPI preflight.
  -h, --help           Show this message.
EOF
}

normalize_field() {
  local field="$1"
  release_field "$VERSION" "$field"
}

canonical_external_path() {
  python3 - "$ROOT_DIR" "$1" <<'PY'
from pathlib import Path
import sys

root = Path(sys.argv[1]).resolve()
candidate = Path(sys.argv[2]).expanduser()
if candidate.is_symlink():
    raise SystemExit("[release-ready] native --staging-dir must not be a symbolic link")
out = candidate.resolve()
if out == root or root in out.parents:
    raise SystemExit("[release-ready] native --staging-dir must be outside the source tree")
print(out)
PY
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
    pyproject.toml|packages/python-qiongli/src/qiongli/__init__.py|content/workflow/SKILL.md|content/workflow/VERSION|content/skills/registry.yaml|content/distribution/plugins.yaml|docs/reference/skills.md|docs/zh/reference/skills.md|package-lock.json|uv.lock|tooling/scripts/build_plugin_artifacts.py|tooling/scripts/materialize_distribution_payloads.py|packages/npm-qiongli/package.json|packages/npm-qiongli/README.md|packages/npm-qiongli/LICENSE|packages/npm-qiongli/bin/*|packages/npm-qiongli/lib/*|packages/npm-qiongli/test/*)
      return 0
      ;;
  esac
  if is_prerelease_tag "$REPO_TAG"; then
    [[ "$path" == "tooling/release/${REPO_TAG}.md" ]] && return 0
  else
    [[ "$path" == "CHANGELOG.md" ]] && return 0
    case "$path" in
      README.md|README_CN.md|docs/index.md|docs/zh/index.md|docs/guide/install.md|docs/zh/guide/install.md)
        return 0
        ;;
    esac
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

  if [[ "$SKIP_BUMP" -eq 1 && "$RELEASE_LINE" == "legacy-1x" ]]; then
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
    --cli-github)
      CLI_GITHUB=1
      shift
      ;;
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
    --staging-dir)
      [[ $# -ge 2 ]] || { echo "[release-ready] missing value for --staging-dir" >&2; exit 2; }
      RELEASE_STAGING_DIR="$2"
      shift 2
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

if [[ "$CLI_GITHUB" -eq 1 ]]; then
  [[ -n "$RELEASE_STAGING_DIR" ]] || { echo "[release-ready] --cli-github requires --staging-dir" >&2; exit 2; }
  exec python3 tooling/scripts/native_cli_release.py --version "$VERSION" --out-dir "$RELEASE_STAGING_DIR"
fi

PACKAGE_VERSION="$(normalize_field package_version)"
REPO_TAG="$(normalize_field repo_version)"
RELEASE_LINE="$(normalize_field release_line)"
RELEASE_CHANNEL="$(normalize_field channel)"
SKILL_VERSION="${REPO_TAG#v}"

if [[ "$RELEASE_LINE" != "legacy-1x" && "$RELEASE_LINE" != "native-2x" ]]; then
  echo "[release-ready] unsupported release line: $RELEASE_LINE" >&2
  exit 2
fi

ensure_clean_worktree

if [[ "$RELEASE_LINE" == "native-2x" ]]; then
  echo "[release-ready] native product version is read from packages/qiongli-native/Cargo.toml"
elif [[ "$SKIP_BUMP" -eq 0 ]]; then
  echo "[release-ready] syncing release versions"
  ./scripts/bump-version.sh "$VERSION"
else
  echo "[release-ready] version sync skipped"
fi

if [[ "$RELEASE_LINE" == "legacy-1x" ]] && ! is_prerelease_tag "$REPO_TAG"; then
  echo "[release-ready] update stable download sections"
  python3 scripts/update_stable_download_sections.py --tag "$REPO_TAG" --root "$ROOT_DIR"
fi

PRE_ARGS+=(--tag "$REPO_TAG")
if [[ -n "$FROM_TAG" ]]; then
  PRE_ARGS+=(--from-tag "$FROM_TAG")
fi

if [[ -z "$RELEASE_STAGING_DIR" ]]; then
  RELEASE_STAGING_DIR="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-release-ready.XXXXXX")"
  RELEASE_STAGING_AUTO=1
else
  if [[ "$RELEASE_LINE" == "native-2x" ]]; then
    if ! RELEASE_STAGING_DIR="$(canonical_external_path "$RELEASE_STAGING_DIR")"; then
      exit 2
    fi
  fi
  mkdir -p "$RELEASE_STAGING_DIR"
  RELEASE_STAGING_DIR="$(cd "$RELEASE_STAGING_DIR" && pwd)"
fi

echo "[release-ready] release preflight"
./scripts/release_automation.sh pre "${PRE_ARGS[@]}" --materialize-out "$RELEASE_STAGING_DIR"

VERIFY_ROOT="$RELEASE_STAGING_DIR"
if [[ "$RELEASE_LINE" == "native-2x" ]]; then
  VERIFY_ROOT="$ROOT_DIR"
fi
echo "[release-ready] verify staged release tag version"
bash ./scripts/verify_release_tag_version.sh --root "$VERIFY_ROOT" --tag "$REPO_TAG"

if [[ "$RELEASE_LINE" == "native-2x" ]]; then
  echo "[release-ready] native ${RELEASE_CHANNEL} dry-run generated; legacy package and installer gates are not applicable"
  echo "[release-ready] normalized versions"
  echo "  - native_version:  ${REPO_TAG#v}"
  echo "  - package_version: not-applicable"
  echo "  - skill_version:   not-applicable"
  echo "  - repo_tag:        ${REPO_TAG}"
  echo "  - release_line:    ${RELEASE_LINE}"
  echo "  - channel:         ${RELEASE_CHANNEL}"
  echo "[release-ready] prepare+verify completed; native publication remains blocked by RLS-201/PKG gates"
  exit 0
fi

echo "[release-ready] experience schema compatibility"
python3 scripts/check_experience_schema_compatibility.py --root "$RELEASE_STAGING_DIR"

echo "[release-ready] local plugin install acceptance"
python3 scripts/release_local_install_check.py --root "$RELEASE_STAGING_DIR"

echo "[release-ready] package preflight"
bash ./scripts/pypi_preflight.sh --root "$RELEASE_STAGING_DIR" "${PYPI_ARGS[@]}"

echo "[release-ready] npm package preflight"
bash ./scripts/npm_preflight.sh --root "$RELEASE_STAGING_DIR"

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

echo "[release-ready] prepare+verify completed; publish mode owns commit/tag/push"
