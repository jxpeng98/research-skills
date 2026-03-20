#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MODE="${1:-}"
shift || true

usage() {
  cat <<'EOF'
Usage:
  ./scripts/release_automation.sh <pre|post|full> [options]

Examples:
  ./scripts/release_automation.sh pre --tag v0.1.0 --from-tag v0.1.0-beta.6
  ./scripts/release_automation.sh pre --tag v0.1.1-beta.1 --from-tag v0.1.0
  ./scripts/release_automation.sh post --tag v0.1.0
  ./scripts/release_automation.sh full --tag v0.1.0

Notes:
  - pre  -> runs scripts/release_preflight.sh
  - post -> runs scripts/release_postflight.sh
  - full -> runs preflight, then postflight (uses --tag for postflight)
  - pre/full support --from-tag, --skip-note-gen, --note-overwrite
EOF
}

[[ -n "$MODE" ]] || { usage; exit 2; }

cd "$ROOT_DIR"

case "$MODE" in
  pre)
    ./scripts/release_preflight.sh "$@"
    ;;
  post)
    ./scripts/release_postflight.sh "$@"
    ;;
  full)
    pre_args=()
    post_args=()
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --tag)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --tag" >&2; exit 2; }
          post_args+=("$1" "$2")
          shift 2
          ;;
        --skip-smoke|--no-strict)
          pre_args+=("$1")
          shift
          ;;
        --skip-note-gen|--note-overwrite)
          pre_args+=("$1")
          shift
          ;;
        --from-tag)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --from-tag" >&2; exit 2; }
          pre_args+=("$1" "$2")
          shift 2
          ;;
        --skip-remote|--skip-ci-status|--create-release)
          post_args+=("$1")
          shift
          ;;
        --repo|--acceptance-out)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for $1" >&2; exit 2; }
          post_args+=("$1" "$2")
          shift 2
          ;;
        *)
          echo "[release-automation] unknown option for full mode: $1" >&2
          usage
          exit 2
          ;;
      esac
    done

    ./scripts/release_preflight.sh "${pre_args[@]}"
    ./scripts/release_postflight.sh "${post_args[@]}"
    ;;
  -h|--help|help)
    usage
    ;;
  *)
    echo "[release-automation] unknown mode: $MODE" >&2
    usage
    exit 2
    ;;
esac
