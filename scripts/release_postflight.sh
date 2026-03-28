#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TAG=""
REPO_SLUG=""
SKIP_REMOTE=0
SKIP_CI=0
CREATE_RELEASE=0
ACCEPTANCE_OUT=""
CI_STATUS="unknown"
REQUIRED_WORKFLOWS=("CI" "Install Check")
WAIT_CI=0
CI_TIMEOUT_SECONDS=1800
CI_POLL_INTERVAL_SECONDS=30

usage() {
  cat <<'EOF'
Usage:
  ./scripts/release_postflight.sh --tag <tag> [options]

Description:
  Run standardized post-release checks:
    1) verify local/remote tag consistency
    2) verify release notes + rollback docs
    3) optionally check CI status on GitHub Actions
    4) generate release acceptance receipt from template

Options:
  --tag <tag>           Required release tag (for example v0.1.0 or v0.1.1-beta.1).
  --repo <owner/repo>   Optional GitHub repo slug. Auto-derived from origin if omitted.
  --skip-remote         Skip remote ref checks.
  --skip-ci-status      Skip GitHub Actions status check.
  --wait-ci             Wait for required GitHub Actions workflows to complete successfully.
  --ci-timeout-seconds <n>
                        Max time to wait for CI when --wait-ci is enabled (default: 1800).
  --ci-poll-interval-seconds <n>
                        Poll interval for CI checks when --wait-ci is enabled (default: 30).
  --create-release      Create GitHub release if missing and gh auth is available.
  --acceptance-out <p>  Output path for acceptance receipt (default: release/acceptance/<tag>-receipt.md).
  -h, --help            Show this message.
EOF
}

derive_repo_slug() {
  local remote_url
  remote_url="$(git remote get-url origin 2>/dev/null || true)"
  if [[ "$remote_url" =~ github\.com[:/]([^/]+)/([^/.]+)(\.git)?$ ]]; then
    printf '%s/%s\n' "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}"
    return 0
  fi
  return 1
}

detect_primary_branch() {
  local candidate
  for candidate in main master; do
    if git show-ref --verify --quiet "refs/heads/$candidate"; then
      printf '%s\t%s\n' "$candidate" "$candidate"
      return 0
    fi
    if git show-ref --verify --quiet "refs/remotes/origin/$candidate"; then
      printf '%s\trefs/remotes/origin/%s\n' "$candidate" "$candidate"
      return 0
    fi
  done
  return 1
}

is_prerelease_tag() {
  [[ "$1" == *beta* || "$1" =~ b[0-9]+ ]]
}

query_ci_status() {
  local repo_slug="$1"
  local branch="$2"
  local commit="$3"
  local api_url ci_json ci_json_file
  local -a curl_cmd

  if [[ -z "$repo_slug" ]]; then
    printf 'skipped:no-repo-slug\n'
    return 0
  fi

  api_url="https://api.github.com/repos/${repo_slug}/actions/runs?branch=${branch}&per_page=20"
  curl_cmd=(curl -fsSL "$api_url")
  if [[ -n "${GH_TOKEN:-}" ]]; then
    curl_cmd=(curl -fsSL -H "Authorization: Bearer ${GH_TOKEN}" "$api_url")
  fi

  if ! ci_json="$("${curl_cmd[@]}" 2>/dev/null)"; then
    printf 'skipped:request-failed\n'
    return 0
  fi

  ci_json_file="$(mktemp)"
  printf '%s' "$ci_json" >"$ci_json_file"

  set +e
  python3 - "$ci_json_file" "$commit" "${REQUIRED_WORKFLOWS[@]}" <<'PY'
import json
import sys

payload_path = sys.argv[1]
commit = sys.argv[2]
required = sys.argv[3:]
with open(payload_path, "r", encoding="utf-8") as fh:
    raw = fh.read().strip()
if not raw:
    print("skipped:empty-response")
    raise SystemExit(0)

try:
    payload = json.loads(raw)
except json.JSONDecodeError:
    print("skipped:invalid-json")
    raise SystemExit(0)

runs = payload.get("workflow_runs", [])
results = []
pending = []
failed = []
missing = []

for workflow_name in required:
    matches = [r for r in runs if r.get("head_sha") == commit and r.get("name") == workflow_name]
    if not matches:
        missing.append(workflow_name)
        continue
    latest = sorted(matches, key=lambda r: r.get("created_at", ""), reverse=True)[0]
    status = latest.get("status") or "unknown"
    conclusion = latest.get("conclusion") or "unknown"
    html_url = latest.get("html_url") or ""
    results.append(f"{workflow_name}={status}/{conclusion}:{html_url}")
    if conclusion == "success":
        continue
    if status != "completed":
        pending.append(workflow_name)
        continue
    failed.append(workflow_name)

if failed:
    print("failed:" + "; ".join(results))
    raise SystemExit(1)
if pending or missing:
    labels = []
    if pending:
        labels.append("pending=" + ",".join(sorted(pending)))
    if missing:
        labels.append("missing=" + ",".join(sorted(missing)))
    print("pending:" + "; ".join(labels + results))
    raise SystemExit(0)
print("success:" + "; ".join(results))
raise SystemExit(0)
PY
  local status=$?
  set -e
  rm -f "$ci_json_file"
  return "$status"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      [[ $# -ge 2 ]] || { echo "[postflight] missing value for --tag" >&2; exit 2; }
      TAG="$2"
      shift 2
      ;;
    --repo)
      [[ $# -ge 2 ]] || { echo "[postflight] missing value for --repo" >&2; exit 2; }
      REPO_SLUG="$2"
      shift 2
      ;;
    --skip-remote)
      SKIP_REMOTE=1
      shift
      ;;
    --skip-ci-status)
      SKIP_CI=1
      shift
      ;;
    --wait-ci)
      WAIT_CI=1
      shift
      ;;
    --ci-timeout-seconds)
      [[ $# -ge 2 ]] || { echo "[postflight] missing value for --ci-timeout-seconds" >&2; exit 2; }
      CI_TIMEOUT_SECONDS="$2"
      shift 2
      ;;
    --ci-poll-interval-seconds)
      [[ $# -ge 2 ]] || { echo "[postflight] missing value for --ci-poll-interval-seconds" >&2; exit 2; }
      CI_POLL_INTERVAL_SECONDS="$2"
      shift 2
      ;;
    --create-release)
      CREATE_RELEASE=1
      shift
      ;;
    --acceptance-out)
      [[ $# -ge 2 ]] || { echo "[postflight] missing value for --acceptance-out" >&2; exit 2; }
      ACCEPTANCE_OUT="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[postflight] unknown option: $1" >&2
      usage
      exit 2
      ;;
  esac
done

[[ -n "$TAG" ]] || { echo "[postflight] --tag is required" >&2; usage; exit 2; }

cd "$ROOT_DIR"

if ! LOCAL_TAG_COMMIT="$(git rev-parse "$TAG^{}" 2>/dev/null)"; then
  echo "[postflight] local tag not found: $TAG" >&2
  exit 1
fi
echo "[postflight] local tag commit: $LOCAL_TAG_COMMIT"

./scripts/verify_release_tag_version.sh --tag "$TAG"

if ! primary_branch_record="$(detect_primary_branch)"; then
  echo "[postflight] unable to detect primary branch (expected main or master locally or under origin/)" >&2
  exit 1
fi
PRIMARY_BRANCH="${primary_branch_record%%$'\t'*}"
PRIMARY_BRANCH_REF="${primary_branch_record#*$'\t'}"

if git merge-base --is-ancestor "$LOCAL_TAG_COMMIT" "$PRIMARY_BRANCH_REF"; then
  echo "[postflight] tag commit is reachable from $PRIMARY_BRANCH"
else
  echo "[postflight] tag commit is not reachable from $PRIMARY_BRANCH" >&2
  exit 1
fi

RELEASE_NOTE_PATH="release/${TAG}.md"
ROLLBACK_PATH="release/rollback.md"
TEMPLATE_PATH="release/templates/beta-acceptance-template.md"

[[ -f "$RELEASE_NOTE_PATH" ]] || { echo "[postflight] missing release notes: $RELEASE_NOTE_PATH" >&2; exit 1; }
[[ -f "$ROLLBACK_PATH" ]] || { echo "[postflight] missing rollback doc: $ROLLBACK_PATH" >&2; exit 1; }
[[ -f "$TEMPLATE_PATH" ]] || { echo "[postflight] missing acceptance template: $TEMPLATE_PATH" >&2; exit 1; }
echo "[postflight] release docs present"

if [[ "$SKIP_REMOTE" -eq 0 ]]; then
  if REMOTE_MAIN="$(git ls-remote --heads origin "$PRIMARY_BRANCH" 2>/dev/null | awk '{print $1}' | head -n 1)" \
    && REMOTE_TAG="$(git ls-remote --tags origin "${TAG}^{}" 2>/dev/null | awk '{print $1}' | head -n 1)"; then
    [[ -n "$REMOTE_MAIN" ]] || { echo "[postflight] remote branch not found: $PRIMARY_BRANCH" >&2; exit 1; }
    [[ -n "$REMOTE_TAG" ]] || { echo "[postflight] remote tag not found: $TAG" >&2; exit 1; }
    [[ "$REMOTE_TAG" == "$LOCAL_TAG_COMMIT" ]] || {
      echo "[postflight] remote tag commit mismatch: local=$LOCAL_TAG_COMMIT remote=$REMOTE_TAG" >&2
      exit 1
    }
    echo "[postflight] remote refs verified (branch=$PRIMARY_BRANCH, tag=$TAG)"
  else
    echo "[postflight] warning: remote check skipped (network/auth unavailable)"
    SKIP_REMOTE=1
  fi
else
  echo "[postflight] remote check skipped by flag"
fi

if [[ -z "$REPO_SLUG" ]]; then
  REPO_SLUG="$(derive_repo_slug || true)"
fi

if [[ "$SKIP_CI" -eq 0 ]]; then
  if [[ -z "$REPO_SLUG" ]]; then
    if [[ "$WAIT_CI" -eq 1 ]]; then
      echo "[postflight] cannot derive repo slug while --wait-ci is enabled" >&2
      exit 1
    fi
    echo "[postflight] warning: cannot derive repo slug, skip CI status check"
    CI_STATUS="skipped"
  else
    if [[ "$WAIT_CI" -eq 1 ]]; then
      wait_deadline=$((SECONDS + CI_TIMEOUT_SECONDS))
      while true; do
        set +e
        CI_RESULT="$(query_ci_status "$REPO_SLUG" "$PRIMARY_BRANCH" "$LOCAL_TAG_COMMIT")"
        CI_EXIT=$?
        set -e

        if [[ "$CI_EXIT" -ne 0 ]]; then
          CI_STATUS="failed"
          echo "[postflight] CI failed for release commit: $CI_RESULT" >&2
          exit 1
        fi

        if [[ "$CI_RESULT" == success:* ]]; then
          CI_STATUS="success"
          echo "[postflight] CI status: $CI_RESULT"
          break
        fi

        if [[ "$CI_RESULT" == skipped:* ]]; then
          CI_STATUS="skipped"
          echo "[postflight] unable to query CI status while --wait-ci is enabled: $CI_RESULT" >&2
          exit 1
        fi

        CI_STATUS="pending"
        if (( SECONDS >= wait_deadline )); then
          echo "[postflight] timed out waiting for CI success: $CI_RESULT" >&2
          exit 1
        fi
        echo "[postflight] waiting for CI success: $CI_RESULT"
        sleep "$CI_POLL_INTERVAL_SECONDS"
      done
    else
      set +e
      CI_RESULT="$(query_ci_status "$REPO_SLUG" "$PRIMARY_BRANCH" "$LOCAL_TAG_COMMIT")"
      CI_EXIT=$?
      set -e

      if [[ "$CI_EXIT" -eq 0 ]]; then
        if [[ "$CI_RESULT" == success:* ]]; then
          CI_STATUS="success"
          echo "[postflight] CI status: $CI_RESULT"
        elif [[ "$CI_RESULT" == skipped:* ]]; then
          CI_STATUS="skipped"
          echo "[postflight] warning: unable to query CI status ($CI_RESULT)"
        else
          CI_STATUS="pending"
          echo "[postflight] warning: CI status unresolved: $CI_RESULT"
        fi
      else
        CI_STATUS="failed"
        echo "[postflight] CI failed for release commit: $CI_RESULT" >&2
        exit 1
      fi
    fi
  fi
else
  CI_STATUS="skipped"
  echo "[postflight] CI status check skipped by flag"
fi

if ! command -v gh >/dev/null 2>&1 || ! gh auth status >/dev/null 2>&1; then
  echo "[postflight] gh auth is required to verify or create the GitHub release page" >&2
  exit 1
fi

if [[ -z "$REPO_SLUG" ]]; then
  echo "[postflight] unable to derive GitHub repo slug for release-page checks" >&2
  exit 1
fi

IS_PRERELEASE=0
if is_prerelease_tag "$TAG"; then
  IS_PRERELEASE=1
fi

if release_json="$(gh release view "$TAG" --repo "$REPO_SLUG" --json isDraft,isPrerelease,url 2>/dev/null)"; then
  set +e
  release_state="$(RELEASE_JSON="$release_json" python3 - "$IS_PRERELEASE" <<'PY'
import json
import os
import sys

expected_prerelease = sys.argv[1] == "1"
payload = json.loads(os.environ["RELEASE_JSON"])
is_draft = bool(payload.get("isDraft"))
is_prerelease = bool(payload.get("isPrerelease"))
url = payload.get("url") or ""
if is_draft:
    print(f"draft:{url}")
    raise SystemExit(1)
if is_prerelease != expected_prerelease:
    kind = "prerelease" if is_prerelease else "stable"
    expected = "prerelease" if expected_prerelease else "stable"
    print(f"mismatch:{kind}:{expected}:{url}")
    raise SystemExit(1)
print(f"ok:{url}")
PY
  )"
  release_state_exit=$?
  set -e
  if [[ "$release_state_exit" -ne 0 ]]; then
    echo "[postflight] invalid GitHub release state for $TAG: $release_state" >&2
    exit 1
  fi
  echo "[postflight] GitHub release exists: ${release_state#ok:}"
elif [[ "$CREATE_RELEASE" -eq 1 ]]; then
  release_args=(
    "$TAG"
    --repo "$REPO_SLUG"
    --title "$TAG"
    --notes-file "$RELEASE_NOTE_PATH"
  )
  if [[ "$IS_PRERELEASE" -eq 1 ]]; then
    release_args+=(--prerelease)
  fi
  release_args+=(dist/*)
  gh release create "${release_args[@]}"
  if [[ "$IS_PRERELEASE" -eq 1 ]]; then
    echo "[postflight] GitHub prerelease created: $TAG"
  else
    echo "[postflight] GitHub release created: $TAG"
  fi
else
  echo "[postflight] missing GitHub release page for $TAG (rerun with --create-release)" >&2
  exit 1
fi

if [[ -z "$ACCEPTANCE_OUT" ]]; then
  ACCEPTANCE_OUT="release/acceptance/${TAG}-receipt.md"
fi
mkdir -p "$(dirname "$ACCEPTANCE_OUT")"

RELEASE_DATE="$(date +%F)"
python3 - "$TEMPLATE_PATH" "$ACCEPTANCE_OUT" "$TAG" "$RELEASE_DATE" "$LOCAL_TAG_COMMIT" "$CI_STATUS" <<'PY'
import sys

template_path, out_path, tag, date, commit, ci_status = sys.argv[1:]
with open(template_path, "r", encoding="utf-8") as f:
    content = f.read()
content = (
    content.replace("{{TAG}}", tag)
    .replace("{{DATE}}", date)
    .replace("{{COMMIT}}", commit)
    .replace("{{CI_STATUS}}", ci_status)
)
with open(out_path, "w", encoding="utf-8") as f:
    f.write(content)
PY
echo "[postflight] acceptance receipt generated: $ACCEPTANCE_OUT"
echo "[postflight] completed"
