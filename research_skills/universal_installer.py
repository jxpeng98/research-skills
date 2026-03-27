from __future__ import annotations

import os
import shutil
import stat
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


TARGET_CHOICES = ("codex", "claude", "gemini", "antigravity", "all")
PROFILE_CHOICES = ("partial", "full")
MANIFEST_PATH = Path(__file__).resolve().parents[1] / "install" / "install_manifest.tsv"


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
    )


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
    for raw_line in MANIFEST_PATH.read_text(encoding="utf-8").splitlines():
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


def _copy_workflows(repo_root: Path, project_dir: Path, options: InstallOptions) -> None:
    workflows_src = repo_root / ".agent" / "workflows"
    workflows_dest = project_dir / ".agent" / "workflows"
    _ensure_dir(workflows_dest, options.dry_run)
    copied = 0
    skipped = 0
    for workflow_file in sorted(workflows_src.glob("*.md")):
        status, _ = _copy_path(
            workflow_file,
            workflows_dest / workflow_file.name,
            options.mode,
            options.overwrite,
            options.dry_run,
        )
        if status == "ok":
            copied += 1
        else:
            skipped += 1
    if copied:
        _print_result("Workflows", f"{workflows_dest}/ ({copied} files)", "ok")
    else:
        _print_result("Workflows", f"{workflows_dest}/ ({skipped} skipped)", "skip")


def _install_project_env(repo_root: Path, project_dir: Path, options: InstallOptions) -> None:
    _copy_display(repo_root / ".env.example", project_dir / ".env", "Env", options)


def _write_gemini_quickstart(repo_root: Path, project_dir: Path, options: InstallOptions) -> None:
    quickstart_dir = project_dir / ".gemini"
    quickstart_path = quickstart_dir / "research-skills.md"
    quickstart_src = repo_root / ".gemini" / "research-skills.md"
    _copy_display(quickstart_src, quickstart_path, "Quickstart", options)


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
    result = subprocess.run(
        [sys.executable, "-m", "bridges.orchestrator", "doctor", "--cwd", str(project_dir)],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
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
        )
    )

    if options.target not in TARGET_CHOICES:
        raise ValueError(f"Unsupported target: {options.target}")
    if options.mode not in {"copy", "link"}:
        raise ValueError(f"Unsupported mode: {options.mode}")
    install_cli = bool(options.install_cli)
    doctor = bool(options.doctor)

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
    if install_cli:
        print(f"  cli:     install -> {options.cli_dir}")

    _print_full_readiness(options)
    antigravity_cli_found = _print_cli_checks(options.target)

    section_targets = ("codex", "claude", "gemini", "antigravity")
    for section_target in section_targets:
        if options.target not in {section_target, "all"}:
            continue
        _print_section(section_target.capitalize() if section_target != "antigravity" else "Antigravity")
        for entry in manifest_entries:
            if entry["target"] != section_target:
                continue
            op = entry["op"]
            label = entry["label"]
            src = repo_root / entry["source"]
            dest = _expand_path(entry["destination"], manifest_values)

            if op == "dir-copy" or op == "file-copy":
                _copy_display(src, dest, label, options)
                continue

            if op == "glob-copy":
                _copy_workflows(repo_root, options.project_dir, options)
                continue

            if op == "claude-template":
                claude_target = dest
                if claude_target.exists() and not options.overwrite:
                    claude_target = options.project_dir / "CLAUDE.research-skills.md"
                _copy_display(src, claude_target, label, options)
                continue

            if op == "quickstart-file":
                _write_gemini_quickstart(repo_root, options.project_dir, options)
                continue

            if op == "conditional-dir-copy":
                if antigravity_cli_found:
                    _copy_display(src, dest, label, options)
                else:
                    _print_result(label, f"{dest} (antigravity CLI not found)", "skip")
                continue

            raise ValueError(f"Unsupported manifest operation: {op}")

    if install_cli:
        _install_shell_cli(options)

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
