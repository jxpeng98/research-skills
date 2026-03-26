#!/usr/bin/env bash
set -euo pipefail

DEFAULT_REPO="jxpeng98/research-skills"
CLI_FLAVOR="shell-bootstrap"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BOOTSTRAP_SCRIPT=""

if [[ -n "${RESEARCH_SKILLS_BOOTSTRAP:-}" && -f "${RESEARCH_SKILLS_BOOTSTRAP:-}" ]]; then
  BOOTSTRAP_SCRIPT="${RESEARCH_SKILLS_BOOTSTRAP}"
elif [[ -f "$SCRIPT_DIR/research-skills-bootstrap" ]]; then
  BOOTSTRAP_SCRIPT="$SCRIPT_DIR/research-skills-bootstrap"
elif [[ -f "$SCRIPT_DIR/bootstrap_research_skill.sh" ]]; then
  BOOTSTRAP_SCRIPT="$SCRIPT_DIR/bootstrap_research_skill.sh"
fi

usage() {
  cat <<'EOF'
Usage:
  research-skills <command> [options]

Commands:
  check     Check installed skill versions and latest upstream release
  upgrade   Download upstream archive and refresh installed assets/CLI
  align     Print a short usage alignment
  help      Show this help

Examples:
  rsk check --repo owner/repo
  rsk upgrade --project-dir . --target all
  rsk align
EOF
}

err() {
  printf '[error] %s\n' "$*" >&2
}

trim() {
  local value="${1:-}"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
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

have_cmd() {
  command -v "$1" >/dev/null 2>&1
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

  return 1
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

find_repo_root() {
  local current="$PWD"
  while [[ "$current" != "/" ]]; do
    if [[ -f "$current/standards/research-workflow-contract.yaml" ]]; then
      printf '%s\n' "$current"
      return 0
    fi
    current="$(dirname "$current")"
  done
  return 1
}

read_version_file() {
  local path="$1"
  [[ -f "$path" ]] || return 1
  IFS= read -r raw < "$path" || true
  raw="$(trim "$raw")"
  [[ -n "$raw" ]] || return 1
  printf '%s\n' "$raw"
}

version_key() {
  local raw="${1:-}"
  local major minor patch beta

  if [[ "$raw" =~ ^v?([0-9]+)\.([0-9]+)\.([0-9]+)(-beta\.([0-9]+)|b([0-9]+))?$ ]]; then
    major="${BASH_REMATCH[1]}"
    minor="${BASH_REMATCH[2]}"
    patch="${BASH_REMATCH[3]}"
    beta="${BASH_REMATCH[5]:-${BASH_REMATCH[6]:-999999}}"
    printf '%06d%06d%06d%06d\n' "$major" "$minor" "$patch" "$beta"
    return 0
  fi
  return 1
}

extract_repo_from_toml() {
  local path="$1"
  local section=""
  local line key value repo_value="" url_value=""

  [[ -f "$path" ]] || return 1

  while IFS= read -r line || [[ -n "$line" ]]; do
    line="$(trim "$line")"
    [[ -n "$line" ]] || continue
    [[ "$line" == \#* ]] && continue

    if [[ "$line" =~ ^\[(.*)\]$ ]]; then
      section="$(trim "${BASH_REMATCH[1]}")"
      continue
    fi

    [[ "$section" == "upstream" ]] || continue
    [[ "$line" == *=* ]] || continue

    key="$(trim "${line%%=*}")"
    value="$(trim "${line#*=}")"
    value="${value%%#*}"
    value="$(trim "$value")"
    value="${value%\"}"
    value="${value#\"}"
    value="${value%\'}"
    value="${value#\'}"

    case "$key" in
      repo|repo_slug|upstream_repo)
        repo_value="$value"
        ;;
      url|repo_url|remote_url|upstream_url)
        url_value="$value"
        ;;
    esac
  done < "$path"

  if [[ -n "$repo_value" ]] && normalize_repo "$repo_value" >/dev/null 2>&1; then
    normalize_repo "$repo_value"
    return 0
  fi
  if [[ -n "$url_value" ]] && normalize_repo "$url_value" >/dev/null 2>&1; then
    normalize_repo "$url_value"
    return 0
  fi
  return 1
}

infer_repo_from_config() {
  local current="$PWD"
  local candidate
  while [[ "$current" != "/" ]]; do
    for candidate in "$current/research-skills.toml" "$current/.research-skills.toml"; do
      if extract_repo_from_toml "$candidate" >/dev/null 2>&1; then
        extract_repo_from_toml "$candidate"
        return 0
      fi
    done
    current="$(dirname "$current")"
  done
  return 1
}

infer_repo_from_git() {
  local url remote
  if ! have_cmd git; then
    return 1
  fi
  for remote in upstream origin; do
    url="$(git remote get-url "$remote" 2>/dev/null || true)"
    if [[ -n "$url" ]] && normalize_repo "$url" >/dev/null 2>&1; then
      normalize_repo "$url"
      return 0
    fi
  done
  return 1
}

resolve_repo() {
  local arg_repo="${1:-}"

  if [[ -n "$arg_repo" ]]; then
    normalize_repo "$arg_repo"
    return 0
  fi

  if [[ -n "${RESEARCH_SKILLS_REPO:-}" ]] && normalize_repo "${RESEARCH_SKILLS_REPO}" >/dev/null 2>&1; then
    normalize_repo "${RESEARCH_SKILLS_REPO}"
    return 0
  fi

  if infer_repo_from_config >/dev/null 2>&1; then
    infer_repo_from_config
    return 0
  fi

  if infer_repo_from_git >/dev/null 2>&1; then
    infer_repo_from_git
    return 0
  fi

  printf '%s\n' "$DEFAULT_REPO"
}

json_escape() {
  local raw="${1:-}"
  raw="${raw//\\/\\\\}"
  raw="${raw//\"/\\\"}"
  raw="${raw//$'\n'/\\n}"
  raw="${raw//$'\r'/\\r}"
  raw="${raw//$'\t'/\\t}"
  printf '%s' "$raw"
}

cmd_align() {
  local repo_hint="${1:-<owner>/<repo>}"
  local prog
  prog="$(basename "$0")"
  [[ -n "$prog" ]] || prog="research-skills"

  printf '%s — Quick Reference\n\n' "$prog"
  printf 'What this shell CLI installs:\n'
  printf -- '- A standalone shell CLI (per-user).\n'
  printf -- '- CLI aliases: `research-skills`, `rsk`, `rsw`.\n'
  printf -- '- A bundled bootstrap helper used by `upgrade`.\n\n'
  printf 'What `%s upgrade` modifies:\n' "$prog"
  printf -- '- Global skills: ~/.codex|~/.claude|~/.gemini and ~/.gemini/antigravity under `skills/research-paper-workflow/`\n'
  printf -- '- One project: `<project>/.agent/workflows/`, `<project>/.agents/skills/`, `CLAUDE.md`, `.gemini/`\n'
  printf -- '- Shell CLI files in `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}` when installed via bootstrap\n\n'
  printf 'Typical usage:\n'
  printf '1) Check:   %s check --repo %s\n' "$prog" "$repo_hint"
  printf '2) Upgrade: %s upgrade --repo %s --project-dir . --target all\n' "$prog" "$repo_hint"
  printf '\nTip:\n'
  printf -- '- Set `RESEARCH_SKILLS_REPO=owner/repo` to avoid passing `--repo` every time.\n'
  printf -- '- Or add `research-skills.toml` to your project root to persist the upstream repo.\n'
}

cmd_upgrade() {
  if [[ -z "$BOOTSTRAP_SCRIPT" || ! -f "$BOOTSTRAP_SCRIPT" ]]; then
    err "bootstrap helper not found next to this CLI. Reinstall the shell CLI."
    return 1
  fi
  "$BOOTSTRAP_SCRIPT" "$@"
}

cmd_check() {
  local repo_arg="" json=0 strict_network=0
  local repo repo_root="" local_version="" latest_tag="" latest_status="" update_available=0
  local codex_dir claude_dir gemini_dir antigravity_dir
  local codex_version="" claude_version="" gemini_version="" antigravity_version=""
  local best_version="" best_key="" candidate key

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --repo)
        [[ $# -ge 2 ]] || { err "missing value for --repo"; return 2; }
        repo_arg="$2"
        shift 2
        ;;
      --json)
        json=1
        shift
        ;;
      --strict-network)
        strict_network=1
        shift
        ;;
      -h|--help)
        cat <<'EOF'
Usage:
  rsk check [--repo <owner/repo|git-url>] [--json] [--strict-network]
EOF
        return 0
        ;;
      *)
        err "unknown argument for check: $1"
        return 2
        ;;
    esac
  done

  if ! repo="$(resolve_repo "$repo_arg" 2>/dev/null)"; then
    err "unsupported repo spec: $repo_arg"
    return 2
  fi

  repo_root="$(find_repo_root || true)"
  if [[ -n "$repo_root" ]]; then
    local_version="$(read_version_file "$repo_root/research-paper-workflow/VERSION" || true)"
  fi

  codex_dir="${CODEX_HOME:-$HOME/.codex}/skills/research-paper-workflow"
  claude_dir="${CLAUDE_CODE_HOME:-$HOME/.claude}/skills/research-paper-workflow"
  gemini_dir="${GEMINI_HOME:-$HOME/.gemini}/skills/research-paper-workflow"
  antigravity_dir="${ANTIGRAVITY_HOME:-$HOME/.gemini/antigravity}/skills/research-paper-workflow"
  codex_version="$(read_version_file "$codex_dir/VERSION" || true)"
  claude_version="$(read_version_file "$claude_dir/VERSION" || true)"
  gemini_version="$(read_version_file "$gemini_dir/VERSION" || true)"
  antigravity_version="$(read_version_file "$antigravity_dir/VERSION" || true)"

  if latest_tag="$(resolve_latest_tag "$repo" 2>/dev/null)"; then
    latest_status="ok"
  else
    latest_status="unavailable"
    if [[ "$strict_network" -eq 1 ]]; then
      err "failed to resolve latest release tag for $repo"
      return 1
    fi
  fi

  for candidate in "$local_version" "$codex_version" "$claude_version" "$gemini_version" "$antigravity_version"; do
    [[ -n "$candidate" ]] || continue
    if key="$(version_key "$candidate" 2>/dev/null)"; then
      if [[ -z "$best_key" || "$key" > "$best_key" ]]; then
        best_key="$key"
        best_version="$candidate"
      fi
    fi
  done

  if [[ -n "$latest_tag" ]] && key="$(version_key "$latest_tag" 2>/dev/null)"; then
    if [[ -z "$best_key" || "$key" > "$best_key" ]]; then
      update_available=1
    fi
  fi

  if [[ "$json" -eq 1 ]]; then
    cat <<EOF
{
  "cli_package": {
    "installed": "$(json_escape "$CLI_FLAVOR")",
    "latest_pypi": "",
    "status": "shell-cli"
  },
  "repo": "$(json_escape "$repo")",
  "local_repo_version": "$(json_escape "$local_version")",
  "installed": {
    "codex": {
      "path": "$(json_escape "$codex_dir")",
      "installed": $( [[ -d "$codex_dir" ]] && printf 'true' || printf 'false' ),
      "version": "$(json_escape "$codex_version")"
    },
    "claude": {
      "path": "$(json_escape "$claude_dir")",
      "installed": $( [[ -d "$claude_dir" ]] && printf 'true' || printf 'false' ),
      "version": "$(json_escape "$claude_version")"
    },
    "gemini": {
      "path": "$(json_escape "$gemini_dir")",
      "installed": $( [[ -d "$gemini_dir" ]] && printf 'true' || printf 'false' ),
      "version": "$(json_escape "$gemini_version")"
    },
    "antigravity": {
      "path": "$(json_escape "$antigravity_dir")",
      "installed": $( [[ -d "$antigravity_dir" ]] && printf 'true' || printf 'false' ),
      "version": "$(json_escape "$antigravity_version")"
    }
  },
  "latest_release": "$(json_escape "$latest_tag")",
  "latest_release_status": "$(json_escape "$latest_status")"
}
EOF
    return "$update_available"
  fi

  printf 'Research Skills Check\n'
  printf '=====================\n\n'
  printf '1) CLI Package\n'
  printf '   - Installed: %s\n' "$CLI_FLAVOR"
  printf '   - Status: shell CLI installed (python-free)\n\n'
  printf '2) Installed Workflow Skills\n'
  if [[ -n "$repo_root" ]]; then
    printf '   - Detected repo root: %s\n' "$repo_root"
  fi
  if [[ -n "$local_version" ]]; then
    printf '   - Local repo version: %s\n' "$local_version"
  fi
  printf '   - codex:  version=%s, path=%s\n' "${codex_version:-<unknown>}" "$codex_dir"
  printf '   - claude: version=%s, path=%s\n' "${claude_version:-<unknown>}" "$claude_dir"
  printf '   - gemini: version=%s, path=%s\n' "${gemini_version:-<unknown>}" "$gemini_dir"
  printf '   - antigravity: version=%s, path=%s\n' "${antigravity_version:-<unknown>}" "$antigravity_dir"
  printf '\n3) Upstream Release\n'
  printf '   - Repo: %s\n' "$repo"
  if [[ -n "$latest_tag" ]]; then
    printf '   - Latest: %s\n' "$latest_tag"
  else
    printf '   - Latest: <unavailable>\n'
  fi

  if [[ "$update_available" -eq 1 ]]; then
    printf '   - Status: update available -> rsk upgrade --repo %s --project-dir <your-project> --target all\n' "$repo"
    return 1
  fi

  if [[ -n "$latest_tag" ]]; then
    printf '   - Status: up-to-date\n'
  fi
  return 0
}

main() {
  local cmd="${1:-help}"
  shift || true

  case "$cmd" in
    check)
      cmd_check "$@"
      ;;
    upgrade)
      cmd_upgrade "$@"
      ;;
    align)
      if [[ "${1:-}" == "--repo" && $# -ge 2 ]]; then
        cmd_align "$2"
      else
        cmd_align
      fi
      ;;
    help|-h|--help)
      usage
      ;;
    *)
      err "unknown command: $cmd"
      usage
      return 2
      ;;
  esac
}

main "$@"
