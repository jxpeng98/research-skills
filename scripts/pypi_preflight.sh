#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
RUN_BUILD=1
RUN_INSTALL_SMOKE=1
KEEP_DIST=0

usage() {
  cat <<'EOF'
Usage:
  ./scripts/pypi_preflight.sh [options]

Description:
  Pre-publish checks for PyPI/TestPyPI release safety.

Checks:
  1) Clean dist/ (optional)
  2) Build sdist + wheel
  3) twine metadata validation
  4) Install latest wheel in a temporary virtualenv
  5) CLI smoke checks (research-skills / rsk / rsw)

Options:
  --no-build         Skip build step (expects artifacts in dist/)
  --no-install-smoke Skip temporary venv install + CLI smoke checks
  --keep-dist        Do not delete existing dist/ before build
  -h, --help         Show this message
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-build)
      RUN_BUILD=0
      shift
      ;;
    --no-install-smoke)
      RUN_INSTALL_SMOKE=0
      shift
      ;;
    --keep-dist)
      KEEP_DIST=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[pypi-preflight] unknown option: $1" >&2
      usage
      exit 2
      ;;
  esac
done

cd "$ROOT_DIR"

if [[ "$RUN_BUILD" -eq 1 ]]; then
  if [[ "$KEEP_DIST" -eq 0 ]]; then
    echo "[pypi-preflight] cleaning dist/"
    rm -rf dist
  fi

  echo "[pypi-preflight] building package"
  python3 -m build
fi

echo "[pypi-preflight] checking package metadata"
python3 -m twine check dist/*

if [[ "$RUN_INSTALL_SMOKE" -eq 1 ]]; then
  shopt -s nullglob
  wheels=(dist/*.whl)
  shopt -u nullglob
  if [[ ${#wheels[@]} -eq 0 ]]; then
    echo "[pypi-preflight] no wheel found under dist/" >&2
    exit 1
  fi

  latest_wheel="$(ls -1t dist/*.whl | head -n1)"
  tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/rs-pypi-preflight.XXXXXX")"
  trap 'rm -rf "$tmp_dir"' EXIT

  echo "[pypi-preflight] creating temp venv: $tmp_dir"
  python3 -m venv "$tmp_dir/venv"
  venv_python="$tmp_dir/venv/bin/python"
  venv_research_skills="$tmp_dir/venv/bin/research-skills"
  venv_rsk="$tmp_dir/venv/bin/rsk"
  venv_rsw="$tmp_dir/venv/bin/rsw"

  "$venv_python" -m pip install --upgrade pip >/dev/null
  "$venv_python" -m pip install --ignore-installed --force-reinstall "$latest_wheel"

  echo "[pypi-preflight] smoke: research-skills --help"
  "$venv_research_skills" --help >/dev/null

  echo "[pypi-preflight] smoke: rsk --help"
  "$venv_rsk" --help >/dev/null

  echo "[pypi-preflight] smoke: rsw --help"
  "$venv_rsw" --help >/dev/null

  echo "[pypi-preflight] smoke: subcommand help"
  "$venv_rsk" check --help >/dev/null
  "$venv_rsk" upgrade --help >/dev/null
fi

echo "[pypi-preflight] all checks passed"
echo "[pypi-preflight] next steps:"
echo "  - TestPyPI (manual): run workflow 'Publish to TestPyPI' in GitHub Actions"
echo "  - PyPI (tag trigger): git tag v<version> && git push origin v<version>"
