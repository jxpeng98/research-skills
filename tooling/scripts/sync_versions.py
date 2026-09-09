#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.source_layout import RepoLayout

try:
    from tooling.scripts.release_version import ReleaseIdentity, parse_release_version
except ModuleNotFoundError:  # Direct execution from tooling/scripts.
    from release_version import ReleaseIdentity, parse_release_version


PRINTABLE_FIELDS = ("package_version", "skill_version", "repo_version", "npm_version")


def parse_version(raw: str) -> tuple[str, str, str, str]:
    """Preserve the historical four-value API over the shared release contract."""

    identity = parse_release_version(raw)
    return (
        identity.package_version,
        identity.skill_version,
        identity.repo_tag,
        identity.npm_version,
    )


def replace_pattern(path: Path, pattern: re.Pattern[str], replacement: str) -> bool:
    original = path.read_text(encoding="utf-8")
    updated, count = pattern.subn(replacement, original)
    if count == 0:
        raise ValueError(f"no matching version field found in {path}")
    if updated != original:
        path.write_text(updated, encoding="utf-8")
        return True
    return False


def replace_json_versions(path: Path, version: str) -> bool:
    original = path.read_text(encoding="utf-8")
    data = json.loads(original)
    changed = False

    def visit(value: object) -> None:
        nonlocal changed
        if isinstance(value, dict):
            for key, item in value.items():
                if key == "version" and item != version:
                    value[key] = version
                    changed = True
                else:
                    visit(item)
        elif isinstance(value, list):
            for item in value:
                visit(item)

    visit(data)
    if changed:
        path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    return changed


def replace_npm_lock_workspace_version(path: Path, version: str) -> bool:
    original = path.read_text(encoding="utf-8")
    data = json.loads(original)
    packages = data.get("packages")
    if not isinstance(packages, dict):
        raise ValueError(f"missing packages object in {path}")
    workspace = packages.get("packages/npm-qiongli")
    if not isinstance(workspace, dict):
        raise ValueError(f"missing packages/npm-qiongli entry in {path}")
    if workspace.get("version") == version:
        return False
    workspace["version"] = version
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    return True


def replace_skill_entrypoint_version(path: Path, repo_version: str) -> bool:
    original = path.read_text(encoding="utf-8")
    label = "Qiongli Next" if re.search(r"(?m)^name:\s*qiongli-next\s*$", original) else "Qiongli"
    updated = original
    description_line = re.compile(r"(?m)^(description:\s*)(.+)$")

    def replace_description(match: re.Match[str]) -> str:
        current = _unquote_yaml_like_string(match.group(2).strip())
        prefix = re.compile(
            r"^(?:Qiongli(?: Next)? version:\s*"
            r"v?\d+\.\d+\.\d+(?:-(?:alpha|beta)\.\d+)?\.\s*)+"
        )
        body = prefix.sub("", current, count=1)
        description = f"{label} version: {repo_version}. {body}"
        return f"{match.group(1)}{json.dumps(description, ensure_ascii=False)}"

    updated, description_count = description_line.subn(replace_description, updated, count=1)
    if description_count == 0:
        raise ValueError(f"no matching description field found in {path}")

    updated, body_count = re.subn(
        r"Installed Qiongli workflow version: `[^`]+`",
        f"Installed Qiongli workflow version: `{repo_version}`",
        updated,
        count=1,
    )
    if body_count == 0:
        updated = updated.rstrip() + f"\n\nInstalled Qiongli workflow version: `{repo_version}`\n"

    if updated != original:
        path.write_text(updated, encoding="utf-8")
        return True
    return False


def _unquote_yaml_like_string(value: str) -> str:
    if len(value) >= 2 and value[0] == value[-1] == '"':
        try:
            decoded = json.loads(value)
        except json.JSONDecodeError:
            return value[1:-1]
        return decoded if isinstance(decoded, str) else value
    if len(value) >= 2 and value[0] == value[-1] == "'":
        return value[1:-1].replace("''", "'")
    return value


def replace_uv_lock_editable_package_version(path: Path, package_name: str, version: str) -> bool:
    original = path.read_text(encoding="utf-8")
    package_block_pattern = re.compile(
        rf'(?ms)^(\[\[package\]\]\nname = "{re.escape(package_name)}"\nversion = ")[^"]+(".*?source = \{{ editable = "\." \}}.*?)(?=^\[\[package\]\]|\Z)'
    )
    updated, count = package_block_pattern.subn(rf"\g<1>{version}\g<2>", original, count=1)
    if count == 0:
        raise ValueError(f"missing editable package {package_name!r} in {path}")
    if updated == original:
        return False
    path.write_text(updated, encoding="utf-8")
    return True


def _replace_toml_section_string(
    content: str, *, section: str, field: str, value: str, path: Path
) -> str:
    section_pattern = re.compile(
        rf"(?ms)^\[{re.escape(section)}\]\s*$\n(?P<body>.*?)(?=^\[|\Z)"
    )
    section_match = section_pattern.search(content)
    if section_match is None:
        raise ValueError(f"missing [{section}] section in {path}")

    body = section_match.group("body")
    field_pattern = re.compile(
        rf'(?m)^(?P<prefix>{re.escape(field)}\s*=\s*)"[^"]*"(?P<suffix>\s*(?:#.*)?)$'
    )
    updated_body, count = field_pattern.subn(
        lambda match: f'{match.group("prefix")}"{value}"{match.group("suffix")}',
        body,
    )
    if count != 1:
        raise ValueError(
            f"expected exactly one {field!r} field in [{section}] in {path}; found {count}"
        )
    return (
        content[: section_match.start("body")]
        + updated_body
        + content[section_match.end("body") :]
    )


def _replace_cargo_lock_package_version(
    content: str, *, package_name: str, version: str, path: Path
) -> str:
    package_pattern = re.compile(r"(?ms)^\[\[package\]\]\s*$\n.*?(?=^\[\[package\]\]\s*$|\Z)")
    matching_blocks = [
        match
        for match in package_pattern.finditer(content)
        if re.search(
            rf'(?m)^name\s*=\s*"{re.escape(package_name)}"\s*$',
            match.group(0),
        )
    ]
    if len(matching_blocks) != 1:
        raise ValueError(
            f"expected exactly one Cargo.lock package named {package_name!r} in {path}; "
            f"found {len(matching_blocks)}"
        )

    block_match = matching_blocks[0]
    updated_block, count = re.subn(
        r'(?m)^(version\s*=\s*)"[^"]*"(\s*)$',
        rf'\g<1>"{version}"\g<2>',
        block_match.group(0),
        count=1,
    )
    if count != 1:
        raise ValueError(f"missing version field for package {package_name!r} in {path}")
    return content[: block_match.start()] + updated_block + content[block_match.end() :]


def _native_workspace_package_names(manifest: dict[str, object], native_root: Path) -> tuple[str, ...]:
    workspace = manifest.get("workspace")
    if not isinstance(workspace, dict):
        raise ValueError(f"missing [workspace] section in {native_root / 'Cargo.toml'}")
    members = workspace.get("members")
    if members is None:
        return ("qiongli",)
    if not isinstance(members, list) or not members:
        raise ValueError(f"invalid workspace members in {native_root / 'Cargo.toml'}")

    names: list[str] = []
    for member in members:
        if not isinstance(member, str) or "*" in member:
            raise ValueError(f"workspace members must be explicit paths: {member!r}")
        member_manifest = native_root / member / "Cargo.toml"
        try:
            with member_manifest.open("rb") as handle:
                member_data = tomllib.load(handle)
        except (OSError, tomllib.TOMLDecodeError) as exc:
            raise ValueError(f"unable to read workspace member {member_manifest}: {exc}") from exc
        package = member_data.get("package")
        name = package.get("name") if isinstance(package, dict) else None
        if not isinstance(name, str) or not name:
            raise ValueError(f"missing package.name in workspace member {member_manifest}")
        if package.get("version") != {"workspace": True}:
            raise ValueError(f"workspace member must inherit version: {member_manifest}")
        names.append(name)
    if len(set(names)) != len(names):
        raise ValueError("native workspace contains duplicate package names")
    return tuple(sorted(names))


def _replace_native_content_versions(root: Path, identity: ReleaseIdentity) -> list[Path]:
    changed: list[Path] = []
    content_root = root / "content"
    for manifest in (
        content_root / ".codex-plugin" / "plugin.json",
        content_root / ".claude-plugin" / "plugin.json",
        root / "packages" / "qiongli-full-mcpb" / "manifest.json",
    ):
        if manifest.exists() and replace_json_versions(manifest, identity.version):
            changed.append(manifest)

    registry = content_root / "skills" / "registry.yaml"
    if registry.exists() and replace_pattern(
        registry,
        re.compile(r'^(\s*version:\s*)"?[^"\n]+"?$', re.MULTILINE),
        rf'\g<1>"{identity.version}"',
    ):
        changed.append(registry)

    workflow_version = content_root / "workflow" / "VERSION"
    if workflow_version.exists() and workflow_version.read_text(encoding="utf-8").strip() != identity.repo_tag:
        workflow_version.write_text(identity.repo_tag + "\n", encoding="utf-8")
        changed.append(workflow_version)

    workflow_skill = content_root / "workflow" / "SKILL.md"
    if workflow_skill.exists() and replace_skill_entrypoint_version(workflow_skill, identity.repo_tag):
        changed.append(workflow_skill)
    return changed


def _sync_native_versions(root: Path, identity: ReleaseIdentity) -> list[Path]:
    native_root = root / "packages" / "qiongli-native"
    manifest = native_root / "Cargo.toml"
    lockfile = native_root / "Cargo.lock"
    lite_lockfile = root / "packages" / "qiongli-lite-mcp" / "Cargo.lock"

    try:
        with manifest.open("rb") as handle:
            parsed_manifest = tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise ValueError(f"unable to read native workspace manifest {manifest}: {exc}") from exc
    workspace_package_names = _native_workspace_package_names(parsed_manifest, native_root)

    manifest_original = manifest.read_text(encoding="utf-8")
    manifest_updated = _replace_toml_section_string(
        manifest_original,
        section="workspace.package",
        field="version",
        value=identity.version,
        path=manifest,
    )
    manifest_updated = _replace_toml_section_string(
        manifest_updated,
        section="workspace.metadata.qiongli",
        field="channel",
        value=identity.channel,
        path=manifest,
    )
    lock_updates: list[tuple[Path, str, str]] = []
    lock_original = lockfile.read_text(encoding="utf-8")
    lock_updated = lock_original
    for package_name in workspace_package_names:
        lock_updated = _replace_cargo_lock_package_version(
            lock_updated,
            package_name=package_name,
            version=identity.version,
            path=lockfile,
        )
    lock_updates.append((lockfile, lock_original, lock_updated))

    if lite_lockfile.exists():
        lite_lock_original = lite_lockfile.read_text(encoding="utf-8")
        lite_lock_updated = lite_lock_original
        for package_name in workspace_package_names:
            if re.search(
                rf'(?m)^name\s*=\s*"{re.escape(package_name)}"\s*$',
                lite_lock_original,
            ):
                lite_lock_updated = _replace_cargo_lock_package_version(
                    lite_lock_updated,
                    package_name=package_name,
                    version=identity.version,
                    path=lite_lockfile,
                )
        lock_updates.append((lite_lockfile, lite_lock_original, lite_lock_updated))

    changed: list[Path] = []
    if manifest_updated != manifest_original:
        manifest.write_text(manifest_updated, encoding="utf-8")
        changed.append(manifest)
    for path, original, updated in lock_updates:
        if updated != original:
            path.write_text(updated, encoding="utf-8")
            changed.append(path)
    changed.extend(_replace_native_content_versions(root, identity))
    return changed


def sync_versions(root: Path, raw_version: str) -> list[Path]:
    root = root.resolve()
    identity = parse_release_version(raw_version)
    if identity.release_line == "native-2x":
        return _sync_native_versions(root, identity)

    package_version = identity.package_version
    skill_version = identity.skill_version
    repo_version = identity.repo_tag
    npm_version = identity.npm_version
    changed: list[Path] = []
    layout = RepoLayout(root)

    replacements: list[tuple[Path, re.Pattern[str], str]] = [
        (
            root / "pyproject.toml",
            re.compile(r'^version = "[^"]+"$', re.MULTILINE),
            f'version = "{package_version}"',
        ),
        (
            layout.python_package / "__init__.py",
            re.compile(r'^__version__ = "[^"]+"$', re.MULTILINE),
            f'__version__ = "{package_version}"',
        ),
    ]

    for path, pattern, replacement in replacements:
        if replace_pattern(path, pattern, replacement):
            changed.append(path)

    registry_files = (
        layout.skills / "registry.yaml",
        root / "qiongli-workflow" / "skills" / "registry.yaml",
        root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow" / "skills" / "registry.yaml",
        root / "packages" / "npm-qiongli" / "python-runtime" / "skills" / "registry.yaml",
    )
    for registry_file in registry_files:
        if not registry_file.exists():
            continue
        if replace_pattern(
            registry_file,
            re.compile(r'^(\s*version:\s*)"?[^"\n]+"?$', re.MULTILINE),
            rf'\g<1>"{skill_version}"',
        ):
            changed.append(registry_file)

    workflow_version_files = (
        layout.workflow / "VERSION",
        root / "qiongli-workflow" / "VERSION",
        root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow" / "VERSION",
    )
    for workflow_version_file in workflow_version_files:
        if not workflow_version_file.exists():
            continue
        original_repo_version = workflow_version_file.read_text(encoding="utf-8").strip()
        if original_repo_version != repo_version:
            workflow_version_file.write_text(repo_version + "\n", encoding="utf-8")
            changed.append(workflow_version_file)

    skill_entrypoint_files = (
        layout.workflow / "SKILL.md",
        root / "qiongli-workflow" / "SKILL.md",
        root / "packages" / "npm-qiongli" / "payload" / "qiongli-workflow" / "SKILL.md",
    )
    for skill_entrypoint in skill_entrypoint_files:
        if not skill_entrypoint.exists():
            continue
        if replace_skill_entrypoint_version(skill_entrypoint, repo_version):
            changed.append(skill_entrypoint)

    npm_manifest = root / "packages" / "npm-qiongli" / "package.json"
    if npm_manifest.exists():
        if replace_json_versions(npm_manifest, npm_version):
            changed.append(npm_manifest)

    npm_lock = root / "package-lock.json"
    if npm_lock.exists():
        if replace_npm_lock_workspace_version(npm_lock, npm_version):
            changed.append(npm_lock)

    uv_lock = root / "uv.lock"
    if uv_lock.exists():
        if replace_uv_lock_editable_package_version(uv_lock, "qiongli", package_version):
            changed.append(uv_lock)

    bundled_python_init_files = (
        root / "packages" / "npm-qiongli" / "python-runtime" / "qiongli" / "__init__.py",
    )
    for bundled_python_init in bundled_python_init_files:
        if not bundled_python_init.exists():
            continue
        if replace_pattern(
            bundled_python_init,
            re.compile(r'^__version__ = "[^"]+"$', re.MULTILINE),
            f'__version__ = "{package_version}"',
        ):
            changed.append(bundled_python_init)

    return changed


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Sync the version sources owned by one Qiongli release line."
    )
    parser.add_argument(
        "version",
        help="Stable, beta, or native alpha version, e.g. 1.19.1b1 or 2.0.0-alpha.3",
    )
    parser.add_argument(
        "--print-field",
        choices=PRINTABLE_FIELDS,
        help="Print one normalized version field and exit without writing files.",
    )
    parser.add_argument(
        "--root",
        default=Path(__file__).resolve().parents[2],
        type=Path,
        help="Repository root (defaults to current repo root)",
    )
    args = parser.parse_args(argv)

    package_version, skill_version, repo_version, npm_version = parse_version(args.version)
    if args.print_field:
        print(
            {
                "package_version": package_version,
                "skill_version": skill_version,
                "repo_version": repo_version,
                "npm_version": npm_version,
            }[args.print_field]
        )
        return 0

    root = args.root.resolve()
    changed = sync_versions(root, args.version)

    print("[sync-versions] normalized versions")
    print(f"  - package_version: {package_version}")
    print(f"  - skill_version:   {skill_version}")
    print(f"  - repo_version:    {repo_version}")
    print(f"  - npm_version:     {npm_version}")
    print(f"  - changed_files:   {len(changed)}")
    for path in changed:
        print(f"    - {path.relative_to(root)}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ValueError as exc:
        print(f"[sync-versions] {exc}", file=sys.stderr)
        raise SystemExit(2)
