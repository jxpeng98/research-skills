#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TAG=""
FROM_TAG=""
OUTPUT=""
RELEASE_DATE="$(date +%F)"
STAGE=""
OVERWRITE=0
MAX_COMMITS=12
UPDATE_EXISTING=0
VALIDATOR_RESULT="TODO"
UNITTEST_RESULT="TODO"
SMOKE_RESULT="TODO"

usage() {
  cat <<'EOF'
Usage:
  ./scripts/generate_release_notes.sh --tag <tag> [options]

Description:
  Generate a draft release notes file at release/<tag>.md.

Options:
  --tag <tag>           Required release tag (for example v0.1.0 or v0.1.1-beta.1)
  --from-tag <tag>      Optional baseline tag for commit highlights
  --output <path>       Output file path (default: release/<tag>.md)
  --date <YYYY-MM-DD>   Release date (default: today)
  --stage <name>        Optional stage label (default: inferred from tag)
  --max-commits <n>     Max commits to include in highlights (default: 12)
  --validator-result <s>  Value for validator evidence line
  --unittest-result <s>   Value for unittest evidence line
  --smoke-result <s>      Value for smoke evidence line
  --update-existing       Update evidence lines in existing release note
  --overwrite           Overwrite output if it already exists
  -h, --help            Show help
EOF
}

latest_prior_tag() {
  git tag -l 'v*' --sort=-v:refname | while read -r item; do
    [[ "$item" == "$TAG" ]] && continue
    [[ -z "$item" ]] && continue
    echo "$item"
    break
  done
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      [[ $# -ge 2 ]] || { echo "[notes] missing value for --tag" >&2; exit 2; }
      TAG="$2"
      shift 2
      ;;
    --from-tag)
      [[ $# -ge 2 ]] || { echo "[notes] missing value for --from-tag" >&2; exit 2; }
      FROM_TAG="$2"
      shift 2
      ;;
    --output)
      [[ $# -ge 2 ]] || { echo "[notes] missing value for --output" >&2; exit 2; }
      OUTPUT="$2"
      shift 2
      ;;
    --date)
      [[ $# -ge 2 ]] || { echo "[notes] missing value for --date" >&2; exit 2; }
      RELEASE_DATE="$2"
      shift 2
      ;;
    --stage)
      [[ $# -ge 2 ]] || { echo "[notes] missing value for --stage" >&2; exit 2; }
      STAGE="$2"
      shift 2
      ;;
    --max-commits)
      [[ $# -ge 2 ]] || { echo "[notes] missing value for --max-commits" >&2; exit 2; }
      MAX_COMMITS="$2"
      shift 2
      ;;
    --validator-result)
      [[ $# -ge 2 ]] || { echo "[notes] missing value for --validator-result" >&2; exit 2; }
      VALIDATOR_RESULT="$2"
      shift 2
      ;;
    --unittest-result)
      [[ $# -ge 2 ]] || { echo "[notes] missing value for --unittest-result" >&2; exit 2; }
      UNITTEST_RESULT="$2"
      shift 2
      ;;
    --smoke-result)
      [[ $# -ge 2 ]] || { echo "[notes] missing value for --smoke-result" >&2; exit 2; }
      SMOKE_RESULT="$2"
      shift 2
      ;;
    --update-existing)
      UPDATE_EXISTING=1
      shift
      ;;
    --overwrite)
      OVERWRITE=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[notes] unknown option: $1" >&2
      usage
      exit 2
      ;;
  esac
done

[[ -n "$TAG" ]] || { echo "[notes] --tag is required" >&2; usage; exit 2; }

cd "$ROOT_DIR"

if [[ -z "$OUTPUT" ]]; then
  OUTPUT="release/${TAG}.md"
fi

if [[ -z "$STAGE" ]]; then
  if [[ "$TAG" == *beta* || "$TAG" =~ b[0-9]+ ]]; then
    STAGE="Beta"
  else
    STAGE="Stable"
  fi
fi

if [[ "$UPDATE_EXISTING" -eq 1 && -e "$OUTPUT" ]]; then
  python3 - "$OUTPUT" "$VALIDATOR_RESULT" "$UNITTEST_RESULT" "$SMOKE_RESULT" <<'PY'
import re
import sys

path, validator, unittest, smoke = sys.argv[1:]
with open(path, "r", encoding="utf-8") as f:
    content = f.read()

patterns = [
    (
        r"- `validate_research_standard\.py --strict`: `[^`]*`",
        f"- `validate_research_standard.py --strict`: `{validator}`",
    ),
    (
        r"- `unittest`: `[^`]*`",
        f"- `unittest`: `{unittest}`",
    ),
    (
        r"- `run_beta_smoke\.sh`: `[^`]*`",
        f"- `run_beta_smoke.sh`: `{smoke}`",
    ),
]
for pattern, replacement in patterns:
    if re.search(pattern, content):
        content = re.sub(pattern, replacement, content, count=1)

with open(path, "w", encoding="utf-8") as f:
    f.write(content)
PY
  echo "[notes] updated evidence lines: $OUTPUT"
  exit 0
fi

if [[ -e "$OUTPUT" && "$OVERWRITE" -ne 1 ]]; then
  echo "[notes] skip existing file: $OUTPUT (use --overwrite)"
  exit 0
fi

if [[ -z "$FROM_TAG" ]]; then
  FROM_TAG="$(latest_prior_tag || true)"
fi

if [[ -n "$FROM_TAG" ]] && ! git rev-parse -q --verify "refs/tags/$FROM_TAG" >/dev/null; then
  echo "[notes] from-tag not found: $FROM_TAG" >&2
  exit 1
fi

COMMITS=""
if [[ -n "$FROM_TAG" ]]; then
  COMMITS="$(git log --no-merges --pretty='- %s (%h)' "${FROM_TAG}..HEAD" | head -n "$MAX_COMMITS" || true)"
fi
if [[ -z "$COMMITS" ]]; then
  COMMITS="$(git log --no-merges --pretty='- %s (%h)' -n "$MAX_COMMITS" || true)"
fi
if [[ -z "$COMMITS" ]]; then
  COMMITS="- Draft highlights pending."
fi

mkdir -p "$(dirname "$OUTPUT")"

{
  echo "# research-skills \`${TAG}\` Release Notes"
  echo
  echo "Date: ${RELEASE_DATE}"
  echo "Stage: ${STAGE}"
  echo
  echo "## Highlights (Draft)"
  echo
  echo "${COMMITS}"
  echo
  echo "## Validation Evidence"
  echo
  echo "Executed locally on ${RELEASE_DATE}:"
  echo
  cat <<'EOF'
```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest tests.test_orchestrator_workflows -v
./scripts/run_beta_smoke.sh
```
EOF
  echo
  cat <<'EOF'
Observed result:

- `validate_research_standard.py --strict`: `${VALIDATOR_RESULT}`
- `unittest`: `${UNITTEST_RESULT}`
- `run_beta_smoke.sh`: `${SMOKE_RESULT}`
EOF
  echo
  echo "## Publish Steps"
  echo
  cat <<EOF
\`\`\`bash
./scripts/release_automation.sh pre --tag ${TAG}
git tag -a ${TAG} -m "research-skills release"
git push origin ${TAG}
./scripts/release_automation.sh post --tag ${TAG}
\`\`\`
EOF
  echo
  echo "For rollback procedure, see \`release/rollback.md\`."
  if [[ -n "$FROM_TAG" ]]; then
    echo
    echo "_Baseline tag for highlights: \`${FROM_TAG}\`_"
  fi
} >"$OUTPUT"

echo "[notes] generated: $OUTPUT"
