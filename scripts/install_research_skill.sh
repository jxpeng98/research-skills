#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TARGET="all"
MODE="copy"
PROJECT_DIR="$(pwd)"
OVERWRITE=0
DRY_RUN=0
RUN_DOCTOR=0

usage() {
  cat <<'EOF'
Usage:
  ./scripts/install_research_skill.sh [options]

Options:
  --target <codex|claude|gemini|all>   Install target (default: all)
  --mode <copy|link>                   Install mode (default: copy)
  --project-dir <path>                 Project directory for command/workflow integration (default: current dir)
  --overwrite                           Overwrite existing installed files
  --doctor                              Run orchestrator doctor after install
  --dry-run                             Print actions without writing files
  -h, --help                            Show help

Environment overrides:
  CODEX_HOME        Default: $HOME/.codex
  CLAUDE_CODE_HOME  Default: $HOME/.claude
  GEMINI_HOME       Default: $HOME/.gemini
EOF
}

log() {
  echo "[install] $*"
}

run_cmd() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf '[dry-run]'
    printf ' %q' "$@"
    printf '\n'
    return 0
  fi
  "$@"
}

ensure_dir() {
  run_cmd mkdir -p "$1"
}

resolve_abs() {
  python3 -c 'import os,sys; print(os.path.abspath(os.path.expanduser(sys.argv[1])))' "$1"
}

copy_item() {
  local src="$1"
  local dest="$2"
  local src_abs dest_abs
  src_abs="$(resolve_abs "$src")"
  dest_abs="$(resolve_abs "$dest")"

  if [[ "$src_abs" == "$dest_abs" ]]; then
    log "skip same path: $dest_abs"
    return 0
  fi

  if [[ -e "$dest" || -L "$dest" ]]; then
    if [[ "$OVERWRITE" -ne 1 ]]; then
      log "skip exists (use --overwrite): $dest"
      return 0
    fi
    run_cmd rm -rf "$dest"
  fi

  ensure_dir "$(dirname "$dest")"

  if [[ "$MODE" == "link" ]]; then
    run_cmd ln -s "$src_abs" "$dest"
  else
    run_cmd cp -R "$src_abs" "$dest"
  fi
  log "installed: $dest"
}

copy_workflows_for_claude() {
  local workflows_src="$ROOT_DIR/.agent/workflows"
  local workflows_dest="$PROJECT_DIR/.agent/workflows"
  ensure_dir "$workflows_dest"
  local workflow_file
  for workflow_file in "$workflows_src"/*.md; do
    copy_item "$workflow_file" "$workflows_dest/$(basename "$workflow_file")"
  done
}

write_gemini_quickstart() {
  local quickstart_dir="$PROJECT_DIR/.gemini"
  local quickstart_path="$quickstart_dir/research-skills.md"
  local profile_dest="$quickstart_dir/agent-profiles.example.json"

  if [[ -e "$quickstart_path" && "$OVERWRITE" -ne 1 ]]; then
    log "skip exists (use --overwrite): $quickstart_path"
  else
    ensure_dir "$quickstart_dir"
    if [[ "$DRY_RUN" -eq 1 ]]; then
      printf '[dry-run] write %q\n' "$quickstart_path"
    else
      cat >"$quickstart_path" <<'EOF'
# Research Skills for Gemini Runtime

Use this project through orchestrator for Codex/Claude/Gemini collaboration:

```bash
python3 -m bridges.orchestrator doctor --cwd .
python3 -m bridges.orchestrator parallel --prompt "Analyze this study design" --cwd . --summarizer gemini
python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic your-topic --cwd . --triad
```
EOF
    fi
    log "installed: $quickstart_path"
  fi

  copy_item "$ROOT_DIR/standards/agent-profiles.example.json" "$profile_dest"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      [[ $# -ge 2 ]] || { echo "Missing value for --target" >&2; exit 2; }
      TARGET="$2"
      shift 2
      ;;
    --mode)
      [[ $# -ge 2 ]] || { echo "Missing value for --mode" >&2; exit 2; }
      MODE="$2"
      shift 2
      ;;
    --project-dir)
      [[ $# -ge 2 ]] || { echo "Missing value for --project-dir" >&2; exit 2; }
      PROJECT_DIR="$2"
      shift 2
      ;;
    --overwrite)
      OVERWRITE=1
      shift
      ;;
    --doctor)
      RUN_DOCTOR=1
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

case "$TARGET" in
  codex|claude|gemini|all) ;;
  *)
    echo "Unsupported target: $TARGET" >&2
    usage
    exit 2
    ;;
esac

case "$MODE" in
  copy|link) ;;
  *)
    echo "Unsupported mode: $MODE" >&2
    usage
    exit 2
    ;;
esac

PROJECT_DIR="$(resolve_abs "$PROJECT_DIR")"
SKILL_SRC="$ROOT_DIR/research-paper-workflow"
if [[ ! -f "$SKILL_SRC/SKILL.md" ]]; then
  echo "Missing skill source: $SKILL_SRC/SKILL.md" >&2
  exit 1
fi

CODEX_SKILL_DEST="${CODEX_HOME:-$HOME/.codex}/skills/research-paper-workflow"
CLAUDE_SKILL_DEST="${CLAUDE_CODE_HOME:-$HOME/.claude}/skills/research-paper-workflow"
GEMINI_SKILL_DEST="${GEMINI_HOME:-$HOME/.gemini}/skills/research-paper-workflow"

log "source skill: $SKILL_SRC"
log "project dir: $PROJECT_DIR"
log "target: $TARGET"
log "mode: $MODE"

if [[ "$TARGET" == "codex" || "$TARGET" == "all" ]]; then
  log "installing Codex skill"
  copy_item "$SKILL_SRC" "$CODEX_SKILL_DEST"
fi

if [[ "$TARGET" == "claude" || "$TARGET" == "all" ]]; then
  log "installing Claude skill bundle"
  copy_item "$SKILL_SRC" "$CLAUDE_SKILL_DEST"
  copy_workflows_for_claude
  if [[ -f "$PROJECT_DIR/CLAUDE.md" && "$OVERWRITE" -ne 1 ]]; then
    copy_item "$ROOT_DIR/CLAUDE.md" "$PROJECT_DIR/CLAUDE.research-skills.md"
  else
    copy_item "$ROOT_DIR/CLAUDE.md" "$PROJECT_DIR/CLAUDE.md"
  fi
fi

if [[ "$TARGET" == "gemini" || "$TARGET" == "all" ]]; then
  log "installing Gemini skill bundle"
  copy_item "$SKILL_SRC" "$GEMINI_SKILL_DEST"
  write_gemini_quickstart
fi

if [[ "$RUN_DOCTOR" -eq 1 ]]; then
  log "running orchestrator doctor"
  run_cmd python3 -m bridges.orchestrator doctor --cwd "$PROJECT_DIR"
fi

log "done"
log "restart Codex / Claude Code / Gemini CLI after installation."
