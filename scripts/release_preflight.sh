#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
RUN_SMOKE=1
MAINTAINER_SMOKE=0
STRICT_MODE=1
TAG=""
SKIP_NOTE_GEN=0
NOTE_OVERWRITE=0
FROM_TAG=""

is_prerelease_tag() {
  [[ "$1" == *beta* || "$1" =~ b[0-9]+ ]]
}

usage() {
  cat <<'EOF'
Usage:
  ./scripts/release_preflight.sh [--tag <tag>] [options]

Description:
  Run standardized pre-release gates:
    0) prerelease: auto-generate release/<tag>.md draft
       stable: verify matching CHANGELOG.md section exists
    1) strict standard validator
    2) repository unit tests
    3) release smoke tier (literature pipeline + doctor)
    4) optional maintainer smoke tier (parallel + task-run profile paths)

Options:
  --tag <tag>     Optional release tag to pre-check. If provided, script verifies
                  the tag does not already exist locally.
  --from-tag <t>  Optional baseline tag passed to prerelease note generator.
  --skip-note-gen Skip auto generation of release/<tag>.md draft for prerelease tags.
  --note-overwrite  Overwrite release/<tag>.md when auto-generating prerelease draft.
  --skip-smoke    Skip smoke test stage.
  --maintainer-smoke  Run maintainer smoke tier instead of release smoke tier.
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
    --from-tag)
      [[ $# -ge 2 ]] || { echo "[preflight] missing value for --from-tag" >&2; exit 2; }
      FROM_TAG="$2"
      shift 2
      ;;
    --skip-note-gen)
      SKIP_NOTE_GEN=1
      shift
      ;;
    --note-overwrite)
      NOTE_OVERWRITE=1
      shift
      ;;
    --skip-smoke)
      RUN_SMOKE=0
      shift
      ;;
    --maintainer-smoke)
      MAINTAINER_SMOKE=1
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

  if is_prerelease_tag "$TAG"; then
    if [[ "$SKIP_NOTE_GEN" -eq 0 ]]; then
      note_cmd=(./scripts/generate_release_notes.sh --tag "$TAG")
      if [[ -n "$FROM_TAG" ]]; then
        note_cmd+=(--from-tag "$FROM_TAG")
      fi
      if [[ "$NOTE_OVERWRITE" -eq 1 ]]; then
        note_cmd+=(--overwrite)
      fi
      echo "[preflight] prerelease note draft"
      "${note_cmd[@]}"
    else
      echo "[preflight] prerelease note generation skipped"
    fi

    if [[ ! -f "release/${TAG}.md" ]]; then
      echo "[preflight] missing prerelease notes file: release/${TAG}.md" >&2
      exit 1
    fi
  else
    echo "[preflight] validating stable release changelog entry"
    python3 scripts/changelog_section.py --version "${TAG#v}" --check
  fi
fi

validate_cmd=(python3 scripts/validate_research_standard.py)
if [[ "$STRICT_MODE" -eq 1 ]]; then
  validate_cmd+=(--strict)
fi

validator_log="$(mktemp -t research-skills-validator.XXXXXX.log)"
unit_log="$(mktemp -t research-skills-unittest.XXXXXX.log)"
smoke_log="$(mktemp -t research-skills-smoke.XXXXXX.log)"
trap 'rm -f "$validator_log" "$unit_log" "$smoke_log"' EXIT

echo "[preflight] validator"
"${validate_cmd[@]}" 2>&1 | tee "$validator_log"
validator_summary="$(grep '^Summary:' "$validator_log" | tail -n1 || true)"
if [[ -z "$validator_summary" ]]; then
  validator_summary="passed"
fi

echo "[preflight] unit tests"
python3 -m unittest discover -s tests -v 2>&1 | tee "$unit_log"
unit_ran_line="$(grep -E '^Ran [0-9]+ tests? in ' "$unit_log" | tail -n1 || true)"
if grep -q '^OK$' "$unit_log"; then
  if [[ -n "$unit_ran_line" ]]; then
    unittest_summary="${unit_ran_line} ... OK"
  else
    unittest_summary="OK"
  fi
else
  unittest_summary="FAILED"
fi

if [[ "$RUN_SMOKE" -eq 1 ]]; then
  smoke_tier="release"
  if [[ "$MAINTAINER_SMOKE" -eq 1 ]]; then
    smoke_tier="maintainer"
  fi
  echo "[preflight] smoke (${smoke_tier} tier)"
  ./scripts/run_beta_smoke.sh --tier "$smoke_tier" 2>&1 | tee "$smoke_log"
  if grep -q '\[smoke\] passed' "$smoke_log"; then
    smoke_summary="passed (${smoke_tier}-tier)"
  else
    smoke_summary="completed (${smoke_tier}-tier)"
  fi
else
  echo "[preflight] smoke skipped"
  smoke_summary="skipped"
fi

if [[ -n "$TAG" && "$SKIP_NOTE_GEN" -eq 0 ]] && is_prerelease_tag "$TAG"; then
  update_note_cmd=(
    ./scripts/generate_release_notes.sh
    --tag "$TAG"
    --update-existing
    --validator-result "$validator_summary"
    --unittest-result "$unittest_summary"
    --smoke-result "$smoke_summary"
  )
  if [[ -n "$FROM_TAG" ]]; then
    update_note_cmd+=(--from-tag "$FROM_TAG")
  fi
  echo "[preflight] release note evidence update"
  "${update_note_cmd[@]}"
fi

echo "[preflight] all checks passed"
if [[ -n "$TAG" ]]; then
  echo "[preflight] next publish commands:"
  echo "  git tag -a $TAG -m \"research-skills release\""
  echo "  git push origin $TAG"
fi
