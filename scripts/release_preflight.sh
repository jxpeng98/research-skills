#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
RUN_SMOKE=1
STRICT_MODE=1
TAG=""

usage() {
  cat <<'EOF'
Usage:
  ./scripts/release_preflight.sh [--tag <tag>] [--skip-smoke] [--no-strict]

Description:
  Run standardized pre-release gates:
    1) strict standard validator
    2) orchestrator workflow unit tests
    3) beta smoke script (doctor + parallel + task-run)

Options:
  --tag <tag>     Optional release tag to pre-check. If provided, script verifies
                  the tag does not already exist locally.
  --skip-smoke    Skip smoke test stage.
  --no-strict     Run validator without --strict.
  -h, --help      Show this message.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      [[ $# -ge 2 ]] || { echo "[preflight] missing value for --tag" >&2; exit 2; }
      TAG="$2"
      shift 2
      ;;
    --skip-smoke)
      RUN_SMOKE=0
      shift
      ;;
    --no-strict)
      STRICT_MODE=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[preflight] unknown option: $1" >&2
      usage
      exit 2
      ;;
  esac
done

cd "$ROOT_DIR"

if [[ -n "$TAG" ]]; then
  if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
    echo "[preflight] tag already exists locally: $TAG" >&2
    exit 1
  fi
  echo "[preflight] tag pre-check passed: $TAG is available"
fi

validate_cmd=(python3 scripts/validate_research_standard.py)
if [[ "$STRICT_MODE" -eq 1 ]]; then
  validate_cmd+=(--strict)
fi

echo "[preflight] validator"
"${validate_cmd[@]}"

echo "[preflight] unit tests"
python3 -m unittest tests.test_orchestrator_workflows -v

if [[ "$RUN_SMOKE" -eq 1 ]]; then
  echo "[preflight] smoke"
  ./scripts/run_beta_smoke.sh
else
  echo "[preflight] smoke skipped"
fi

echo "[preflight] all checks passed"
if [[ -n "$TAG" ]]; then
  echo "[preflight] next publish commands:"
  echo "  git tag -a $TAG -m \"research-skills beta release\""
  echo "  git push origin $TAG"
fi
