#!/usr/bin/env bash
set -euo pipefail

DEFAULT_REPO="jxpeng98/research-skills"
TARGET="all"
MODE="copy"
PROJECT_DIR="$(pwd)"
OVERWRITE=0
RUN_DOCTOR=0
DRY_RUN=0
PROFILE=""
REF=""
REF_TYPE="tag"
INCLUDE_BETA=0
SOURCE_REPO=""
RAW_REPO="${RESEARCH_SKILLS_REPO:-$DEFAULT_REPO}"
INSTALL_CLI=1
CLI_DIR="${RESEARCH_SKILLS_BIN_DIR:-$HOME/.local/bin}"
PARTS=""
MISE_BIN="${HOME}/.local/bin/mise"
PYTHON_RUNTIME_MODE=""

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
  ./scripts/bootstrap_research_skill.sh [options]

Options:
  --profile <partial|full>            Install preset (partial: assets only, full: assets + shell CLI + doctor)
  --repo <owner/repo|git-url>          Upstream repo (default: RESEARCH_SKILLS_REPO or jxpeng98/research-skills)
  --ref <tag-or-branch>                Explicit release tag or branch name
  --ref-type <tag|branch>              How to interpret --ref (default: tag)
  --beta                               Install the latest beta/prerelease tag when --ref is omitted
  --source-repo <path>                 Use a local checkout instead of downloading from GitHub
  --target <codex|claude|gemini|antigravity|all> Install target (default: all)
  --mode <copy>                        Install mode for remote bootstrap (default: copy)
  --project-dir <path>                 Project directory used when project surfaces are enabled (default: current dir)
  --install-cli                        Install shell CLI commands into the bin dir (default: on)
  --no-cli                             Skip shell CLI installation
  --cli-dir <path>                     Directory for shell CLI binaries (default: RESEARCH_SKILLS_BIN_DIR or ~/.local/bin)
  --parts <globals,project,cli,doctor> Limit install surfaces passed to the installer
  --overwrite                          Overwrite existing installed files
  --doctor                             Run orchestrator doctor after install when python3 exists
  --no-doctor                          Skip doctor even in full profile
  --dry-run                            Print actions without writing files
  -h, --help                           Show help

Notes:
  - This bootstrap path only requires bash + curl/wget + tar.
  - Remote bootstrap always uses copy mode. For link mode, clone the repo and run scripts/install_research_skill.sh locally.
  - `partial` skips shell CLI installation and doctor.
  - `full` enables shell CLI installation and doctor (when python3 exists).
EOF
}

describe_profiles() {
  cat <<'EOF'

Choose an install profile:

  1) partial
     - Installs global workflow assets only
     - Does not install shell CLI
     - Does not require Python
     - Does not run orchestrator doctor

  2) full
     - Installs global workflow assets
     - Installs shell CLI commands (`research-skills`, `rsk`, `rsw`)
     - Ensures Python 3.12 is available via mise if missing
     - Runs orchestrator doctor after install

EOF
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

prompt_for_profile() {
  local answer

  if ! : >/dev/tty 2>/dev/null || ! : </dev/tty 2>/dev/null; then
    err "Missing --profile and no interactive terminal is available. Pass --profile partial or --profile full."
    exit 2
  fi

  describe_profiles >/dev/tty
  while true; do
    printf "Select profile [1/2]: " >/dev/tty
    IFS= read -r answer </dev/tty
    case "$answer" in
      1|partial|PARTIAL)
        PROFILE="partial"
        apply_profile_defaults "$PROFILE"
        return 0
        ;;
      2|full|FULL)
        PROFILE="full"
        apply_profile_defaults "$PROFILE"
        return 0
        ;;
      *)
        printf "  %sPlease enter 1, 2, partial, or full.%s\n" "${C_YELLOW}" "${C_RESET}" >/dev/tty
        ;;
    esac
  done
}

have_cmd() {
  command -v "$1" >/dev/null 2>&1
}

path_has_dir() {
  local target="$1"
  local entry
  local -a path_entries=()
  IFS=':' read -r -a path_entries <<<"${PATH:-}"
  for entry in "${path_entries[@]}"; do
    [[ "$entry" == "$target" ]] && return 0
  done
  return 1
}

prepend_path_dir() {
  local target="$1"
  [[ -n "$target" ]] || return 0
  if ! path_has_dir "$target"; then
    export PATH="$target:$PATH"
  fi
}

shell_path_literal() {
  local target="$1"
  if [[ "$target" == "$HOME"* ]]; then
    printf '%s\n' "\$HOME${target#$HOME}"
    return 0
  fi
  printf '%s\n' "$target"
}

ensure_rc_path_entry() {
  local rc_file="$1"
  local target="$2"
  local marker path_literal
  [[ -n "$rc_file" && -n "$target" ]] || return 0
  marker="# research-skills bootstrap path"
  path_literal="$(shell_path_literal "$target")"

  mkdir -p "$(dirname "$rc_file")"
  [[ -f "$rc_file" ]] || : >"$rc_file"
  if grep -Fqs "$path_literal" "$rc_file"; then
    return 0
  fi
  printf '\n%s\nexport PATH="%s:$PATH"\n' "$marker" "$path_literal" >>"$rc_file"
}

persist_shell_path_entries() {
  local target shell_name primary_rc
  shell_name="${SHELL##*/}"
  primary_rc=""
  case "$shell_name" in
    zsh) primary_rc="${ZDOTDIR:-$HOME}/.zshrc" ;;
    bash) primary_rc="$HOME/.bashrc" ;;
  esac

  for target in "$@"; do
    [[ -n "$target" ]] || continue
    prepend_path_dir "$target"
    if [[ -n "$primary_rc" ]]; then
      ensure_rc_path_entry "$primary_rc" "$target"
    fi
    ensure_rc_path_entry "$HOME/.profile" "$target"
  done
}

resolve_mise_bin() {
  if have_cmd mise; then
    command -v mise
    return 0
  fi
  if [[ -x "$MISE_BIN" ]]; then
    printf '%s\n' "$MISE_BIN"
    return 0
  fi
  return 1
}

resolve_abs_path() {
  local raw="${1:-}"
  if [[ -z "$raw" ]]; then
    return 1
  fi
  case "$raw" in
    "~") raw="$HOME" ;;
    "~/"*) raw="$HOME/${raw#~/}" ;;
  esac
  if [[ "$raw" != /* ]]; then
    raw="$PWD/$raw"
  fi
  printf '%s\n' "$(cd "$raw" 2>/dev/null && pwd -P)"
}

install_mise() {
  local installer_url="https://mise.run"
  local mise_path
  local mise_bin_dir shims_dir

  if mise_path="$(resolve_mise_bin)"; then
    MISE_BIN="$mise_path"
    mise_bin_dir="$(dirname "$MISE_BIN")"
    shims_dir="${HOME}/.local/share/mise/shims"
    persist_shell_path_entries "$mise_bin_dir" "$shims_dir"
    info "mise:    $MISE_BIN"
    return 0
  fi

  if [[ "$DRY_RUN" -eq 1 ]]; then
    info "mise:    install via $installer_url"
    MISE_BIN="${HOME}/.local/bin/mise"
    return 0
  fi

  info "mise:    installing via $installer_url"
  download_text "$installer_url" | sh
  if ! mise_path="$(resolve_mise_bin)"; then
    err "Installed mise but could not locate the binary."
    exit 1
  fi
  MISE_BIN="$mise_path"
  mise_bin_dir="$(dirname "$MISE_BIN")"
  shims_dir="${HOME}/.local/share/mise/shims"
  persist_shell_path_entries "$mise_bin_dir" "$shims_dir"
  info "mise:    ready -> $MISE_BIN"
}

ensure_python_runtime() {
  local current_python
  local current_version

  if current_python="$(command -v python3 2>/dev/null)"; then
    current_version="$("$current_python" -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")' 2>/dev/null || true)"
    if [[ "$current_version" =~ ^([0-9]+)\.([0-9]+)$ ]]; then
      if (( BASH_REMATCH[1] > 3 || (BASH_REMATCH[1] == 3 && BASH_REMATCH[2] >= 12) )); then
        info "python:  $current_python ($current_version)"
        PYTHON_RUNTIME_MODE="direct"
        return 0
      fi
    fi
    warn "python3 found at $current_python but version is below 3.12; installing python@3.12 via mise."
  else
    warn "python3 not found; full profile needs Python 3.12 for orchestrator."
  fi

  install_mise
  if [[ "$DRY_RUN" -eq 1 ]]; then
    info "python:  install via mise -> python@3.12"
    PYTHON_RUNTIME_MODE="mise"
    return 0
  fi

  "$MISE_BIN" install python@3.12
  "$MISE_BIN" use -g python@3.12
  info "python:  installed via mise -> python@3.12"
  PYTHON_RUNTIME_MODE="mise"
}

normalize_repo() {
  local raw="${1:-}"
  local value owner repo

  value="$(printf '%s' "$raw" | tr -d '[:space:]')"
  if [[ -z "$value" ]]; then
    return 1
  fi

  if [[ "$value" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]]; then
    printf '%s\n' "$value"
    return 0
  fi

  value="$(printf '%s\n' "$value" | sed -E \
    -e 's#^git@[^:]+:##' \
    -e 's#^ssh://git@[^/]+/##' \
    -e 's#^https?://[^/]+/##' \
    -e 's#\.git$##')"
  value="${value#/}"
  owner="${value%%/*}"
  repo="${value#*/}"
  repo="${repo%%/*}"

  if [[ "$owner/$repo" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]]; then
    printf '%s\n' "$owner/$repo"
    return 0
  fi

  return 1
}

github_token() {
  printf '%s' "${GITHUB_TOKEN:-${GH_TOKEN:-}}"
}

download_text() {
  local url="$1"
  local token
  token="$(github_token)"

  if have_cmd curl; then
    local -a args
    args=(-fsSL)
    if [[ -n "$token" ]]; then
      args+=(-H "Authorization: Bearer $token")
    fi
    curl "${args[@]}" "$url"
    return 0
  fi

  if have_cmd wget; then
    local -a args
    args=(-qO-)
    if [[ -n "$token" ]]; then
      args+=("--header=Authorization: Bearer $token")
    fi
    wget "${args[@]}" "$url"
    return 0
  fi

  err "Missing downloader: install curl or wget."
  exit 1
}

download_file() {
  local url="$1"
  local dest="$2"
  local token
  token="$(github_token)"

  if have_cmd curl; then
    local -a args
    args=(-fsSL -o "$dest")
    if [[ -n "$token" ]]; then
      args+=(-H "Authorization: Bearer $token")
    fi
    curl "${args[@]}" "$url"
    return 0
  fi

  if have_cmd wget; then
    local -a args
    args=(-q -O "$dest")
    if [[ -n "$token" ]]; then
      args+=("--header=Authorization: Bearer $token")
    fi
    wget "${args[@]}" "$url"
    return 0
  fi

  err "Missing downloader: install curl or wget."
  exit 1
}

resolve_latest_tag() {
  local repo="$1"
  local require_beta="${2:-0}"
  local payload tag url
  local -a release_tags=()
  local -a git_tags=()

  tag_sort_key() {
    local raw_tag="$1"
    local require_beta_local="${2:-0}"
    local beta_num beta_key

    if [[ ! "$raw_tag" =~ ^v?([0-9]+)\.([0-9]+)\.([0-9]+)(-beta\.([0-9]+)|b([0-9]+))?$ ]]; then
      return 1
    fi

    beta_num="${BASH_REMATCH[5]:-${BASH_REMATCH[6]:-}}"
    if [[ "$require_beta_local" -eq 1 && -z "$beta_num" ]]; then
      return 1
    fi

    if [[ -n "$beta_num" ]]; then
      beta_key="$beta_num"
    else
      beta_key=1000000000
    fi

    printf '%010d.%010d.%010d.%010d\n' "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}" "${BASH_REMATCH[3]}" "$beta_key"
  }

  choose_best_tag() {
    local require_beta_local="${1:-0}"
    local best_tag="" best_key="" candidate candidate_key
    shift

    for candidate in "$@"; do
      candidate_key="$(tag_sort_key "$candidate" "$require_beta_local")" || continue
      if [[ -z "$best_key" || "$candidate_key" > "$best_key" ]]; then
        best_key="$candidate_key"
        best_tag="$candidate"
      fi
    done

    [[ -n "$best_tag" ]] || return 1
    printf '%s\n' "$best_tag"
  }

  if [[ "$require_beta" -eq 0 ]] && payload="$(download_text "https://api.github.com/repos/$repo/releases/latest" 2>/dev/null)"; then
    tag="$(printf '%s\n' "$payload" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)"
    if [[ -n "$tag" ]]; then
      printf '%s\n' "$tag"
      return 0
    fi
  fi

  if payload="$(download_text "https://api.github.com/repos/$repo/releases?per_page=20" 2>/dev/null)"; then
    mapfile -t release_tags < <(
      printf '%s' "$payload" |
        grep -o '"tag_name"[[:space:]]*:[[:space:]]*"[^"]*"' |
        sed -E 's/.*:[[:space:]]*"([^"]*)"/\1/'
    )
    if tag="$(choose_best_tag "$require_beta" "${release_tags[@]}")"; then
      printf '%s\n' "$tag"
      return 0
    fi
  fi

  if payload="$(download_text "https://api.github.com/repos/$repo/tags?per_page=50" 2>/dev/null)"; then
    mapfile -t git_tags < <(
      printf '%s' "$payload" |
        grep -o '"name"[[:space:]]*:[[:space:]]*"[^"]*"' |
        sed -E 's/.*:[[:space:]]*"([^"]*)"/\1/'
    )
    if tag="$(choose_best_tag "$require_beta" "${git_tags[@]}")"; then
      printf '%s\n' "$tag"
      return 0
    fi
  fi

  if [[ "$require_beta" -eq 0 ]] && have_cmd curl; then
    local -a args
    args=(-fsSL -o /dev/null -w '%{url_effective}')
    local token
    token="$(github_token)"
    if [[ -n "$token" ]]; then
      args+=(-H "Authorization: Bearer $token")
    fi
    url="$(curl "${args[@]}" "https://github.com/$repo/releases/latest" 2>/dev/null || true)"
    tag="${url##*/}"
    if [[ -n "$tag" && "$tag" != "latest" && "$tag" != "releases" ]]; then
      printf '%s\n' "$tag"
      return 0
    fi
  fi

  return 1
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      [[ $# -ge 2 ]] || { err "Missing value for --profile"; exit 2; }
      PROFILE="$2"
      apply_profile_defaults "$PROFILE"
      shift 2
      ;;
    --repo)
      [[ $# -ge 2 ]] || { err "Missing value for --repo"; exit 2; }
      RAW_REPO="$2"
      shift 2
      ;;
    --ref)
      [[ $# -ge 2 ]] || { err "Missing value for --ref"; exit 2; }
      REF="$2"
      shift 2
      ;;
    --ref-type)
      [[ $# -ge 2 ]] || { err "Missing value for --ref-type"; exit 2; }
      REF_TYPE="$2"
      shift 2
      ;;
    --beta)
      INCLUDE_BETA=1
      shift
      ;;
    --source-repo)
      [[ $# -ge 2 ]] || { err "Missing value for --source-repo"; exit 2; }
      SOURCE_REPO="$2"
      shift 2
      ;;
    --target)
      [[ $# -ge 2 ]] || { err "Missing value for --target"; exit 2; }
      TARGET="$2"
      shift 2
      ;;
    --mode)
      [[ $# -ge 2 ]] || { err "Missing value for --mode"; exit 2; }
      MODE="$2"
      shift 2
      ;;
    --project-dir)
      [[ $# -ge 2 ]] || { err "Missing value for --project-dir"; exit 2; }
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
      [[ $# -ge 2 ]] || { err "Missing value for --cli-dir"; exit 2; }
      CLI_DIR="$2"
      shift 2
      ;;
    --parts)
      [[ $# -ge 2 ]] || { err "Missing value for --parts"; exit 2; }
      PARTS="$2"
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
    --no-doctor)
      RUN_DOCTOR=0
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
      err "Unknown argument: $1"
      usage
      exit 2
      ;;
  esac
done

if [[ -z "$PROFILE" ]]; then
  prompt_for_profile
fi

case "$TARGET" in
  codex|claude|gemini|antigravity|all) ;;
  *)
    err "Unsupported target: $TARGET"
    exit 2
    ;;
esac

case "$REF_TYPE" in
  tag|branch) ;;
  *)
    err "Unsupported ref type: $REF_TYPE"
    exit 2
    ;;
esac

case "$MODE" in
  copy) ;;
  link)
    err "Remote bootstrap does not support --mode link. Clone the repo and run scripts/install_research_skill.sh locally."
    exit 2
    ;;
  *)
    err "Unsupported mode: $MODE"
    exit 2
    ;;
esac

if [[ -n "$SOURCE_REPO" ]]; then
  if ! SOURCE_REPO="$(resolve_abs_path "$SOURCE_REPO")"; then
    err "Unable to resolve --source-repo: $SOURCE_REPO"
    exit 2
  fi
  if [[ ! -f "$SOURCE_REPO/scripts/install_research_skill.sh" ]]; then
    err "Local source repo is missing scripts/install_research_skill.sh: $SOURCE_REPO"
    exit 2
  fi
  REPO="<local>"
  REF="<checkout>"
  REF_TYPE="local"
else
  if ! REPO="$(normalize_repo "$RAW_REPO")"; then
    err "Unsupported repo spec: $RAW_REPO"
    exit 2
  fi

  if [[ -z "$REF" ]]; then
    if [[ "$DRY_RUN" -eq 1 ]]; then
      if [[ "$INCLUDE_BETA" -eq 1 ]]; then
        REF="<latest-beta>"
      else
        REF="<latest>"
      fi
    else
      if ! REF="$(resolve_latest_tag "$REPO" "$INCLUDE_BETA")"; then
        if [[ "$INCLUDE_BETA" -eq 1 ]]; then
          err "Unable to resolve the latest beta/prerelease tag for $REPO."
          err "Pass --ref <beta-tag> --ref-type tag, or use --ref main --ref-type branch."
        else
          err "Unable to resolve the latest GitHub release for $REPO."
          err "Pass --ref <tag> --ref-type tag, or use --ref main --ref-type branch."
        fi
        exit 1
      fi
    fi
  fi
fi

if [[ -z "$SOURCE_REPO" ]] && ! have_cmd tar; then
  err "Missing required command: tar"
  exit 1
fi

TMP_DIR=""
cleanup() {
  if [[ -n "$TMP_DIR" ]]; then
    rm -rf "$TMP_DIR"
  fi
  return 0
}
trap cleanup EXIT

if [[ -n "$SOURCE_REPO" ]]; then
  TARBALL_URL="<local-checkout>"
else
  TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/research-skills-bootstrap.XXXXXX")"
  if [[ "$REF" == "<latest>" ]]; then
    TARBALL_URL="https://github.com/$REPO/releases/latest"
  elif [[ "$REF_TYPE" == "tag" ]]; then
    TARBALL_URL="https://github.com/$REPO/archive/refs/tags/$REF.tar.gz"
  else
    TARBALL_URL="https://github.com/$REPO/archive/refs/heads/$REF.tar.gz"
  fi
fi

ARCHIVE_PATH="${TMP_DIR:+$TMP_DIR/research-skills.tar.gz}"

printf "\n${C_BOLD}${C_CYAN}Research Skills Bootstrap${C_RESET}\n"
info "repo:    $REPO"
info "ref:     $REF ($REF_TYPE)"
info "project: $PROJECT_DIR"
info "target:  $TARGET  |  mode: $MODE"
info "profile: $PROFILE"
if [[ "$INSTALL_CLI" -eq 1 ]]; then
  info "cli:     install -> $CLI_DIR"
else
  info "cli:     skip"
fi
if [[ -n "$SOURCE_REPO" ]]; then
  info "source:  $SOURCE_REPO"
fi

if [[ "$PROFILE" == "full" ]]; then
  ensure_python_runtime
fi

if [[ "$DRY_RUN" -eq 1 ]]; then
  if [[ -n "$SOURCE_REPO" ]]; then
    info "source:   local checkout"
    printf '[dry-run] bash %q --target %q --mode %q --project-dir %q' "$SOURCE_REPO/scripts/install_research_skill.sh" "$TARGET" "$MODE" "$PROJECT_DIR"
  else
    info "download: $TARBALL_URL"
    printf '[dry-run] bash <downloaded>/scripts/install_research_skill.sh --target %q --mode %q --project-dir %q' "$TARGET" "$MODE" "$PROJECT_DIR"
  fi
  if [[ "$OVERWRITE" -eq 1 ]]; then
    printf ' --overwrite'
  fi
  if [[ "$INSTALL_CLI" -eq 1 ]]; then
    printf ' --install-cli --cli-dir %q' "$CLI_DIR"
  fi
  if [[ -n "$PARTS" ]]; then
    printf ' --parts %q' "$PARTS"
  fi
  if [[ "$RUN_DOCTOR" -eq 1 ]]; then
    printf ' --doctor'
  fi
  printf ' --dry-run\n'
  exit 0
else
  if [[ -n "$SOURCE_REPO" ]]; then
    info "source:   local checkout"
    EXTRACTED_ROOT="$SOURCE_REPO"
  else
    info "download: $TARBALL_URL"
    download_file "$TARBALL_URL" "$ARCHIVE_PATH"
    tar -xzf "$ARCHIVE_PATH" -C "$TMP_DIR"

    EXTRACTED_ROOT=""
    for candidate in "$TMP_DIR"/*; do
      if [[ -d "$candidate" && -f "$candidate/scripts/install_research_skill.sh" ]]; then
        EXTRACTED_ROOT="$candidate"
        break
      fi
    done

    if [[ -z "$EXTRACTED_ROOT" ]]; then
      err "Failed to locate scripts/install_research_skill.sh in downloaded archive."
      exit 1
    fi
  fi
fi

cmd=(
  bash
  "$EXTRACTED_ROOT/scripts/install_research_skill.sh"
  --target "$TARGET"
  --mode "$MODE"
  --project-dir "$PROJECT_DIR"
)

if [[ "$OVERWRITE" -eq 1 ]]; then
  cmd+=(--overwrite)
fi
if [[ "$INSTALL_CLI" -eq 1 ]]; then
  cmd+=(--install-cli --cli-dir "$CLI_DIR")
fi
if [[ -n "$PARTS" ]]; then
  cmd+=(--parts "$PARTS")
fi
if [[ "$RUN_DOCTOR" -eq 1 ]]; then
  cmd+=(--doctor)
fi
if [[ "$DRY_RUN" -eq 1 ]]; then
  cmd+=(--dry-run)
fi

if [[ "$PROFILE" == "full" && "$DRY_RUN" -ne 1 && "$PYTHON_RUNTIME_MODE" == "mise" ]]; then
  "$MISE_BIN" exec python@3.12 -- "${cmd[@]}"
else
  "${cmd[@]}"
fi
