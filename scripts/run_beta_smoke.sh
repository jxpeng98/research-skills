#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TMP_PROFILE="$(mktemp -t research-skills-smoke-profile.XXXXXX.json)"
trap 'rm -f "$TMP_PROFILE"' EXIT

contains_text() {
  local pattern="$1"
  local text="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -q "$pattern" <<<"$text"
  else
    grep -q "$pattern" <<<"$text"
  fi
}

run_and_assert_output() {
  local label="$1"
  local pattern="$2"
  shift 2

  local output
  if ! output="$("$@" 2>&1)"; then
    echo "[smoke] ${label} failed" >&2
    echo "$output" >&2
    return 1
  fi

  if ! contains_text "$pattern" "$output"; then
    echo "[smoke] ${label} output missing pattern: $pattern" >&2
    echo "$output" >&2
    return 1
  fi
}

cat >"$TMP_PROFILE" <<'JSON'
{
  "profiles": {
    "smoke-fast": {
      "persona": "Smoke test profile for fast non-interactive checks.",
      "analysis_style": "Return concise diagnostics.",
      "draft_style": "Return concise diagnostics.",
      "review_style": "Return concise diagnostics.",
      "summary_style": "Return concise diagnostics.",
      "triad_style": "Return concise diagnostics.",
      "runtime_options": {
        "codex": {
          "non_interactive": true,
          "require_api_key": true,
          "timeout_seconds": 5
        },
        "claude": {
          "non_interactive": true,
          "require_api_key": true,
          "timeout_seconds": 5
        },
        "gemini": {
          "non_interactive": true,
          "require_api_key": true,
          "timeout_seconds": 5
        }
      }
    }
  }
}
JSON

cd "$ROOT_DIR"

echo "[smoke] literature pipeline"
# Success marker expected from the delegated smoke script: [literature-smoke] passed
literature_output="$(./scripts/run_literature_smoke.sh 2>&1)"
echo "$literature_output"
if ! contains_text "\\[literature-smoke\\] passed" "$literature_output"; then
  echo "[smoke] literature pipeline missing success marker" >&2
  exit 1
fi

echo "[smoke] doctor"
run_and_assert_output "doctor" "Doctor Summary" \
  python3 -m bridges.orchestrator doctor --cwd .

echo "[smoke] parallel/profile path"
run_and_assert_output "parallel" "Parallel Execution" \
  python3 -m bridges.orchestrator parallel \
  --prompt "beta smoke: parallel" \
  --cwd . \
  --profile-file "$TMP_PROFILE" \
  --profile smoke-fast \
  --summarizer-profile smoke-fast \
  --summarizer claude

echo "[smoke] task-run/profile path"
run_and_assert_output "task-run" "Agent Profiles" \
  python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic beta-smoke \
  --cwd . \
  --profile-file "$TMP_PROFILE" \
  --profile smoke-fast \
  --draft-profile smoke-fast \
  --review-profile smoke-fast \
  --triad-profile smoke-fast \
  --triad

echo "[smoke] passed"
