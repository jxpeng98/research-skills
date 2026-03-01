from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path

from . import __version__

TAG_PATTERN = re.compile(r"^v(\d+)\.(\d+)\.(\d+)(?:-beta\.(\d+))?$")
RELEASE_NOTE_PATTERN = re.compile(r"^v(\d+)\.(\d+)\.(\d+)-beta\.(\d+)\.md$")
OWNER_REPO_PATTERN = re.compile(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$")


@dataclass(frozen=True)
class Version:
    major: int
    minor: int
    patch: int
    beta: int | None = None

    @classmethod
    def parse(cls, raw: str) -> "Version | None":
        match = TAG_PATTERN.match(raw.strip())
        if not match:
            return None
        major, minor, patch = (int(match.group(i)) for i in range(1, 4))
        beta_raw = match.group(4)
        beta = int(beta_raw) if beta_raw is not None else None
        return cls(major=major, minor=minor, patch=patch, beta=beta)

    def sort_key(self) -> tuple[int, int, int, int]:
        beta_key = self.beta if self.beta is not None else 10**9
        return (self.major, self.minor, self.patch, beta_key)

    def __str__(self) -> str:
        base = f"v{self.major}.{self.minor}.{self.patch}"
        if self.beta is None:
            return base
        return f"{base}-beta.{self.beta}"


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8").strip()


def _find_repo_root(start: Path) -> Path | None:
    for candidate in (start, *start.parents):
        if (candidate / "standards" / "research-workflow-contract.yaml").exists():
            return candidate
    return None


def _normalize_repo_spec(raw: str) -> str:
    value = str(raw or "").strip()
    if not value:
        raise ValueError("empty repo spec")
    if OWNER_REPO_PATTERN.match(value):
        return value

    # Accept Git URLs and convert to owner/repo.
    # Examples:
    # - https://github.com/owner/repo
    # - https://github.com/owner/repo.git
    # - git@github.com:owner/repo.git
    # - ssh://git@github.com/owner/repo.git
    if value.startswith("git@"):
        match = re.match(r"^git@[^:]+:(?P<path>.+)$", value)
        if match:
            value = "ssh://" + value.replace(":", "/", 1)

    if "://" in value:
        from urllib.parse import urlparse

        parsed = urlparse(value)
        path = (parsed.path or "").strip("/")
        if path.endswith(".git"):
            path = path[: -len(".git")]
        parts = [part for part in path.split("/") if part]
        if len(parts) >= 2:
            owner, repo = parts[0], parts[1]
            candidate = f"{owner}/{repo}"
            if OWNER_REPO_PATTERN.match(candidate):
                return candidate

    raise ValueError(f"unsupported repo spec: {raw!r} (expected owner/repo or Git URL)")


def _infer_repo_from_env() -> str | None:
    raw = os.getenv("RESEARCH_SKILLS_REPO", "").strip()
    if not raw:
        return None
    return _normalize_repo_spec(raw)


def _infer_repo_from_git(repo_root: Path) -> tuple[str | None, str]:
    for remote in ("upstream", "origin"):
        try:
            result = subprocess.run(
                ["git", "remote", "get-url", remote],
                cwd=str(repo_root),
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True,
                check=False,
            )
        except OSError:
            return None, ""
        url = (result.stdout or "").strip()
        if not url:
            continue
        try:
            return _normalize_repo_spec(url), f"git:{remote}"
        except ValueError:
            continue
    return None, ""


def _read_upstream_repo_from_toml(path: Path) -> str | None:
    try:
        content = path.read_text(encoding="utf-8")
    except OSError:
        return None

    in_upstream = False
    repo_value = ""
    url_value = ""

    for raw_line in content.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1].strip()
            in_upstream = section == "upstream"
            continue
        if not in_upstream:
            continue
        match = re.match(r"^(?P<key>[A-Za-z0-9_.-]+)\s*=\s*(?P<value>.+?)\s*$", line)
        if not match:
            continue
        key = match.group("key").strip().lower()
        value = match.group("value").strip()
        if "#" in value:
            value = value.split("#", 1)[0].strip()
        if (
            len(value) >= 2
            and ((value.startswith('"') and value.endswith('"')) or (value.startswith("'") and value.endswith("'")))
        ):
            value = value[1:-1]
        if not value:
            continue
        if key in {"repo", "repo_slug", "upstream_repo"}:
            repo_value = value
        if key in {"url", "repo_url", "remote_url", "upstream_url"}:
            url_value = value

    candidate = repo_value or url_value
    if not candidate:
        return None
    try:
        return _normalize_repo_spec(candidate)
    except ValueError:
        return None


def _infer_repo_from_project_config(start: Path) -> tuple[str | None, str]:
    for candidate in (start, *start.parents):
        for name in ("research-skills.toml", ".research-skills.toml"):
            path = candidate / name
            if not path.exists():
                continue
            repo = _read_upstream_repo_from_toml(path)
            if repo:
                return repo, f"config:{path}"
    return None, ""


def _infer_repo_from_packaged_defaults() -> tuple[str | None, str]:
    path = Path(__file__).resolve().parent / "project.toml"
    if not path.exists():
        return None, ""
    repo = _read_upstream_repo_from_toml(path)
    if not repo:
        return None, ""
    return repo, "package"


def _resolve_upstream_repo(
    args_repo: str | None, repo_root: Path | None, config_start: Path | None = None
) -> tuple[str | None, str]:
    if args_repo and str(args_repo).strip():
        return _normalize_repo_spec(args_repo), "arg"

    env_repo = _infer_repo_from_env()
    if env_repo:
        return env_repo, "env"

    start = config_start or Path.cwd()
    config_repo, config_source = _infer_repo_from_project_config(start)
    if config_repo:
        return config_repo, config_source

    packaged_repo, packaged_source = _infer_repo_from_packaged_defaults()
    if packaged_repo:
        return packaged_repo, packaged_source

    if repo_root:
        inferred, source = _infer_repo_from_git(repo_root)
        if inferred:
            return inferred, source

    return None, ""


def _local_repo_version(root: Path) -> tuple[str, Version] | None:
    version_path = root / "research-paper-workflow" / "VERSION"
    if version_path.exists():
        raw = _read_text(version_path)
        parsed = Version.parse(raw)
        if parsed:
            return raw, parsed

    release_dir = root / "release"
    if not release_dir.exists():
        return None
    candidates: list[tuple[Version, str]] = []
    for item in release_dir.iterdir():
        if not item.is_file():
            continue
        match = RELEASE_NOTE_PATTERN.match(item.name)
        if not match:
            continue
        major, minor, patch, beta = (int(match.group(i)) for i in range(1, 5))
        version = Version(major=major, minor=minor, patch=patch, beta=beta)
        candidates.append((version, str(version)))
    if not candidates:
        return None
    candidates.sort(key=lambda pair: pair[0].sort_key(), reverse=True)
    chosen_version, chosen_tag = candidates[0]
    return chosen_tag, chosen_version


def _installed_skill_dirs() -> dict[str, Path]:
    codex_home = Path(os.environ.get("CODEX_HOME", "~/.codex")).expanduser()
    claude_home = Path(os.environ.get("CLAUDE_CODE_HOME", "~/.claude")).expanduser()
    gemini_home = Path(os.environ.get("GEMINI_HOME", "~/.gemini")).expanduser()
    return {
        "codex": codex_home / "skills" / "research-paper-workflow",
        "claude": claude_home / "skills" / "research-paper-workflow",
        "gemini": gemini_home / "skills" / "research-paper-workflow",
    }


def _read_installed_version(skill_dir: Path) -> tuple[str, Version] | None:
    version_path = skill_dir / "VERSION"
    if not version_path.exists():
        return None
    raw = _read_text(version_path)
    parsed = Version.parse(raw)
    if not parsed:
        return None
    return raw, parsed


def _github_token() -> str:
    return os.environ.get("GITHUB_TOKEN", "").strip() or os.environ.get("GH_TOKEN", "").strip()


def _http_get_json(url: str) -> dict:
    headers = {
        "User-Agent": "research-skills-updater",
        "Accept": "application/vnd.github+json",
    }
    token = _github_token()
    if token:
        headers["Authorization"] = f"Bearer {token}"
    request = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(request, timeout=20) as response:
        payload = response.read().decode("utf-8", errors="replace")
    return json.loads(payload)


def _latest_release_tag(repo: str) -> str:
    api_url = f"https://api.github.com/repos/{repo}/releases/latest"
    try:
        payload = _http_get_json(api_url)
        tag = str(payload.get("tag_name", "")).strip()
        if tag:
            return tag
    except (urllib.error.URLError, urllib.error.HTTPError, json.JSONDecodeError):
        pass

    html_url = f"https://github.com/{repo}/releases/latest"
    headers = {"User-Agent": "research-skills-updater"}
    request = urllib.request.Request(html_url, headers=headers)
    with urllib.request.urlopen(request, timeout=20) as response:
        final_url = response.geturl()
    tag = final_url.rstrip("/").split("/")[-1].strip()
    if not tag:
        raise RuntimeError(f"Unable to resolve latest release tag from {final_url}")
    return tag


def cmd_check(args: argparse.Namespace) -> int:
    repo_root = _find_repo_root(Path.cwd())
    local = _local_repo_version(repo_root) if repo_root else None
    installed: dict[str, dict[str, object]] = {}
    for client, path in _installed_skill_dirs().items():
        installed[client] = {
            "path": str(path),
            "installed": path.exists(),
            "version": None,
        }
        found = _read_installed_version(path)
        if found:
            installed[client]["version"] = found[0]

    resolved_repo, resolved_source = _resolve_upstream_repo(getattr(args, "repo", None), repo_root)

    latest_tag = ""
    latest_version: Version | None = None
    if resolved_repo:
        try:
            latest_tag = _latest_release_tag(resolved_repo)
            latest_version = Version.parse(latest_tag)
        except Exception as exc:  # noqa: BLE001
            if args.strict_network:
                raise
            latest_tag = f"<unavailable: {exc}>"

    payload = {
        "repo": resolved_repo or "",
        "repo_source": resolved_source or "",
        "local_repo_version": local[0] if local else "",
        "installed": installed,
        "latest_release": latest_tag,
    }

    if args.json:
        print(json.dumps(payload, ensure_ascii=False, indent=2))
        return 0

    print("Research Skills Update Check")
    print("")
    print(f"- CLI version: {__version__}")
    if repo_root:
        print(f"- Detected repo root: {repo_root}")
    if local:
        print(f"- Local repo version: {local[0]}")
    print("")
    print("Installed skill versions:")
    for client in ("codex", "claude", "gemini"):
        item = installed[client]
        status = "installed" if item["installed"] else "not-installed"
        version = item["version"] or "<unknown>"
        print(f"- {client}: {status}, version={version}, path={item['path']}")
    print("")
    if resolved_repo:
        suffix = f" (from {resolved_source})" if resolved_source else ""
        print(f"- Upstream repo: {resolved_repo}{suffix}")
        print(f"- Latest upstream release: {latest_tag}")
    else:
        print(
            "- Latest upstream release: <skipped (pass --repo, set RESEARCH_SKILLS_REPO, or add research-skills.toml)>"
        )

    if latest_version:
        local_versions: list[Version] = []
        if local:
            local_versions.append(local[1])
        for client in installed.values():
            raw = str(client.get("version") or "").strip()
            if not raw:
                continue
            parsed = Version.parse(raw)
            if parsed:
                local_versions.append(parsed)
        if local_versions and latest_version.sort_key() > max(v.sort_key() for v in local_versions):
            print("")
            print("Update available.")
            print(
                "Run: research-skills upgrade "
                f"--repo {resolved_repo} --project-dir <your-project> --target all"
            )
            return 1

    return 0


def _download(url: str, dest: Path) -> None:
    headers = {"User-Agent": "research-skills-updater"}
    token = _github_token()
    if token:
        headers["Authorization"] = f"Bearer {token}"
    request = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(request, timeout=60) as response:
        with dest.open("wb") as handle:
            shutil.copyfileobj(response, handle)


def _safe_extract_tar(tar: tarfile.TarFile, dest_dir: Path) -> None:
    dest_real = dest_dir.resolve()
    for member in tar.getmembers():
        if not member.name:
            continue
        member_path = (dest_dir / member.name).resolve()
        if dest_real not in member_path.parents and member_path != dest_real:
            raise RuntimeError(f"Unsafe path in archive member: {member.name}")
    tar.extractall(dest_dir)


def _extract_tarball(tar_path: Path, dest_dir: Path) -> Path:
    with tarfile.open(tar_path, "r:gz") as tar:
        members = tar.getmembers()
        top_levels = {m.name.split("/", 1)[0] for m in members if m.name and "/" in m.name}
        _safe_extract_tar(tar, dest_dir)
    if not top_levels:
        raise RuntimeError("Archive extraction succeeded but no top-level folder detected.")
    if len(top_levels) == 1:
        return dest_dir / next(iter(top_levels))
    for candidate in sorted(top_levels):
        probe = dest_dir / candidate / "scripts" / "install_research_skill.sh"
        if probe.exists():
            return dest_dir / candidate
    return dest_dir / sorted(top_levels)[0]


def cmd_upgrade(args: argparse.Namespace) -> int:
    project_dir = Path(args.project_dir).expanduser().resolve()

    repo_root = _find_repo_root(Path.cwd())
    resolved_repo, _ = _resolve_upstream_repo(
        getattr(args, "repo", None),
        repo_root,
        config_start=project_dir,
    )
    if not resolved_repo:
        print(
            "[error] missing upstream repo. Pass `--repo owner/repo` (or set RESEARCH_SKILLS_REPO / add research-skills.toml).",
            file=sys.stderr,
        )
        return 2

    ref = args.ref
    ref_type = args.ref_type
    if not ref:
        ref = _latest_release_tag(resolved_repo)
        ref_type = "tag"

    if ref_type == "tag":
        tar_url = f"https://github.com/{resolved_repo}/archive/refs/tags/{ref}.tar.gz"
    else:
        tar_url = f"https://github.com/{resolved_repo}/archive/refs/heads/{ref}.tar.gz"

    with tempfile.TemporaryDirectory(prefix="research-skills-upgrade-") as temp_dir:
        temp_root = Path(temp_dir)
        archive_path = temp_root / "repo.tar.gz"
        print(f"[upgrade] download: {tar_url}")
        _download(tar_url, archive_path)
        extracted_root = _extract_tarball(archive_path, temp_root / "src")
        install_script = extracted_root / "scripts" / "install_research_skill.sh"
        if not install_script.exists():
            print(f"[error] install script not found in archive: {install_script}", file=sys.stderr)
            return 1

        cmd = [
            "bash",
            str(install_script),
            "--target",
            args.target,
            "--mode",
            args.mode,
            "--project-dir",
            str(project_dir),
        ]
        if args.overwrite:
            cmd.append("--overwrite")
        if args.doctor:
            cmd.append("--doctor")
        if args.dry_run:
            cmd.append("--dry-run")

        print("[upgrade] install: " + " ".join(cmd))
        result = subprocess.run(cmd, cwd=str(extracted_root), check=False)
        return int(result.returncode)

def cmd_align(args: argparse.Namespace) -> int:
    repo_hint = (
        args.repo.strip()
        if getattr(args, "repo", None) and str(args.repo).strip()
        else "<owner>/<repo>"
    )
    prog = Path(sys.argv[0]).name.strip() if sys.argv and sys.argv[0] else "research-skills"
    if not prog:
        prog = "research-skills"

    print(f"{prog} — Quick Reference")
    print("")
    print("What pipx installs:")
    print("- A global CLI (per-user). It does NOT auto-modify your projects.")
    print("- CLI aliases: `research-skills`, `rs`, `rsw` (same behavior).")
    print("")
    print(f"What `{prog} upgrade` modifies:")
    print("- Global skills: ~/.codex|~/.claude|~/.gemini under `skills/research-paper-workflow/`")
    print("- One project: `<project>/.agent/workflows/`, `CLAUDE.md`, `.gemini/` (via --project-dir)")
    print("")
    print("Typical usage:")
    print(f"1) Check:   {prog} check --repo {repo_hint}")
    print(f"2) Upgrade: {prog} upgrade --repo {repo_hint} --project-dir . --target all --doctor")
    print("")
    print("Tip:")
    print(f"- Run `{prog} upgrade` from your project root to use `--project-dir .`.")
    print("- Set `RESEARCH_SKILLS_REPO=owner/repo` to avoid passing `--repo` every time.")
    print("- Or add `research-skills.toml` to your project root to persist the upstream repo.")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Install/upgrade research-skills (Codex/Claude/Gemini) without requiring a git fork."
    )
    subparsers = parser.add_subparsers(dest="cmd", required=True)

    check = subparsers.add_parser("check", help="Check installed versions and latest upstream release")
    check.add_argument(
        "--repo",
        help=(
            "Upstream repo in owner/repo form (or Git URL). Optional if RESEARCH_SKILLS_REPO is set, "
            "or when running inside a research-skills repo clone, or via a project config file."
        ),
    )
    check.add_argument("--json", action="store_true", help="Emit JSON only")
    check.add_argument(
        "--strict-network",
        action="store_true",
        help="Fail if upstream version check fails (default: warn and continue)",
    )

    upgrade = subparsers.add_parser("upgrade", help="Download release archive and run installer with overwrite")
    upgrade.add_argument(
        "--repo",
        help=(
            "Upstream repo in owner/repo form (or Git URL). Optional if RESEARCH_SKILLS_REPO is set, "
            "or via a project config file."
        ),
    )
    upgrade.add_argument("--ref", help="Tag or branch name (default: latest release tag)")
    upgrade.add_argument(
        "--ref-type",
        choices=["tag", "branch"],
        default="tag",
        help="How to interpret --ref (default: tag; latest uses tag)",
    )
    upgrade.add_argument(
        "--target",
        default="all",
        choices=["codex", "claude", "gemini", "all"],
        help="Install target (default: all)",
    )
    upgrade.add_argument(
        "--mode",
        default="copy",
        choices=["copy", "link"],
        help="Install mode (default: copy)",
    )
    upgrade.add_argument(
        "--project-dir",
        default=str(Path.cwd()),
        help="Project directory for workflow integration (default: current dir)",
    )
    upgrade.add_argument(
        "--overwrite",
        action="store_true",
        default=True,
        help="Overwrite existing installs (default: on)",
    )
    upgrade.add_argument(
        "--no-overwrite",
        action="store_false",
        dest="overwrite",
        help="Do not overwrite existing installs",
    )
    upgrade.add_argument("--doctor", action="store_true", help="Run orchestrator doctor after install")
    upgrade.add_argument("--dry-run", action="store_true", help="Show install actions only")

    align = subparsers.add_parser("align", help="Print a short usage alignment (what installs where)")
    align.add_argument("--repo", help="Optional upstream repo in owner/repo form (used in examples)")

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    if args.cmd == "check":
        return cmd_check(args)
    if args.cmd == "upgrade":
        return cmd_upgrade(args)
    if args.cmd == "align":
        return cmd_align(args)
    raise RuntimeError(f"Unhandled command: {args.cmd}")


if __name__ == "__main__":
    sys.exit(main())
