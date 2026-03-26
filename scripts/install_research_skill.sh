#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TARGET="all"
MODE="copy"
PROJECT_DIR="$(pwd)"
OVERWRITE=0
DRY_RUN=0
RUN_DOCTOR=0
INSTALL_CLI=0
CLI_DIR="${RESEARCH_SKILLS_BIN_DIR:-$HOME/.local/bin}"

# ── ANSI colors (respects NO_COLOR) ──────────────────────────────────────────
if [[ -z "${NO_COLOR:-}" ]] && [[ -t 1 ]]; then
  C_RESET='\033[0m'
  C_BOLD='\033[1m'
  C_DIM='\033[2m'
  C_GREEN='\033[32m'
  C_YELLOW='\033[33m'
  C_CYAN='\033[36m'
  C_RED='\033[31m'
else
  C_RESET='' C_BOLD='' C_DIM='' C_GREEN='' C_YELLOW='' C_CYAN='' C_RED=''
fi

# ── Output helpers ───────────────────────────────────────────────────────────
section() {
  printf "\n${C_BOLD}${C_CYAN}─── %s ${C_DIM}%s${C_RESET}\n" "$1" "$(printf '─%.0s' $(seq 1 $((40 - ${#1}))))"
}

ok() {
  printf "  ${C_GREEN}✓${C_RESET}  %-12s → %s\n" "$1" "$2"
}

skip() {
  printf "  ${C_DIM}○  %-12s → %s (skipped)${C_RESET}\n" "$1" "$2"
}

info() {
  printf "  ${C_DIM}%s${C_RESET}\n" "$*"
}

warn() {
  printf "  ${C_YELLOW}⚠${C_RESET}  %s\n" "$*"
}

err() {
  printf "  ${C_RED}✗${C_RESET}  %s\n" "$*" >&2
}

usage() {
  cat <<'EOF'
Usage:
  ./scripts/install_research_skill.sh [options]

Options:
  --target <codex|claude|gemini|antigravity|all> Install target (default: all)
  --mode <copy|link>                   Install mode (default: copy)
  --project-dir <path>                 Project directory for command/workflow integration (default: current dir)
  --install-cli                        Install shell CLI commands (`research-skills`, `rsk`, `rsw`)
  --no-cli                             Skip shell CLI installation
  --cli-dir <path>                     Directory for shell CLI binaries (default: RESEARCH_SKILLS_BIN_DIR or ~/.local/bin)
  --overwrite                           Overwrite existing installed files
  --doctor                              Run orchestrator doctor after install
  --dry-run                             Print actions without writing files
  -h, --help                            Show help

Environment overrides:
  CODEX_HOME        Default: $HOME/.codex
  CLAUDE_CODE_HOME  Default: $HOME/.claude
  GEMINI_HOME       Default: $HOME/.gemini
  ANTIGRAVITY_HOME  Default: $HOME/.gemini/antigravity
  RESEARCH_SKILLS_BIN_DIR Default: $HOME/.local/bin
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
  local raw="${1:-}"
  local path="$raw"
  local part
  local normalized=""

  if [[ -z "$path" ]]; then
    return 1
  fi

  case "$path" in
    "~") path="$HOME" ;;
    "~/"*) path="$HOME/${path#~/}" ;;
  esac

  if [[ "$path" != /* ]]; then
    path="$PWD/$path"
  fi

  while [[ "$path" == *"//"* ]]; do
    path="${path//\/\//\/}"
  done

  while [[ -n "$path" ]]; do
    if [[ "$path" == /* ]]; then
      path="${path#/}"
      if [[ -z "$path" ]]; then
        break
      fi
    fi

    if [[ "$path" == */* ]]; then
      part="${path%%/*}"
      path="${path#*/}"
    else
      part="$path"
      path=""
    fi

    case "$part" in
      ""|".")
        continue
        ;;
      "..")
        if [[ -n "$normalized" ]]; then
          normalized="${normalized%/*}"
        fi
        ;;
      *)
        if [[ -n "$normalized" ]]; then
          normalized="$normalized/$part"
        else
          normalized="$part"
        fi
        ;;
    esac
  done

  if [[ -z "$normalized" ]]; then
    printf '/\n'
    return 0
  fi

  printf '/%s\n' "$normalized"
}

has_python3() {
  command -v python3 >/dev/null 2>&1
}

cli_name_for_target() {
  case "$1" in
    codex) printf 'codex\n' ;;
    claude) printf 'claude\n' ;;
    gemini) printf 'gemini\n' ;;
    antigravity) printf 'antigravity\n' ;;
    *) return 1 ;;
  esac
}

check_target_cli() {
  local target="$1"
  local cli_name resolved

  cli_name="$(cli_name_for_target "$target")" || return 1
  if resolved="$(command -v "$cli_name" 2>/dev/null)"; then
    ok "CLI" "$target -> $resolved"
    return 0
  fi

  warn "$target CLI not found in PATH: $cli_name"
  return 1
}

path_contains_dir() {
  local target="$1"
  local entry
  local expanded

  IFS=':' read -r -a path_entries <<< "${PATH:-}"
  expanded="$(resolve_abs "$target")"
  for entry in "${path_entries[@]}"; do
    [[ -n "$entry" ]] || continue
    if [[ "$(resolve_abs "$entry")" == "$expanded" ]]; then
      return 0
    fi
  done
  return 1
}

# ── Core copy logic (silent – callers handle display) ────────────────────────
_copy_item() {
  local src="$1"
  local dest="$2"
  local src_abs dest_abs
  src_abs="$(resolve_abs "$src")"
  dest_abs="$(resolve_abs "$dest")"

  if [[ "$src_abs" == "$dest_abs" ]]; then
    return 1  # same-path skip
  fi

  if [[ -e "$dest" || -L "$dest" ]]; then
    if [[ "$OVERWRITE" -ne 1 ]]; then
      return 2  # exists skip
    fi
    run_cmd rm -rf "$dest"
  fi

  ensure_dir "$(dirname "$dest")"

  if [[ "$MODE" == "link" ]]; then
    run_cmd ln -s "$src_abs" "$dest"
  else
    run_cmd cp -R "$src_abs" "$dest"
  fi
  return 0
}

copy_item_display() {
  local src="$1"
  local dest="$2"
  local label="${3:-File}"
  local ret=0
  _copy_item "$src" "$dest" || ret=$?
  case $ret in
    0) ok "$label" "$dest" ;;
    1) skip "$label" "$dest (same path)" ;;
    2) skip "$label" "$dest (use --overwrite)" ;;
  esac
  return 0
}

set_executable_if_present() {
  local path="$1"
  if [[ -e "$path" && ! -L "$path" ]]; then
    run_cmd chmod +x "$path"
  fi
}

install_alias_link() {
  local target_path="$1"
  local dest="$2"

  if [[ -e "$dest" || -L "$dest" ]]; then
    if [[ "$OVERWRITE" -ne 1 ]]; then
      skip "Alias" "$dest (use --overwrite)"
      return 0
    fi
    run_cmd rm -rf "$dest"
  fi

  ensure_dir "$(dirname "$dest")"
  run_cmd ln -s "$target_path" "$dest"
  ok "Alias" "$dest"
}

install_cli_assets() {
  local cli_src="$ROOT_DIR/scripts/research_skills_cli.sh"
  local bootstrap_src="$ROOT_DIR/scripts/bootstrap_research_skill.sh"
  local cli_dest="$CLI_DIR/research-skills"
  local bootstrap_dest="$CLI_DIR/research-skills-bootstrap"

  if [[ ! -f "$cli_src" ]]; then
    err "Missing shell CLI source: $cli_src"
    return 1
  fi
  if [[ ! -f "$bootstrap_src" ]]; then
    err "Missing bootstrap source: $bootstrap_src"
    return 1
  fi

  section "Shell CLI"
  copy_item_display "$cli_src" "$cli_dest" "CLI"
  set_executable_if_present "$cli_dest"
  copy_item_display "$bootstrap_src" "$bootstrap_dest" "Bootstrap"
  set_executable_if_present "$bootstrap_dest"
  install_alias_link "$cli_dest" "$CLI_DIR/rsk"
  install_alias_link "$cli_dest" "$CLI_DIR/rsw"

  if path_contains_dir "$CLI_DIR"; then
    info "cli dir on PATH: $CLI_DIR"
  else
    warn "CLI installed to $CLI_DIR but this directory is not on PATH."
  fi
}

copy_workflows_for_claude() {
  local workflows_src="$ROOT_DIR/.agent/workflows"
  local workflows_dest="$PROJECT_DIR/.agent/workflows"
  ensure_dir "$workflows_dest"
  local count=0
  local workflow_file
  for workflow_file in "$workflows_src"/*.md; do
    local ret=0
    _copy_item "$workflow_file" "$workflows_dest/$(basename "$workflow_file")" || ret=$?
    if [[ $ret -eq 0 ]]; then
      count=$((count + 1))
    fi
  done
  if [[ $count -gt 0 ]]; then
    ok "Workflows" "$workflows_dest/ ($count files)"
  else
    skip "Workflows" "$workflows_dest/ (already up-to-date)"
  fi
}

install_project_env() {
  local env_src="$ROOT_DIR/.env.example"
  local env_dest="$PROJECT_DIR/.env"

  if [[ ! -f "$env_src" ]]; then
    warn "Missing .env template: $env_src"
    return 0
  fi

  copy_item_display "$env_src" "$env_dest" "Env"
}

write_gemini_quickstart() {
  local quickstart_dir="$PROJECT_DIR/.gemini"
  local quickstart_path="$quickstart_dir/research-skills.md"
  local profile_dest="$quickstart_dir/agent-profiles.example.json"
  local quickstart_src="$ROOT_DIR/.gemini/research-skills.md"

  if [[ -e "$quickstart_path" && "$OVERWRITE" -ne 1 ]]; then
    skip "Quickstart" "$quickstart_path (use --overwrite)"
  elif [[ -f "$quickstart_src" ]]; then
    copy_item_display "$quickstart_src" "$quickstart_path" "Quickstart"
  else
    # Fallback: write minimal stub if source template is missing
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
    ok "Quickstart" "$quickstart_path (fallback stub)"
  fi

  copy_item_display "$ROOT_DIR/standards/agent-profiles.example.json" "$profile_dest" "Profiles"
}

install_antigravity_workspace() {
  local primary_dest="$PROJECT_DIR/.agents/skills/research-paper-workflow"
  local legacy_dest="$PROJECT_DIR/.agent/skills/research-paper-workflow"
  copy_item_display "$SKILL_SRC" "$primary_dest" "Workspace Skill"
  copy_item_display "$SKILL_SRC" "$legacy_dest" "Legacy Skill"
}

# ── Parse arguments ──────────────────────────────────────────────────────────
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
    --install-cli)
      INSTALL_CLI=1
      shift
      ;;
    --no-cli)
      INSTALL_CLI=0
      shift
      ;;
    --cli-dir)
      [[ $# -ge 2 ]] || { echo "Missing value for --cli-dir" >&2; exit 2; }
      CLI_DIR="$2"
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
  codex|claude|gemini|antigravity|all) ;;
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
CLI_DIR="$(resolve_abs "$CLI_DIR")"
SKILL_SRC="$ROOT_DIR/research-paper-workflow"
if [[ ! -f "$SKILL_SRC/SKILL.md" ]]; then
  echo "Missing skill source: $SKILL_SRC/SKILL.md" >&2
  exit 1
fi

CODEX_SKILL_DEST="${CODEX_HOME:-$HOME/.codex}/skills/research-paper-workflow"
CLAUDE_SKILL_DEST="${CLAUDE_CODE_HOME:-$HOME/.claude}/skills/research-paper-workflow"
GEMINI_SKILL_DEST="${GEMINI_HOME:-$HOME/.gemini}/skills/research-paper-workflow"
ANTIGRAVITY_SKILL_DEST="${ANTIGRAVITY_HOME:-$HOME/.gemini/antigravity}/skills/research-paper-workflow"
ANTIGRAVITY_CLI_FOUND=0

# ── Header ───────────────────────────────────────────────────────────────────
printf "\n${C_BOLD}Research Skills Installer${C_RESET}\n"
info "source:  $SKILL_SRC"
info "project: $PROJECT_DIR"
info "target:  $TARGET  |  mode: $MODE"
if [[ "$INSTALL_CLI" -eq 1 ]]; then
  info "cli:     install -> $CLI_DIR"
fi

section "CLI Checks"
case "$TARGET" in
  codex)
    check_target_cli codex || true
    ;;
  claude)
    check_target_cli claude || true
    ;;
  gemini)
    check_target_cli gemini || true
    ;;
  antigravity)
    if check_target_cli antigravity; then
      ANTIGRAVITY_CLI_FOUND=1
    fi
    ;;
  all)
    check_target_cli codex || true
    check_target_cli claude || true
    check_target_cli gemini || true
    if check_target_cli antigravity; then
      ANTIGRAVITY_CLI_FOUND=1
    fi
    ;;
esac

# ── Install targets ──────────────────────────────────────────────────────────
if [[ "$TARGET" == "codex" || "$TARGET" == "all" ]]; then
  section "Codex"
  copy_item_display "$SKILL_SRC" "$CODEX_SKILL_DEST" "Skill"
fi

if [[ "$TARGET" == "claude" || "$TARGET" == "all" ]]; then
  section "Claude"
  copy_item_display "$SKILL_SRC" "$CLAUDE_SKILL_DEST" "Skill"
  copy_workflows_for_claude
  if [[ -f "$PROJECT_DIR/CLAUDE.md" && "$OVERWRITE" -ne 1 ]]; then
    copy_item_display "$ROOT_DIR/templates/CLAUDE.project.md" "$PROJECT_DIR/CLAUDE.research-skills.md" "CLAUDE.md"
  else
    copy_item_display "$ROOT_DIR/templates/CLAUDE.project.md" "$PROJECT_DIR/CLAUDE.md" "CLAUDE.md"
  fi
fi

if [[ "$TARGET" == "gemini" || "$TARGET" == "all" ]]; then
  section "Gemini"
  copy_item_display "$SKILL_SRC" "$GEMINI_SKILL_DEST" "Skill"
  write_gemini_quickstart
fi

if [[ "$TARGET" == "antigravity" || "$TARGET" == "all" ]]; then
  section "Antigravity"
  install_antigravity_workspace
  if [[ "$ANTIGRAVITY_CLI_FOUND" -eq 1 ]]; then
    copy_item_display "$SKILL_SRC" "$ANTIGRAVITY_SKILL_DEST" "Global Skill"
  else
    skip "Global Skill" "$ANTIGRAVITY_SKILL_DEST (antigravity CLI not found)"
  fi
fi

if [[ "$INSTALL_CLI" -eq 1 ]]; then
  install_cli_assets
fi

section "Project Env"
install_project_env

# ── Doctor ───────────────────────────────────────────────────────────────────
if [[ "$RUN_DOCTOR" -eq 1 ]]; then
  section "Doctor"
  if ! has_python3; then
    warn "Skipping doctor: python3 not found. Install/update still completed."
  else
    DOCTOR_OUTPUT=$(run_cmd python3 -m bridges.orchestrator doctor --cwd "$PROJECT_DIR" 2>&1) || true

    # Parse JSON output and render human-friendly summary
    python3 -c "
import sys, json

raw = sys.stdin.read()
# Strip any non-JSON preamble (e.g. language selection prompts)
json_start = raw.find('{')
if json_start < 0:
    print(raw)
    sys.exit(0)
preamble = raw[:json_start].strip()
if preamble:
    for line in preamble.splitlines():
        print(f'  {line}')
    print()

try:
    data = json.loads(raw[json_start:])
except json.JSONDecodeError:
    print(raw)
    sys.exit(0)

analysis = data.get('merged_analysis', '')
recs = data.get('recommendations', [])
confidence = data.get('confidence', 0)

GREEN = '\033[32m'
YELLOW = '\033[33m'
RED = '\033[31m'
DIM = '\033[2m'
RESET = '\033[0m'
BOLD = '\033[1m'

import os
if os.environ.get('NO_COLOR') or not sys.stdout.isatty():
    GREEN = YELLOW = RED = DIM = RESET = BOLD = ''

# Parse check details from merged_analysis
ok_count = analysis.count('[OK]')
warn_count = analysis.count('[WARNING]')
err_count = analysis.count('[ERROR]')
total = ok_count + warn_count + err_count

if total > 0:
    bar_width = 30
    ok_bar = int(ok_count / total * bar_width)
    warn_bar = int(warn_count / total * bar_width)
    err_bar = bar_width - ok_bar - warn_bar
    bar = f'{GREEN}{\"█\" * ok_bar}{YELLOW}{\"█\" * warn_bar}{RED}{\"█\" * err_bar}{RESET}'
    print(f'  {bar}  {ok_count}/{total} passed')
    print()

# Show individual check lines
for line in analysis.splitlines():
    line = line.strip()
    if line.startswith('- [OK]'):
        detail = line[len('- [OK] '):]
        print(f'  {GREEN}✓{RESET}  {detail}')
    elif line.startswith('- [WARNING]'):
        detail = line[len('- [WARNING] '):]
        print(f'  {YELLOW}⚠{RESET}  {DIM}{detail}{RESET}')
    elif line.startswith('- [ERROR]'):
        detail = line[len('- [ERROR] '):]
        print(f'  {RED}✗{RESET}  {detail}')

if recs:
    print()
    print(f'  {BOLD}Recommendations:{RESET}')
    for r in recs:
        print(f'  {DIM}•  {r}{RESET}')
" <<< "$DOCTOR_OUTPUT"
  fi
fi

# ── Footer ───────────────────────────────────────────────────────────────────
printf "\n${C_GREEN}${C_BOLD}✓ Installation complete${C_RESET}"
case "$TARGET" in
  all)  printf " ${C_DIM}(codex + claude + gemini + antigravity)${C_RESET}" ;;
  *)    printf " ${C_DIM}($TARGET)${C_RESET}" ;;
esac
printf "\n"
if [[ "$INSTALL_CLI" -eq 1 ]] && ! path_contains_dir "$CLI_DIR"; then
  printf "  ${C_YELLOW}Add %s to PATH to use research-skills / rsk / rsw.${C_RESET}\n" "$CLI_DIR"
fi
printf "  ${C_DIM}Restart Codex / Claude Code / Gemini CLI to activate.${C_RESET}\n\n"
