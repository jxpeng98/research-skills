#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MODE="${1:-}"
shift || true

normalize_field() {
  local version="$1"
  local field="$2"
  python3 "${ROOT_DIR}/scripts/sync_versions.py" "$version" --print-field "$field"
}

detect_primary_branch() {
  if git show-ref --verify --quiet "refs/heads/main"; then
    printf 'main\n'
    return 0
  fi
  if git show-ref --verify --quiet "refs/heads/master"; then
    printf 'master\n'
    return 0
  fi
  git rev-parse --abbrev-ref HEAD
}

ensure_git_identity() {
  if git config user.name >/dev/null 2>&1 && git config user.email >/dev/null 2>&1; then
    return 0
  fi

  if [[ "${GITHUB_ACTIONS:-}" == "true" ]]; then
    git config user.name "github-actions[bot]"
    git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
    return 0
  fi

  echo "[release-automation] git user.name and user.email must be configured before publish mode can create a commit" >&2
  exit 1
}

usage() {
  cat <<'EOF'
Usage:
  ./scripts/release_automation.sh <pre|post|publish> [options]

Examples:
  ./scripts/release_automation.sh pre --tag v0.1.0 --from-tag v0.1.0-beta.6
  ./scripts/release_automation.sh pre --tag v0.1.1-beta.1 --from-tag v0.1.0
  ./scripts/release_automation.sh post --tag v0.1.0
  ./scripts/release_automation.sh post --tag v0.1.0 --create-release
  ./scripts/release_automation.sh publish --version 0.1.0 --from-tag v0.1.0-beta.6

Notes:
  - pre  -> runs scripts/release_preflight.sh
  - post -> runs scripts/release_postflight.sh
  - Run them in two phases: preflight before tagging, then postflight after the tag exists remotely.
  - pre supports pass-through flags such as --from-tag, --skip-note-gen, --note-overwrite, --skip-smoke, and --no-strict.
  - publish -> runs release_ready, commits release-prep files, creates/pushes the tag, waits for CI, then runs postflight with release-page creation.
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
  publish)
    version=""
    from_tag=""
    allow_dirty=0
    skip_bump=0
    commit_message=""
    push_remote="origin"
    push_branch=""
    tag_message="research-skills release"
    create_release=1
    skip_remote=0
    skip_ci_status=0
    wait_ci=1
    ci_timeout_seconds=1800
    ci_poll_interval_seconds=30
    ready_args=()
    post_args=()

    while [[ $# -gt 0 ]]; do
      case "$1" in
        --version)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --version" >&2; exit 2; }
          version="$2"
          shift 2
          ;;
        --from-tag)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --from-tag" >&2; exit 2; }
          from_tag="$2"
          ready_args+=("$1" "$2")
          shift 2
          ;;
        --skip-bump)
          skip_bump=1
          ready_args+=("$1")
          shift
          ;;
        --allow-dirty)
          allow_dirty=1
          ready_args+=("$1")
          shift
          ;;
        --skip-smoke|--no-strict|--skip-note-gen|--note-overwrite|--no-build|--no-install-smoke|--keep-dist)
          ready_args+=("$1")
          shift
          ;;
        --commit-message)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --commit-message" >&2; exit 2; }
          commit_message="$2"
          shift 2
          ;;
        --push-remote)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --push-remote" >&2; exit 2; }
          push_remote="$2"
          shift 2
          ;;
        --push-branch)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --push-branch" >&2; exit 2; }
          push_branch="$2"
          shift 2
          ;;
        --tag-message)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --tag-message" >&2; exit 2; }
          tag_message="$2"
          shift 2
          ;;
        --no-create-release)
          create_release=0
          shift
          ;;
        --skip-remote)
          skip_remote=1
          post_args+=("$1")
          shift
          ;;
        --skip-ci-status)
          skip_ci_status=1
          wait_ci=0
          post_args+=("$1")
          shift
          ;;
        --wait-ci)
          wait_ci=1
          shift
          ;;
        --no-wait-ci)
          wait_ci=0
          shift
          ;;
        --ci-timeout-seconds)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --ci-timeout-seconds" >&2; exit 2; }
          ci_timeout_seconds="$2"
          shift 2
          ;;
        --ci-poll-interval-seconds)
          [[ $# -ge 2 ]] || { echo "[release-automation] missing value for --ci-poll-interval-seconds" >&2; exit 2; }
          ci_poll_interval_seconds="$2"
          shift 2
          ;;
        *)
          echo "[release-automation] unknown option for publish mode: $1" >&2
          usage
          exit 2
          ;;
      esac
    done

    [[ -n "$version" ]] || { echo "[release-automation] publish mode requires --version" >&2; exit 2; }

    primary_branch="$(detect_primary_branch)"
    current_branch="$(git rev-parse --abbrev-ref HEAD)"
    if [[ -z "$push_branch" ]]; then
      push_branch="$current_branch"
    fi
    if [[ "$push_branch" != "$primary_branch" ]]; then
      echo "[release-automation] publish mode must run from the primary branch ($primary_branch). Current push branch: $push_branch" >&2
      exit 1
    fi

    repo_tag="$(normalize_field "$version" repo_version)"
    package_version="$(normalize_field "$version" package_version)"
    if [[ -z "$commit_message" ]]; then
      commit_message="chore: prepare release ${package_version}"
    fi

    ./scripts/release_ready.sh --version "$version" "${ready_args[@]}"

    ensure_git_identity

    git add pyproject.toml research_skills/__init__.py research-paper-workflow/VERSION skills/registry.yaml skills "release/${repo_tag}.md"
    if ! git diff --cached --quiet; then
      git commit -m "$commit_message"
    else
      echo "[release-automation] no staged release-prep changes to commit; continuing"
    fi

    if git rev-parse -q --verify "refs/tags/$repo_tag" >/dev/null; then
      echo "[release-automation] tag already exists locally: $repo_tag" >&2
      exit 1
    fi

    git tag -a "$repo_tag" -m "$tag_message"
    git push "$push_remote" "$push_branch" "$repo_tag"

    if [[ "$wait_ci" -eq 1 ]]; then
      post_args+=(--wait-ci --ci-timeout-seconds "$ci_timeout_seconds" --ci-poll-interval-seconds "$ci_poll_interval_seconds")
    fi
    if [[ "$create_release" -eq 1 ]]; then
      post_args+=(--create-release)
    fi

    ./scripts/release_postflight.sh --tag "$repo_tag" "${post_args[@]}"
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
