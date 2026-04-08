from __future__ import annotations

import os
import shutil
import stat
import subprocess
import sys
from importlib import resources
from dataclasses import dataclass
from pathlib import Path


TARGET_CHOICES = ("codex", "claude", "gemini", "antigravity", "all")
PROFILE_CHOICES = ("partial", "full")
PART_CHOICES = ("globals", "project", "cli", "doctor")
LEGACY_MANIFEST_PATH = Path(__file__).resolve().parents[1] / "install" / "install_manifest.tsv"


@dataclass
class InstallOptions:
    repo_root: Path
    project_dir: Path
    target: str = "all"
    mode: str = "copy"
    overwrite: bool = False
    install_cli: bool | None = None
    cli_dir: Path | None = None
    doctor: bool | None = None
    dry_run: bool = False
    profile: str | None = None
    parts: tuple[str, ...] | None = None


def profile_defaults(profile: str) -> dict[str, bool]:
    if profile == "partial":
        return {"install_cli": False, "doctor": False}
    if profile == "full":
        return {"install_cli": True, "doctor": True}
    raise ValueError(f"Unsupported profile: {profile}")


def apply_profile(options: InstallOptions) -> InstallOptions:
    if not options.profile:
        return options
    defaults = profile_defaults(options.profile)
    return InstallOptions(
        repo_root=options.repo_root,
        project_dir=options.project_dir,
        target=options.target,
        mode=options.mode,
        overwrite=options.overwrite,
        install_cli=defaults["install_cli"] if options.install_cli is None else options.install_cli,
        cli_dir=options.cli_dir,
        doctor=defaults["doctor"] if options.doctor is None else options.doctor,
        dry_run=options.dry_run,
        profile=options.profile,
        parts=options.parts,
    )


def normalize_parts(parts: tuple[str, ...] | list[str] | str | None) -> tuple[str, ...] | None:
    if parts is None:
        return None
    raw_items: list[str]
    if isinstance(parts, str):
        raw_items = [item.strip() for item in parts.split(",")]
    else:
        raw_items = [str(item).strip() for item in parts]

    cleaned = [item for item in raw_items if item]
    if not cleaned:
        return None

    normalized: list[str] = []
    for item in cleaned:
        token = item.lower()
        if token in {"all", "*"}:
            return PART_CHOICES
        if token == "global":
            token = "globals"
        if token == "shell-cli":
            token = "cli"
        if token not in PART_CHOICES:
            raise ValueError(f"Unsupported install part: {item}")
        if token not in normalized:
            normalized.append(token)
    return tuple(normalized)


def cli_name_for_target(target: str) -> str:
    mapping = {
        "codex": "codex",
        "claude": "claude",
        "gemini": "gemini",
        "antigravity": "antigravity",
    }
    return mapping[target]


def cli_install_hint(target: str) -> str:
    hints = {
        "codex": "Install the Codex CLI from the official OpenAI distribution and ensure `codex` is on PATH.",
        "claude": "Install Claude Code: npm install -g @anthropic-ai/claude-code",
        "gemini": "Install Gemini CLI: npm install -g @google/gemini-cli",
        "antigravity": "Install Antigravity and ensure `antigravity` is on PATH before relying on the global skill directory.",
    }
    return hints[target]


def _resolve(path: Path | str) -> Path:
    return Path(path).expanduser().resolve()


def _ensure_dir(path: Path, dry_run: bool) -> None:
    if dry_run:
        return
    path.mkdir(parents=True, exist_ok=True)


def _set_executable(path: Path, dry_run: bool) -> None:
    if dry_run or not path.exists() or path.is_symlink():
        return
    current = path.stat().st_mode
    path.chmod(current | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def _remove_path(path: Path, dry_run: bool) -> None:
    if dry_run or not path.exists() and not path.is_symlink():
        return
    if path.is_dir() and not path.is_symlink():
        shutil.rmtree(path)
    else:
        path.unlink()


def _same_path(src: Path, dest: Path) -> bool:
    try:
        return src.resolve() == dest.resolve()
    except OSError:
        return False


def _copy_path(src: Path, dest: Path, mode: str, overwrite: bool, dry_run: bool) -> tuple[str, str]:
    if _same_path(src, dest):
        return "skip", "same path"
    if dest.exists() or dest.is_symlink():
        if not overwrite:
            return "skip", "exists (use --overwrite)"
        _remove_path(dest, dry_run)
    _ensure_dir(dest.parent, dry_run)
    if dry_run:
        return "ok", "dry-run"
    if mode == "link":
        os.symlink(str(src), str(dest), target_is_directory=src.is_dir())
    elif src.is_dir():
        shutil.copytree(src, dest)
    else:
        shutil.copy2(src, dest)
    return "ok", str(dest)


def _parse_manifest() -> list[dict[str, str]]:
    entries: list[dict[str, str]] = []
    try:
        manifest_text = resources.files("research_skills").joinpath("install_manifest.tsv").read_text(encoding="utf-8")
    except (FileNotFoundError, ModuleNotFoundError):
        manifest_text = LEGACY_MANIFEST_PATH.read_text(encoding="utf-8")
    for raw_line in manifest_text.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        target, op, label, source, destination = raw_line.split("\t")
        entries.append(
            {
                "target": target,
                "op": op,
                "label": label,
                "source": source,
                "destination": destination,
            }
        )
    return entries


def _manifest_entry_part(entry: dict[str, str]) -> str:
    return "project" if "${PROJECT_DIR}" in entry["destination"] else "globals"


def _expand_path(template: str, values: dict[str, str]) -> Path:
    result = template
    for key, value in values.items():
        result = result.replace("${" + key + "}", value)
    return Path(result)


def _on_path(directory: Path) -> bool:
    target = str(directory)
    for entry in os.environ.get("PATH", "").split(os.pathsep):
        if not entry:
            continue
        try:
            if str(Path(entry).expanduser().resolve()) == target:
                return True
        except OSError:
            continue
    return False


def _print_result(label: str, dest: str, status: str) -> None:
    if status == "ok":
        print(f"  [ok]   {label:<12} -> {dest}")
    else:
        print(f"  [skip] {label:<12} -> {dest}")


def _print_section(title: str) -> None:
    print(f"\n== {title} ==")


def _copy_display(src: Path, dest: Path, label: str, options: InstallOptions) -> None:
    status, detail = _copy_path(src, dest, options.mode, options.overwrite, options.dry_run)
    _print_result(label, str(dest) if status == "ok" else f"{dest} ({detail})", status)


def _install_alias_copy(src: Path, dest: Path, options: InstallOptions) -> None:
    status, detail = _copy_path(src, dest, "copy", options.overwrite, options.dry_run)
    _print_result("Alias", str(dest) if status == "ok" else f"{dest} ({detail})", status)
    if status == "ok":
        _set_executable(dest, options.dry_run)


def _windows_shell_cli_available() -> bool:
    return shutil.which("bash") is not None


def _install_shell_cli(options: InstallOptions) -> None:
    assert options.cli_dir is not None
    repo_root = options.repo_root
    cli_src = repo_root / "scripts" / "research_skills_cli.sh"
    bootstrap_src = repo_root / "scripts" / "bootstrap_research_skill.sh"
    cli_dir = options.cli_dir
    cli_dest = cli_dir / "research-skills"
    bootstrap_dest = cli_dir / "research-skills-bootstrap"

    _print_section("Shell CLI")

    if os.name == "nt" and not _windows_shell_cli_available():
        print("  [skip] Shell CLI    -> Git Bash / `bash` not found on Windows")
        print("          Hint: winget install -e --id Git.Git --source winget")
        return

    _copy_display(cli_src, cli_dest, "CLI", options)
    _set_executable(cli_dest, options.dry_run)
    _copy_display(bootstrap_src, bootstrap_dest, "Bootstrap", options)
    _set_executable(bootstrap_dest, options.dry_run)

    if os.name == "nt":
        for name in ("rsk", "rsw"):
            alias_dest = cli_dir / name
            _install_alias_copy(cli_src, alias_dest, options)
    else:
        for name in ("rsk", "rsw"):
            alias_dest = cli_dir / name
            if alias_dest.exists() or alias_dest.is_symlink():
                if not options.overwrite:
                    _print_result("Alias", f"{alias_dest} (exists, use --overwrite)", "skip")
                    continue
                _remove_path(alias_dest, options.dry_run)
            _ensure_dir(alias_dest.parent, options.dry_run)
            if options.dry_run:
                _print_result("Alias", str(alias_dest), "ok")
                continue
            try:
                os.symlink(str(cli_dest), str(alias_dest))
            except OSError:
                shutil.copy2(cli_dest, alias_dest)
            _set_executable(alias_dest, options.dry_run)
            _print_result("Alias", str(alias_dest), "ok")

    if _on_path(cli_dir):
        print(f"  [info] cli dir on PATH: {cli_dir}")
    else:
        print(f"  [warn] CLI installed to {cli_dir} but this directory is not on PATH")


# Legacy project-local copy helpers removed — workflows are now bundled
# inside the skill directory and installed globally with each dir-copy.


# ── Sync skill package ───────────────────────────────────────────────────────

_SYNC_DIRS = ("skills", "templates", "standards", "roles")
_SYNC_FILES = ("skills-core.md", "skills-summary.md")
_SYNC_EXCLUDE = {"CLAUDE.project.md"}


def _sync_skill_package(repo_root: Path, *, dry_run: bool = False) -> None:
    """Populate research-paper-workflow/ with bundled copies of repo assets.

    The canonical source of truth remains the repo-root directories.
    These copies are .gitignore'd and regenerated on every install/upgrade.
    """
    pkg_dir = repo_root / "research-paper-workflow"
    if not pkg_dir.is_dir():
        return

    _print_section("Sync Skill Package")
    for dir_name in _SYNC_DIRS:
        src = repo_root / dir_name
        dest = pkg_dir / dir_name
        if not src.is_dir():
            _print_result("Sync", f"{dir_name}/ (source not found)", "skip")
            continue
        if dry_run:
            _print_result("Sync", f"{dir_name}/ (dry-run)", "skip")
            continue
        # Remove stale destination and copy fresh
        if dest.exists():
            shutil.rmtree(dest)
        shutil.copytree(
            src, dest,
            ignore=shutil.ignore_patterns(
                ".DS_Store", "__pycache__", *_SYNC_EXCLUDE,
            ),
        )
        file_count = sum(1 for _ in dest.rglob("*") if _.is_file())
        _print_result("Sync", f"{dir_name}/ ({file_count} files)", "ok")

    for file_name in _SYNC_FILES:
        src = repo_root / file_name
        dest = pkg_dir / file_name
        if not src.is_file():
            _print_result("Sync", f"{file_name} (source not found)", "skip")
            continue
        if dry_run:
            _print_result("Sync", f"{file_name} (dry-run)", "skip")
            continue
        shutil.copy2(src, dest)
        _print_result("Sync", file_name, "ok")


# ── Workflow symlink shims ───────────────────────────────────────────────────

# Maps target → (discovery_dir_name, skill_dest_env_var_key)
# Claude: ~/.claude/commands/<name>.md  (slash command discovery)
# Gemini: ~/.gemini/workflows/<name>.md (workflow discovery)
_SYMLINK_TARGETS: dict[str, tuple[str, str]] = {
    "claude": ("commands", "CLAUDE_CODE_HOME"),
    "gemini": ("workflows", "GEMINI_HOME"),
}


def _create_workflow_symlinks(
    target: str,
    skill_dest: Path,
    *,
    dry_run: bool = False,
) -> None:
    """Create symlinks from canonical workflow discovery paths to bundled workflows.

    For Claude Code:  ~/.claude/commands/<name>.md → ~/.claude/skills/.../workflows/<name>.md
    For Gemini CLI:   ~/.gemini/workflows/<name>.md → ~/.gemini/skills/.../workflows/<name>.md

    This enables direct /slash-command invocation (e.g. /paper, /lit-review).
    """
    if target not in _SYMLINK_TARGETS:
        return

    dir_name, _env_key = _SYMLINK_TARGETS[target]
    workflows_src = skill_dest / "workflows"
    if not workflows_src.is_dir():
        return

    # Discovery dir is sibling to skills/ under the client home
    # skill_dest = ~/.claude/skills/research-paper-workflow
    # discovery_dir = ~/.claude/commands/
    client_home = skill_dest.parent.parent  # ~/.claude or ~/.gemini
    discovery_dir = client_home / dir_name
    discovery_dir.mkdir(parents=True, exist_ok=True)

    workflow_files = sorted(workflows_src.glob("*.md"))
    created = 0
    for wf in workflow_files:
        link_path = discovery_dir / wf.name
        target_path = wf  # absolute path to the bundled workflow

        if dry_run:
            _print_result("Symlink", f"{wf.name} (dry-run)", "skip")
            continue

        # Remove stale link or file if it exists
        if link_path.is_symlink() or link_path.exists():
            link_path.unlink()

        link_path.symlink_to(target_path)
        created += 1

    if not dry_run and created > 0:
        _print_result("Symlinks", f"{created} workflows -> {discovery_dir}", "ok")


def _print_cli_checks(target: str) -> bool:
    found_antigravity = False
    _print_section("CLI Checks")
    targets = TARGET_CHOICES[:-1] if target == "all" else (target,)
    for item in targets:
        cli_name = cli_name_for_target(item)
        resolved = shutil.which(cli_name)
        if resolved:
            _print_result("CLI", f"{item} -> {resolved}", "ok")
            if item == "antigravity":
                found_antigravity = True
            continue
        _print_result("CLI", f"{item} -> missing", "skip")
        print(f"          Hint: {cli_install_hint(item)}")
    return found_antigravity


def _print_full_readiness(options: InstallOptions) -> None:
    if options.profile != "full":
        return
    _print_section("Full Profile Readiness")
    version = sys.version_info
    python_status = "ok" if (version.major, version.minor) >= (3, 12) else "skip"
    _print_result("Python", f"{sys.executable} ({version.major}.{version.minor}.{version.micro})", python_status)
    if python_status != "ok":
        print("          Hint: install Python >= 3.12, preferably with mise")
    for env_var in ("OPENAI_API_KEY", "ANTHROPIC_API_KEY", "GOOGLE_API_KEY"):
        value = os.environ.get(env_var, "").strip()
        _print_result(env_var, "configured" if value else "missing", "ok" if value else "skip")
    if os.name == "nt":
        has_bash = shutil.which("bash") is not None
        _print_result("Windows Bash", "available" if has_bash else "missing", "ok" if has_bash else "skip")
        if not has_bash:
            print("          Hint: winget install -e --id Git.Git --source winget")


def _run_doctor(project_dir: Path, dry_run: bool) -> None:
    _print_section("Doctor")
    if dry_run:
        print(f"  [ok]   Doctor       -> dry-run ({project_dir})")
        return
    repo_root = Path(__file__).resolve().parents[1]
    env = os.environ.copy()
    existing_pythonpath = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = str(repo_root) if not existing_pythonpath else f"{repo_root}{os.pathsep}{existing_pythonpath}"
    result = subprocess.run(
        [sys.executable, "-m", "bridges.orchestrator", "doctor", "--cwd", str(project_dir)],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
        env=env,
    )
    print(result.stdout.strip() if result.stdout.strip() else "  [warn] doctor produced no output")
    if result.returncode != 0:
        print(f"  [warn] doctor exited with code {result.returncode}")


def install(options: InstallOptions) -> int:
    options = apply_profile(
        InstallOptions(
            repo_root=_resolve(options.repo_root),
            project_dir=_resolve(options.project_dir),
            target=options.target,
            mode=options.mode,
            overwrite=options.overwrite,
            install_cli=options.install_cli,
            cli_dir=_resolve(options.cli_dir or Path.home() / ".local" / "bin"),
            doctor=options.doctor,
            dry_run=options.dry_run,
            profile=options.profile,
            parts=options.parts,
        )
    )

    if options.target not in TARGET_CHOICES:
        raise ValueError(f"Unsupported target: {options.target}")
    if options.mode not in {"copy", "link"}:
        raise ValueError(f"Unsupported mode: {options.mode}")
    selected_parts = normalize_parts(options.parts)
    install_globals = True if selected_parts is None else "globals" in selected_parts
    install_project = False if selected_parts is None else "project" in selected_parts
    install_cli = bool(options.install_cli) if selected_parts is None else "cli" in selected_parts
    doctor = bool(options.doctor) if selected_parts is None else "doctor" in selected_parts

    repo_root = options.repo_root
    skill_src = repo_root / "research-paper-workflow"
    if not (skill_src / "SKILL.md").exists():
        raise FileNotFoundError(f"Missing skill source: {skill_src / 'SKILL.md'}")

    codex_dest = Path(os.environ.get("CODEX_HOME", str(Path.home() / ".codex"))) / "skills" / "research-paper-workflow"
    claude_dest = Path(os.environ.get("CLAUDE_CODE_HOME", str(Path.home() / ".claude"))) / "skills" / "research-paper-workflow"
    gemini_dest = Path(os.environ.get("GEMINI_HOME", str(Path.home() / ".gemini"))) / "skills" / "research-paper-workflow"
    antigravity_dest = Path(os.environ.get("ANTIGRAVITY_HOME", str(Path.home() / ".gemini" / "antigravity"))) / "skills" / "research-paper-workflow"
    manifest_values = {
        "PROJECT_DIR": str(options.project_dir),
        "CODEX_HOME": str(codex_dest.parent.parent),
        "CLAUDE_CODE_HOME": str(claude_dest.parent.parent),
        "GEMINI_HOME": str(gemini_dest.parent.parent),
        "ANTIGRAVITY_HOME": str(antigravity_dest.parent.parent),
    }
    manifest_entries = _parse_manifest()

    print("\nResearch Skills Universal Installer")
    print(f"  source:  {repo_root}")
    print(f"  project: {options.project_dir}")
    print(f"  target:  {options.target} | mode: {options.mode}")
    if options.profile:
        print(f"  profile: {options.profile}")
    if selected_parts is not None:
        print(f"  parts:   {', '.join(selected_parts)}")
    if install_cli:
        print(f"  cli:     install -> {options.cli_dir}")

    _print_full_readiness(options)
    _print_cli_checks(options.target)

    # Sync bundled assets into the skill package before dir-copy
    if install_globals and not options.dry_run:
        _sync_skill_package(repo_root, dry_run=options.dry_run)

    section_targets = ("codex", "claude", "gemini", "antigravity")
    for section_target in section_targets:
        if options.target not in {section_target, "all"}:
            continue
        entries_for_target = [
            entry
            for entry in manifest_entries
            if entry["target"] == section_target
            and (
                (install_globals and _manifest_entry_part(entry) == "globals")
                or (install_project and _manifest_entry_part(entry) == "project")
            )
        ]
        if not entries_for_target:
            continue

        _print_section(section_target.capitalize() if section_target != "antigravity" else "Antigravity")
        for entry in entries_for_target:
            op = entry["op"]
            label = entry["label"]
            src = repo_root / entry["source"]
            dest = _expand_path(entry["destination"], manifest_values)

            if op in {"dir-copy", "file-copy"}:
                _copy_display(src, dest, label, options)
                continue

            raise ValueError(f"Unsupported manifest operation: {op}")

    # Create workflow discovery symlinks (Claude: commands/, Gemini: workflows/)
    if install_globals and not options.dry_run:
        _print_section("Workflow Discovery")
        target_dest_map = {
            "claude": claude_dest,
            "gemini": gemini_dest,
        }
        for sym_target, sym_dest in target_dest_map.items():
            if options.target in {sym_target, "all"}:
                _create_workflow_symlinks(sym_target, sym_dest, dry_run=options.dry_run)

    if install_cli:
        _install_shell_cli(options)

    if install_project:
        _print_section("Project Env")
        for entry in manifest_entries:
            if entry["target"] != "project":
                continue
            _copy_display(repo_root / entry["source"], _expand_path(entry["destination"], manifest_values), entry["label"], options)

    if doctor:
        _run_doctor(options.project_dir, options.dry_run)

    print("\n[done] Installation complete")
    if install_cli and options.cli_dir and not _on_path(options.cli_dir):
        print(f"       Add {options.cli_dir} to PATH to use research-skills / rsk / rsw")
    print("       Restart Codex / Claude Code / Gemini CLI to activate changes")
    return 0


# ── Clean stale project-local assets ─────────────────────────────────────────

_CLEANABLE_GLOBS = (
    ".agent/workflows/*.md",
    ".agent/skills/research-paper-workflow",
    ".agents/skills/research-paper-workflow",
    "CLAUDE.research-skills.md",
    ".gemini/research-skills.md",
    ".gemini/agent-profiles.example.json",
)

_CLEANABLE_CONDITIONAL = (
    # Only remove these if their content matches the template we used to install
    "CLAUDE.md",
)


def _is_rsk_claude_md(path: Path, repo_root: Path | None) -> bool:
    """Return True if `path` looks like a research-skills template CLAUDE.md."""
    if not path.is_file():
        return False
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return False
    return "Academic Deep Research Skills" in text and "research-paper-workflow" in text


def clean(project_dir: Path, *, dry_run: bool = False, repo_root: Path | None = None) -> int:
    """Remove stale project-local research-skills assets."""
    project_dir = _resolve(project_dir)
    removed = 0

    _print_section("Clean stale project-local assets")
    for pattern in _CLEANABLE_GLOBS:
        candidates = sorted(project_dir.glob(pattern))
        for path in candidates:
            _remove_path(path, dry_run)
            _print_result("Removed", str(path), "ok")
            removed += 1
    # If no wildcard matches were found for the parent dir, it might be empty now
    for parent in {"agent/workflows", "agent/skills", "agents/skills"}:
        parent_path = project_dir / f".{parent}"
        if parent_path.is_dir() and not any(parent_path.iterdir()):
            _remove_path(parent_path, dry_run)
            _print_result("Removed", f"{parent_path}/ (empty)", "ok")
            removed += 1

    # Conditional: CLAUDE.md only if it matches our template
    claude_md = project_dir / "CLAUDE.md"
    if _is_rsk_claude_md(claude_md, repo_root):
        _remove_path(claude_md, dry_run)
        _print_result("Removed", str(claude_md), "ok")
        removed += 1
    elif claude_md.exists():
        _print_result("Kept", f"{claude_md} (user-customized)", "skip")

    if removed:
        print(f"\n[done] Cleaned {removed} stale asset(s) from {project_dir}")
    else:
        print(f"\n[done] No stale assets found in {project_dir}")
    return 0


def clean_workflow_symlinks(*, dry_run: bool = False) -> int:
    """Remove workflow discovery symlinks created by the installer.

    Cleans: ~/.claude/commands/<name>.md and ~/.gemini/workflows/<name>.md
    Only removes symlinks that point into a research-paper-workflow directory.
    """
    removed = 0
    _print_section("Clean workflow discovery symlinks")

    for target, (dir_name, env_key) in _SYMLINK_TARGETS.items():
        home = Path(os.environ.get(env_key, str(Path.home() / f".{target}")))
        discovery_dir = home / dir_name
        if not discovery_dir.is_dir():
            continue
        for link in sorted(discovery_dir.iterdir()):
            if not link.is_symlink():
                continue
            target_path = str(link.resolve())
            if "research-paper-workflow" in target_path:
                _remove_path(link, dry_run)
                _print_result("Removed", f"{link.name} -> {target}", "ok")
                removed += 1

    if removed:
        print(f"\n[done] Removed {removed} workflow symlink(s)")
    else:
        print(f"\n[done] No workflow symlinks found")
    return 0
