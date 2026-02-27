#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TAG=""
FROM_TAG=""
OUTPUT=""
RELEASE_DATE="$(date +%F)"
STAGE="Beta"
OVERWRITE=0
MAX_COMMITS=12

usage() {
  cat <<'EOF'
Usage:
  ./scripts/generate_release_notes.sh --tag <tag> [options]

Description:
  Generate a draft release notes file at release/<tag>.md.

Options:
  --tag <tag>           Required release tag (for example v0.1.0-beta.3)
  --from-tag <tag>      Optional baseline tag for commit highlights
  --output <path>       Output file path (default: release/<tag>.md)
  --date <YYYY-MM-DD>   Release date (default: today)
  --stage <name>        Stage label (default: Beta)
  --max-commits <n>     Max commits to include in highlights (default: 12)
  --overwrite           Overwrite output if it already exists
  -h, --help            Show help
EOF
}

latest_beta_tag() {
  git tag -l 'v*-beta.*' --sort=-v:refname | while read -r item; do
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

if [[ -e "$OUTPUT" && "$OVERWRITE" -ne 1 ]]; then
  echo "[notes] skip existing file: $OUTPUT (use --overwrite)"
  exit 0
fi

if [[ -z "$FROM_TAG" ]]; then
  FROM_TAG="$(latest_beta_tag || true)"
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

- `validate_research_standard.py --strict`: `TODO`
- `unittest`: `TODO`
- `run_beta_smoke.sh`: `TODO`
EOF
  echo
  echo "## Publish Steps"
  echo
  cat <<EOF
\`\`\`bash
./scripts/release_automation.sh pre --tag ${TAG}
git tag -a ${TAG} -m "research-skills beta release"
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
