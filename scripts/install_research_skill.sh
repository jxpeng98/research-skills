#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST_PATH="$ROOT_DIR/install/install_manifest.tsv"
TARGET="all"
MODE="copy"
PROJECT_DIR="$(pwd)"
OVERWRITE=0
DRY_RUN=0
RUN_DOCTOR=0
INSTALL_CLI=0
PROFILE="custom"
CLI_DIR="${RESEARCH_SKILLS_BIN_DIR:-$HOME/.local/bin}"
PARTS_SPEC=""
PARTS_ACTIVE=0
INSTALL_GLOBALS=1
INSTALL_PROJECT=0

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
  --profile <partial|full>             Install preset (partial: assets only, full: assets + shell CLI + doctor)
  --target <codex|claude|gemini|antigravity|all> Install target (default: all)
  --mode <copy|link>                   Install mode (default: copy)
  --project-dir <path>                 Project directory used when project surfaces are enabled (default: current dir)
  --install-cli                        Install shell CLI commands (`research-skills`, `rsk`, `rsw`)
  --no-cli                             Skip shell CLI installation
  --cli-dir <path>                     Directory for shell CLI binaries (default: RESEARCH_SKILLS_BIN_DIR or ~/.local/bin)
  --parts <globals,project,cli,doctor>
                                       Install only specific surfaces. If omitted, globals stay on by default,
                                       while project/cli/doctor remain opt-in via --parts or explicit switches.
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

Profile behavior:
  partial           Install global workflow assets only
  full              Install global workflow assets, shell CLI, and run doctor
EOF
}

log() {
  echo "[install] $*"
}

apply_profile_defaults() {
  case "${1:-}" in
    partial)
      INSTALL_CLI=0
      RUN_DOCTOR=0
      ;;
    full)
      INSTALL_CLI=1
      RUN_DOCTOR=1
      ;;
    *)
      err "Unsupported profile: $1"
      exit 2
      ;;
  esac
}

parse_parts_spec() {
  local raw="${1:-}"
  local old_ifs="$IFS"
  local token normalized

  INSTALL_GLOBALS=0
  INSTALL_PROJECT=0
  INSTALL_CLI=0
  RUN_DOCTOR=0

  IFS=','
  for token in $raw; do
    token="$(printf '%s' "$token" | tr '[:upper:]' '[:lower:]')"
    token="${token#"${token%%[![:space:]]*}"}"
    token="${token%"${token##*[![:space:]]}"}"
    case "$token" in
      "" )
        ;;
      all|"*")
        INSTALL_GLOBALS=1
        INSTALL_PROJECT=1
        INSTALL_CLI=1
        RUN_DOCTOR=1
        ;;
      global|globals)
        INSTALL_GLOBALS=1
        ;;
      project)
        INSTALL_PROJECT=1
        ;;
      cli|shell-cli)
        INSTALL_CLI=1
        ;;
      doctor)
        RUN_DOCTOR=1
        ;;
      *)
        err "Unsupported install part: $token"
        exit 2
        ;;
    esac
  done
  IFS="$old_ifs"
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

doctor_cmd() {
  local pythonpath="$ROOT_DIR"
  if [[ -n "${PYTHONPATH:-}" ]]; then
    pythonpath="$ROOT_DIR:$PYTHONPATH"
  fi
  PYTHONPATH="$pythonpath" python3 -m bridges.orchestrator "$@"
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

cli_install_hint() {
  case "$1" in
    codex)
      printf 'Install the Codex CLI from the official OpenAI distribution, then ensure `codex` is on PATH.\n'
      ;;
    claude)
      printf 'Install Claude Code: npm install -g @anthropic-ai/claude-code\n'
      ;;
    gemini)
      printf 'Install Gemini CLI: npm install -g @google/gemini-cli\n'
      ;;
    antigravity)
      printf 'Install Antigravity and ensure the `antigravity` binary is on PATH before relying on the global skill directory.\n'
      ;;
    *)
      return 1
      ;;
  esac
}

check_target_cli() {
  local target="$1"
  local cli_name resolved install_hint

  cli_name="$(cli_name_for_target "$target")" || return 1
  if resolved="$(command -v "$cli_name" 2>/dev/null)"; then
    ok "CLI" "$target -> $resolved"
    return 0
  fi

  warn "$target CLI not found in PATH: $cli_name"
  install_hint="$(cli_install_hint "$target" || true)"
  if [[ -n "$install_hint" ]]; then
    info "hint: $install_hint"
  fi
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

expand_manifest_path() {
  local raw="$1"
  local value="$raw"
  value="${value//\$\{PROJECT_DIR\}/$PROJECT_DIR}"
  value="${value//\$\{CODEX_HOME\}/${CODEX_HOME:-$HOME/.codex}}"
  value="${value//\$\{CLAUDE_CODE_HOME\}/${CLAUDE_CODE_HOME:-$HOME/.claude}}"
  value="${value//\$\{GEMINI_HOME\}/${GEMINI_HOME:-$HOME/.gemini}}"
  value="${value//\$\{ANTIGRAVITY_HOME\}/${ANTIGRAVITY_HOME:-$HOME/.gemini/antigravity}}"
  printf '%s\n' "$value"
}

# Legacy copy_workflows_from_manifest removed — workflows now bundled in skill dir-copy

apply_manifest_entry() {
  local op="$1"
  local label="$2"
  local source_rel="$3"
  local dest_tpl="$4"
  local src="$ROOT_DIR/$source_rel"
  local dest
  dest="$(expand_manifest_path "$dest_tpl")"

  case "$op" in
    dir-copy|file-copy)
      copy_item_display "$src" "$dest" "$label"
      ;;
    *)
      err "Unsupported manifest operation: $op"
      exit 1
      ;;
  esac
}

# Legacy project-local helpers removed — skills + workflows now global only

install_manifest_target() {
  local wanted_target="$1"
  local manifest_target op label source_rel dest_tpl part
  while IFS=$'\t' read -r manifest_target op label source_rel dest_tpl; do
    [[ -n "$manifest_target" ]] || continue
    [[ "$manifest_target" == \#* ]] && continue
    [[ "$manifest_target" == "$wanted_target" ]] || continue
    part="$(manifest_entry_part "$dest_tpl")"
    manifest_part_enabled "$part" || continue
    apply_manifest_entry "$op" "$label" "$source_rel" "$dest_tpl"
  done < "$MANIFEST_PATH"
}

# Legacy install_project_manifest removed — only .env remains, opt-in via parts

manifest_entry_part() {
  local dest_tpl="$1"
  if [[ "$dest_tpl" == *'${PROJECT_DIR}'* ]]; then
    printf 'project\n'
  else
    printf 'globals\n'
  fi
}

manifest_part_enabled() {
  local part="$1"
  case "$part" in
    globals) [[ "$INSTALL_GLOBALS" -eq 1 ]] ;;
    project) [[ "$INSTALL_PROJECT" -eq 1 ]] ;;
    *) return 1 ;;
  esac
}

target_has_enabled_entries() {
  local wanted_target="$1"
  local manifest_target op label source_rel dest_tpl part
  while IFS=$'\t' read -r manifest_target op label source_rel dest_tpl; do
    [[ -n "$manifest_target" ]] || continue
    [[ "$manifest_target" == \#* ]] && continue
    [[ "$manifest_target" == "$wanted_target" ]] || continue
    part="$(manifest_entry_part "$dest_tpl")"
    if manifest_part_enabled "$part"; then
      return 0
    fi
  done < "$MANIFEST_PATH"
  return 1
}

# ── Parse arguments ──────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      [[ $# -ge 2 ]] || { echo "Missing value for --profile" >&2; exit 2; }
      PROFILE="$2"
      apply_profile_defaults "$PROFILE"
      shift 2
      ;;
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
    --parts)
      [[ $# -ge 2 ]] || { echo "Missing value for --parts" >&2; exit 2; }
      PARTS_SPEC="$2"
      PARTS_ACTIVE=1
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
if [[ "$PARTS_ACTIVE" -eq 1 ]]; then
  parse_parts_spec "$PARTS_SPEC"
fi
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
info "profile: $PROFILE"
if [[ "$PARTS_ACTIVE" -eq 1 ]]; then
  info "parts:   $PARTS_SPEC"
fi
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

# ── Sync skill package ───────────────────────────────────────────────────────
if [[ "$INSTALL_GLOBALS" -eq 1 ]]; then
  section "Sync Skill Package"
  sync_script="$ROOT_DIR/scripts/sync_skill_package.sh"
  if [[ -x "$sync_script" ]]; then
    if [[ "$DRY_RUN" -eq 1 ]]; then
      ok "Sync" "skill package (dry-run)"
    else
      bash "$sync_script"
    fi
  else
    skip "Sync" "sync_skill_package.sh not found"
  fi
fi

# ── Install targets ──────────────────────────────────────────────────────────
if [[ "$TARGET" == "codex" || "$TARGET" == "all" ]]; then
  if target_has_enabled_entries codex; then
    section "Codex"
    install_manifest_target codex
  fi
fi

if [[ "$TARGET" == "claude" || "$TARGET" == "all" ]]; then
  if target_has_enabled_entries claude; then
    section "Claude"
    install_manifest_target claude
  fi
fi

if [[ "$TARGET" == "gemini" || "$TARGET" == "all" ]]; then
  if target_has_enabled_entries gemini; then
    section "Gemini"
    install_manifest_target gemini
  fi
fi

if [[ "$TARGET" == "antigravity" || "$TARGET" == "all" ]]; then
  if target_has_enabled_entries antigravity; then
    section "Antigravity"
    install_manifest_target antigravity
  fi
fi

if [[ "$INSTALL_CLI" -eq 1 ]]; then
  install_cli_assets
fi

if [[ "$INSTALL_PROJECT" -eq 1 ]]; then
  section "Project Env"
  manifest_target= op= label= source_rel= dest_tpl=
  while IFS=$'\t' read -r manifest_target op label source_rel dest_tpl; do
    [[ -n "$manifest_target" ]] || continue
    [[ "$manifest_target" == \#* ]] && continue
    [[ "$manifest_target" == "project" ]] || continue
    apply_manifest_entry "$op" "$label" "$source_rel" "$dest_tpl"
  done < "$MANIFEST_PATH"
fi

# ── Doctor ───────────────────────────────────────────────────────────────────
if [[ "$RUN_DOCTOR" -eq 1 ]]; then
  section "Doctor"
  if ! has_python3; then
    warn "Skipping doctor: python3 not found. Install/update still completed."
  else
    DOCTOR_OUTPUT=$(run_cmd doctor_cmd doctor --cwd "$PROJECT_DIR" 2>&1) || true

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
