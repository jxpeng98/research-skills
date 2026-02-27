#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TMP_PROFILE="$(mktemp -t research-skills-smoke-profile.XXXXXX.json)"
trap 'rm -f "$TMP_PROFILE"' EXIT

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

echo "[smoke] doctor"
python3 -m bridges.orchestrator doctor --cwd . | rg -q "Doctor Summary"

echo "[smoke] parallel/profile path"
python3 -m bridges.orchestrator parallel \
  --prompt "beta smoke: parallel" \
  --cwd . \
  --profile-file "$TMP_PROFILE" \
  --profile smoke-fast \
  --summarizer-profile smoke-fast \
  --summarizer claude | rg -q "Parallel Execution"

echo "[smoke] task-run/profile path"
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
  --triad | rg -q "Agent Profiles"

echo "[smoke] passed"
