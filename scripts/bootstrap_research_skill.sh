#!/usr/bin/env bash
set -euo pipefail

DEFAULT_REPO="jxpeng98/research-skills"
TARGET="all"
MODE="copy"
PROJECT_DIR="$(pwd)"
OVERWRITE=0
RUN_DOCTOR=0
DRY_RUN=0
REF=""
REF_TYPE="tag"
RAW_REPO="${RESEARCH_SKILLS_REPO:-$DEFAULT_REPO}"
INSTALL_CLI=1
CLI_DIR="${RESEARCH_SKILLS_BIN_DIR:-$HOME/.local/bin}"

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
  --repo <owner/repo|git-url>          Upstream repo (default: RESEARCH_SKILLS_REPO or jxpeng98/research-skills)
  --ref <tag-or-branch>                Explicit release tag or branch name
  --ref-type <tag|branch>              How to interpret --ref (default: tag)
  --target <codex|claude|gemini|antigravity|all> Install target (default: all)
  --mode <copy>                        Install mode for remote bootstrap (default: copy)
  --project-dir <path>                 Project directory for command/workflow integration (default: current dir)
  --install-cli                        Install shell CLI commands into the bin dir (default: on)
  --no-cli                             Skip shell CLI installation
  --cli-dir <path>                     Directory for shell CLI binaries (default: RESEARCH_SKILLS_BIN_DIR or ~/.local/bin)
  --overwrite                          Overwrite existing installed files
  --doctor                             Run orchestrator doctor after install when python3 exists
  --dry-run                            Print actions without writing files
  -h, --help                           Show help

Notes:
  - This bootstrap path only requires bash + curl/wget + tar.
  - Remote bootstrap always uses copy mode. For link mode, clone the repo and run scripts/install_research_skill.sh locally.
EOF
}

have_cmd() {
  command -v "$1" >/dev/null 2>&1
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
  local payload tag url

  if payload="$(download_text "https://api.github.com/repos/$repo/releases/latest" 2>/dev/null)"; then
    tag="$(printf '%s\n' "$payload" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)"
    if [[ -n "$tag" ]]; then
      printf '%s\n' "$tag"
      return 0
    fi
  fi

  if have_cmd curl; then
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
      err "Unknown argument: $1"
      usage
      exit 2
      ;;
  esac
done

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

if ! REPO="$(normalize_repo "$RAW_REPO")"; then
  err "Unsupported repo spec: $RAW_REPO"
  exit 2
fi

if [[ -z "$REF" ]]; then
  if [[ "$DRY_RUN" -eq 1 ]]; then
    REF="<latest>"
  else
    if ! REF="$(resolve_latest_tag "$REPO")"; then
      err "Unable to resolve the latest GitHub release for $REPO."
      err "Pass --ref <tag> --ref-type tag, or use --ref main --ref-type branch."
      exit 1
    fi
  fi
fi

if ! have_cmd tar; then
  err "Missing required command: tar"
  exit 1
fi

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/research-skills-bootstrap.XXXXXX")"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

if [[ "$REF" == "<latest>" ]]; then
  TARBALL_URL="https://github.com/$REPO/releases/latest"
elif [[ "$REF_TYPE" == "tag" ]]; then
  TARBALL_URL="https://github.com/$REPO/archive/refs/tags/$REF.tar.gz"
else
  TARBALL_URL="https://github.com/$REPO/archive/refs/heads/$REF.tar.gz"
fi

ARCHIVE_PATH="$TMP_DIR/research-skills.tar.gz"

printf "\n${C_BOLD}${C_CYAN}Research Skills Bootstrap${C_RESET}\n"
info "repo:    $REPO"
info "ref:     $REF ($REF_TYPE)"
info "project: $PROJECT_DIR"
info "target:  $TARGET  |  mode: $MODE"
if [[ "$INSTALL_CLI" -eq 1 ]]; then
  info "cli:     install -> $CLI_DIR"
else
  info "cli:     skip"
fi

if [[ "$DRY_RUN" -eq 1 ]]; then
  info "download: $TARBALL_URL"
  printf '[dry-run] bash <downloaded>/scripts/install_research_skill.sh --target %q --mode %q --project-dir %q' "$TARGET" "$MODE" "$PROJECT_DIR"
  if [[ "$OVERWRITE" -eq 1 ]]; then
    printf ' --overwrite'
  fi
  if [[ "$INSTALL_CLI" -eq 1 ]]; then
    printf ' --install-cli --cli-dir %q' "$CLI_DIR"
  fi
  if [[ "$RUN_DOCTOR" -eq 1 ]]; then
    printf ' --doctor'
  fi
  printf ' --dry-run\n'
  exit 0
else
  info "download: $TARBALL_URL"
fi

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
if [[ "$RUN_DOCTOR" -eq 1 ]]; then
  cmd+=(--doctor)
fi
if [[ "$DRY_RUN" -eq 1 ]]; then
  cmd+=(--dry-run)
fi

"${cmd[@]}"
