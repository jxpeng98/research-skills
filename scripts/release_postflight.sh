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

usage() {
  cat <<'EOF'
Usage:
  ./scripts/release_postflight.sh --tag <tag> [options]

Description:
  Run standardized post-release checks:
    1) verify local/remote tag consistency
    2) verify release notes + rollback docs
    3) optionally check CI status on GitHub Actions
    4) generate beta acceptance receipt from template

Options:
  --tag <tag>           Required release tag (for example v0.1.0-beta.1).
  --repo <owner/repo>   Optional GitHub repo slug. Auto-derived from origin if omitted.
  --skip-remote         Skip remote ref checks.
  --skip-ci-status      Skip GitHub Actions status check.
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

PRIMARY_BRANCH="main"
if ! git show-ref --verify --quiet "refs/heads/$PRIMARY_BRANCH"; then
  PRIMARY_BRANCH="master"
fi

if git merge-base --is-ancestor "$LOCAL_TAG_COMMIT" "$PRIMARY_BRANCH"; then
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
    echo "[postflight] warning: cannot derive repo slug, skip CI status check"
    CI_STATUS="skipped"
  else
    API_URL="https://api.github.com/repos/${REPO_SLUG}/actions/runs?branch=${PRIMARY_BRANCH}&per_page=20"
    CURL_CMD=(curl -fsSL "$API_URL")
    if [[ -n "${GH_TOKEN:-}" ]]; then
      CURL_CMD=(curl -fsSL -H "Authorization: Bearer ${GH_TOKEN}" "$API_URL")
    fi
    if CI_JSON="$("${CURL_CMD[@]}" 2>/dev/null)"; then
      set +e
      CI_RESULT="$(CI_JSON_PAYLOAD="$CI_JSON" python3 - "$LOCAL_TAG_COMMIT" <<'PY'
import json
import os
import sys

commit = sys.argv[1]
raw = os.environ.get("CI_JSON_PAYLOAD", "").strip()
if not raw:
    print("skipped:empty-response")
    raise SystemExit(0)

try:
    payload = json.loads(raw)
except json.JSONDecodeError:
    print("skipped:invalid-json")
    raise SystemExit(0)

runs = payload.get("workflow_runs", [])
matches = [r for r in runs if r.get("head_sha") == commit and r.get("name") == "CI"]
if not matches:
    print("skipped:no-ci-run-found")
    raise SystemExit(0)

latest = sorted(matches, key=lambda r: r.get("created_at", ""), reverse=True)[0]
status = latest.get("status") or "unknown"
conclusion = latest.get("conclusion") or "unknown"
html_url = latest.get("html_url") or ""
if conclusion == "success":
    print(f"success:{status}:{conclusion}:{html_url}")
    raise SystemExit(0)
if status != "completed":
    print(f"pending:{status}:{conclusion}:{html_url}")
    raise SystemExit(0)
print(f"failed:{status}:{conclusion}:{html_url}")
raise SystemExit(1)
PY
      )"
      CI_EXIT=$?
      set -e

      if [[ "$CI_EXIT" -eq 0 ]]; then
        if [[ "$CI_RESULT" == success:* ]]; then
          CI_STATUS="success"
          echo "[postflight] CI status: $CI_RESULT"
        else
          CI_STATUS="pending"
          echo "[postflight] warning: CI status unresolved: $CI_RESULT"
        fi
      else
        CI_STATUS="failed"
        echo "[postflight] CI failed for release commit: $CI_RESULT" >&2
        exit 1
      fi
    else
      CI_STATUS="skipped"
      echo "[postflight] warning: unable to query CI status (network/private repo/auth)"
    fi
  fi
else
  CI_STATUS="skipped"
  echo "[postflight] CI status check skipped by flag"
fi

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  if gh release view "$TAG" --repo "$REPO_SLUG" >/dev/null 2>&1; then
    echo "[postflight] GitHub release exists: $TAG"
  elif [[ "$CREATE_RELEASE" -eq 1 ]]; then
    gh release create "$TAG" \
      --repo "$REPO_SLUG" \
      --title "$TAG" \
      --notes-file "$RELEASE_NOTE_PATH"
    echo "[postflight] GitHub release created: $TAG"
  else
    echo "[postflight] warning: GitHub release not found (use --create-release to publish)"
  fi
else
  echo "[postflight] warning: gh auth unavailable, skip release-page check/create"
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
