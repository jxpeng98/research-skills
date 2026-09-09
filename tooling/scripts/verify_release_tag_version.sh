#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
TAG=""

usage() {
  cat <<'EOF'
Usage:
  ./scripts/verify_release_tag_version.sh [--root <dir>] --tag <tag>

Description:
  Verify that a Git release tag matches the package and workflow versions in the repo.

Options:
  --root <dir>  Repository root to verify. Defaults to this script's checkout.
  --tag <tag>   Required release tag.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --root)
      [[ $# -ge 2 ]] || { echo "[verify-release-tag] missing value for --root" >&2; exit 2; }
      ROOT_DIR="$(cd "$2" && pwd)"
      shift 2
      ;;
    --tag)
      [[ $# -ge 2 ]] || { echo "[verify-release-tag] missing value for --tag" >&2; exit 2; }
      TAG="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[verify-release-tag] unknown option: $1" >&2
      usage
      exit 2
      ;;
  esac
done

[[ -n "$TAG" ]] || { echo "[verify-release-tag] --tag is required" >&2; usage; exit 2; }

cd "$ROOT_DIR"

release_field() {
  local field="$1"
  "${QIONGLI_PYTHON:-python3}" scripts/release_version.py "$TAG" --print-field "$field"
}

expected_repo_tag="$(release_field repo_version)"
expected_package_version="$(release_field package_version)"
expected_release_line="$(release_field release_line)"
expected_channel="$(release_field channel)"

if [[ "$expected_repo_tag" != "$TAG" ]]; then
  echo "[verify-release-tag] normalized tag mismatch: expected $expected_repo_tag from input $TAG" >&2
  exit 1
fi

if [[ "$expected_release_line" == "native-2x" ]]; then
  expected_native_version="${expected_repo_tag#v}"
  "${QIONGLI_PYTHON:-python3}" - "$expected_native_version" "$expected_channel" <<'PY'
import json
from pathlib import Path
import re
import sys
import tomllib

expected_version, expected_channel = sys.argv[1:]
manifest_path = Path("packages/qiongli-native/Cargo.toml")
lock_path = Path("packages/qiongli-native/Cargo.lock")
lite_lock_path = Path("packages/qiongli-lite-mcp/Cargo.lock")

if not manifest_path.is_file():
    raise SystemExit(f"[verify-release-tag] missing native product manifest: {manifest_path}")
if not lock_path.is_file():
    raise SystemExit(f"[verify-release-tag] missing native lockfile: {lock_path}")
if not lite_lock_path.is_file():
    raise SystemExit(f"[verify-release-tag] missing Lite lockfile: {lite_lock_path}")

with manifest_path.open("rb") as handle:
    manifest = tomllib.load(handle)
workspace = manifest.get("workspace")
if not isinstance(workspace, dict):
    raise SystemExit("[verify-release-tag] native Cargo.toml must define [workspace]")
package = workspace.get("package")
if not isinstance(package, dict):
    raise SystemExit("[verify-release-tag] native Cargo.toml must define [workspace.package]")
actual_version = package.get("version")
if actual_version != expected_version:
    raise SystemExit(
        "[verify-release-tag] native workspace version mismatch: "
        f"tag expects {expected_version}, found {actual_version}"
    )

metadata = workspace.get("metadata")
qiongli = metadata.get("qiongli") if isinstance(metadata, dict) else None
if not isinstance(qiongli, dict) or qiongli.get("product") != "qiongli":
    raise SystemExit("[verify-release-tag] native workspace metadata must identify product qiongli")
actual_channel = qiongli.get("channel")
if actual_channel != expected_channel:
    raise SystemExit(
        "[verify-release-tag] native workspace channel mismatch: "
        f"version expects {expected_channel}, found {actual_channel}"
    )

members = workspace.get("members")
if not isinstance(members, list) or not members:
    raise SystemExit("[verify-release-tag] native Cargo.toml must define explicit workspace members")
workspace_names = []
native_root = manifest_path.parent
for member in members:
    if not isinstance(member, str) or "*" in member:
        raise SystemExit("[verify-release-tag] native workspace members must use explicit paths")
    member_path = native_root / member / "Cargo.toml"
    with member_path.open("rb") as handle:
        member_manifest = tomllib.load(handle)
    member_package = member_manifest.get("package")
    member_name = member_package.get("name") if isinstance(member_package, dict) else None
    if not isinstance(member_name, str) or not member_name:
        raise SystemExit(f"[verify-release-tag] missing package.name in {member_path}")
    if member_package.get("version") != {"workspace": True}:
        raise SystemExit(f"[verify-release-tag] workspace version inheritance missing in {member_path}")
    workspace_names.append(member_name)
if len(set(workspace_names)) != len(workspace_names):
    raise SystemExit("[verify-release-tag] native workspace package names are not unique")

for checked_lock_path, label, require_all in (
    (lock_path, "native", True),
    (lite_lock_path, "Lite", False),
):
    with checked_lock_path.open("rb") as handle:
        lock = tomllib.load(handle)
    locked_packages = lock.get("package", [])
    package_names = workspace_names if require_all else [
        name
        for name in workspace_names
        if any(
            isinstance(package, dict) and package.get("name") == name
            for package in locked_packages
        )
    ]
    if not package_names:
        raise SystemExit(f"[verify-release-tag] {label} Cargo.lock has no native packages")
    for package_name in package_names:
        matches = [
            package
            for package in locked_packages
            if isinstance(package, dict) and package.get("name") == package_name
        ]
        if len(matches) != 1:
            raise SystemExit(
                f"[verify-release-tag] {label} Cargo.lock must contain exactly one "
                f"{package_name} package"
            )
        locked_version = matches[0].get("version")
        if locked_version != expected_version:
            raise SystemExit(
                f"[verify-release-tag] {label} Cargo.lock version mismatch for {package_name}: "
                f"tag expects {expected_version}, found {locked_version}"
            )

for plugin_path in (
    Path("content/.codex-plugin/plugin.json"),
    Path("content/.claude-plugin/plugin.json"),
):
    try:
        plugin = json.loads(plugin_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise SystemExit(f"[verify-release-tag] invalid native plugin manifest {plugin_path}: {exc}")
    if plugin.get("version") != expected_version:
        raise SystemExit(
            f"[verify-release-tag] native plugin version mismatch in {plugin_path}: "
            f"tag expects {expected_version}, found {plugin.get('version')}"
        )

registry_path = Path("content/skills/registry.yaml")
registry_versions = set(
    re.findall(
        r'^\s*version:\s*"?([^"\n]+)"?$',
        registry_path.read_text(encoding="utf-8"),
        re.MULTILINE,
    )
)
if registry_versions != {expected_version}:
    raise SystemExit(
        "[verify-release-tag] native skill registry version mismatch: "
        f"tag expects {expected_version}, found {sorted(registry_versions)}"
    )

workflow_version_path = Path("content/workflow/VERSION")
workflow_version = workflow_version_path.read_text(encoding="utf-8").strip()
if workflow_version != f"v{expected_version}":
    raise SystemExit(
        "[verify-release-tag] native workflow version mismatch: "
        f"tag expects v{expected_version}, found {workflow_version}"
    )
workflow_skill = Path("content/workflow/SKILL.md").read_text(encoding="utf-8")
if f"Qiongli version: v{expected_version}." not in workflow_skill or (
    f"Installed Qiongli workflow version: `v{expected_version}`" not in workflow_skill
):
    raise SystemExit("[verify-release-tag] native workflow skill version mismatch")

content_lock_path = Path(
    "packages/qiongli-native/crates/qiongli-content/resources/qiongli-core.lock.json"
)
try:
    content_lock = json.loads(content_lock_path.read_text(encoding="utf-8"))
except (OSError, json.JSONDecodeError) as exc:
    raise SystemExit(f"[verify-release-tag] invalid embedded content lock: {exc}")
if content_lock.get("content_version") != expected_version:
    raise SystemExit(
        "[verify-release-tag] embedded content version mismatch: "
        f"tag expects {expected_version}, found {content_lock.get('content_version')}"
    )

release_note_path = Path("tooling/release") / f"v{expected_version}.md"
if not release_note_path.is_file():
    raise SystemExit(f"[verify-release-tag] missing native release notes: {release_note_path}")
release_note = release_note_path.read_text(encoding="utf-8")
if f"# Qiongli v{expected_version}" not in release_note:
    raise SystemExit(f"[verify-release-tag] native release note version mismatch: {release_note_path}")

version_driven_sources = {
    Path("packages/qiongli-native/apps/qiongli/examples/native_candidate_acceptance.rs"):
        'env!("CARGO_PKG_VERSION")',
    Path("packages/qiongli-native/apps/qiongli/examples/native_community_alpha_promotion.rs"):
        'env!("CARGO_PKG_VERSION")',
    Path("packages/qiongli-native/apps/qiongli/examples/native_community_alpha_release.rs"):
        'env!("CARGO_PKG_VERSION")',
    Path("packages/qiongli-native/apps/qiongli/examples/native_update_metadata.rs"):
        'env!("CARGO_PKG_VERSION")',
}
for source_path, marker in version_driven_sources.items():
    source = source_path.read_text(encoding="utf-8")
    if marker not in source:
        raise SystemExit(f"[verify-release-tag] release source is not version-driven: {source_path}")
PY
  echo "[verify-release-tag] native tag, workspace version, channel, and Cargo.lock are aligned; plugins, content, and release notes also agree: $TAG"
  exit 0
fi

if [[ "$expected_release_line" != "legacy-1x" ]]; then
  echo "[verify-release-tag] unsupported release line: $expected_release_line" >&2
  exit 2
fi

expected_skill_version="${expected_repo_tag#v}"
expected_npm_version="$expected_skill_version"

actual_package_version="$("${QIONGLI_PYTHON:-python3}" - <<'PY'
import re
from pathlib import Path

content = Path("pyproject.toml").read_text(encoding="utf-8")
match = re.search(r'^version = "([^"]+)"$', content, re.MULTILINE)
if not match:
    raise SystemExit("missing version in pyproject.toml")
print(match.group(1))
PY
)"

actual_init_version="$("${QIONGLI_PYTHON:-python3}" - <<'PY'
import re
from pathlib import Path

content = Path("packages/python-qiongli/src/qiongli/__init__.py").read_text(encoding="utf-8")
match = re.search(r'^__version__ = "([^"]+)"$', content, re.MULTILINE)
if not match:
    raise SystemExit("missing __version__ in packages/python-qiongli/src/qiongli/__init__.py")
print(match.group(1))
PY
)"

actual_skill_version="$("${QIONGLI_PYTHON:-python3}" - <<'PY'
import re
from pathlib import Path

content = Path("content/skills/registry.yaml").read_text(encoding="utf-8")
versions = set(re.findall(r'^\s*version:\s*"?([^"\n]+)"?$', content, re.MULTILINE))
if not versions:
    raise SystemExit("missing version in content/skills/registry.yaml")
if len(versions) != 1:
    raise SystemExit(f"mixed versions in content/skills/registry.yaml: {sorted(versions)}")
print(versions.pop())
PY
)"

actual_workflow_version="$(tr -d '\r\n' < content/workflow/VERSION)"
actual_python_payload_workflow_version="$(tr -d '\r\n' < packages/python-qiongli/src/qiongli/payload/qiongli-workflow/VERSION)"
actual_python_payload_workflow_registry_version="$("${QIONGLI_PYTHON:-python3}" - <<'PY'
import re
from pathlib import Path

path = Path("packages/python-qiongli/src/qiongli/payload/qiongli-workflow/skills/registry.yaml")
if not path.exists():
    raise SystemExit("missing packages/python-qiongli/src/qiongli/payload/qiongli-workflow/skills/registry.yaml")
content = path.read_text(encoding="utf-8")
versions = set(re.findall(r'^\s*version:\s*"?([^"\n]+)"?$', content, re.MULTILINE))
if not versions:
    raise SystemExit(f"missing version in {path}")
if len(versions) != 1:
    raise SystemExit(f"mixed versions in {path}: {sorted(versions)}")
print(versions.pop())
PY
)"
actual_python_payload_registry_version="$("${QIONGLI_PYTHON:-python3}" - <<'PY'
import re
from pathlib import Path

path = Path("packages/python-qiongli/src/qiongli/payload/skills/registry.yaml")
if not path.exists():
    raise SystemExit("missing packages/python-qiongli/src/qiongli/payload/skills/registry.yaml")
content = path.read_text(encoding="utf-8")
versions = set(re.findall(r'^\s*version:\s*"?([^"\n]+)"?$', content, re.MULTILINE))
if not versions:
    raise SystemExit(f"missing version in {path}")
if len(versions) != 1:
    raise SystemExit(f"mixed versions in {path}: {sorted(versions)}")
print(versions.pop())
PY
)"
actual_bundled_workflow_version="$(tr -d '\r\n' < packages/npm-qiongli/payload/qiongli-workflow/VERSION)"
actual_bundled_workflow_registry_version="$("${QIONGLI_PYTHON:-python3}" - <<'PY'
import re
from pathlib import Path

path = Path("packages/npm-qiongli/payload/qiongli-workflow/skills/registry.yaml")
if not path.exists():
    raise SystemExit("missing packages/npm-qiongli/payload/qiongli-workflow/skills/registry.yaml")
content = path.read_text(encoding="utf-8")
versions = set(re.findall(r'^\s*version:\s*"?([^"\n]+)"?$', content, re.MULTILINE))
if not versions:
    raise SystemExit(f"missing version in {path}")
if len(versions) != 1:
    raise SystemExit(f"mixed versions in {path}: {sorted(versions)}")
print(versions.pop())
PY
)"
actual_plugin_workflow_version="$(tr -d '\r\n' < plugins/qiongli/skills/qiongli-workflow/VERSION)"
actual_plugin_workflow_registry_version="$("${QIONGLI_PYTHON:-python3}" - <<'PY'
import re
from pathlib import Path

path = Path("plugins/qiongli/skills/qiongli-workflow/skills/registry.yaml")
if not path.exists():
    raise SystemExit("missing plugins/qiongli/skills/qiongli-workflow/skills/registry.yaml")
content = path.read_text(encoding="utf-8")
versions = set(re.findall(r'^\s*version:\s*"?([^"\n]+)"?$', content, re.MULTILINE))
if not versions:
    raise SystemExit(f"missing version in {path}")
if len(versions) != 1:
    raise SystemExit(f"mixed versions in {path}: {sorted(versions)}")
print(versions.pop())
PY
)"
actual_next_plugin_workflow_version="$(tr -d '\r\n' < plugins/qiongli-next/skills/qiongli-workflow/VERSION)"
actual_next_plugin_workflow_registry_version="$("${QIONGLI_PYTHON:-python3}" - <<'PY'
import re
from pathlib import Path

path = Path("plugins/qiongli-next/skills/qiongli-workflow/skills/registry.yaml")
if not path.exists():
    raise SystemExit("missing plugins/qiongli-next/skills/qiongli-workflow/skills/registry.yaml")
content = path.read_text(encoding="utf-8")
versions = set(re.findall(r'^\s*version:\s*"?([^"\n]+)"?$', content, re.MULTILINE))
if not versions:
    raise SystemExit(f"missing version in {path}")
if len(versions) != 1:
    raise SystemExit(f"mixed versions in {path}: {sorted(versions)}")
print(versions.pop())
PY
)"

actual_npm_version="$("${QIONGLI_PYTHON:-python3}" - <<'PY'
import json
from pathlib import Path

path = Path("packages/npm-qiongli/package.json")
if not path.exists():
    raise SystemExit("missing packages/npm-qiongli/package.json")
print(json.loads(path.read_text(encoding="utf-8"))["version"])
PY
)"

actual_npm_lock_version="$("${QIONGLI_PYTHON:-python3}" - <<'PY'
import json
from pathlib import Path

path = Path("package-lock.json")
if not path.exists():
    raise SystemExit("missing package-lock.json")
data = json.loads(path.read_text(encoding="utf-8"))
try:
    print(data["packages"]["packages/npm-qiongli"]["version"])
except KeyError as exc:
    raise SystemExit(f"missing package-lock workspace version: {exc}") from exc
PY
)"

actual_bundled_init_version="$("${QIONGLI_PYTHON:-python3}" - <<'PY'
import re
from pathlib import Path

path = Path("packages/npm-qiongli/python-runtime/qiongli/__init__.py")
if not path.exists():
    raise SystemExit("missing packages/npm-qiongli/python-runtime/qiongli/__init__.py")
content = path.read_text(encoding="utf-8")
match = re.search(r'^__version__ = "([^"]+)"$', content, re.MULTILINE)
if not match:
    raise SystemExit("missing __version__ in packages/npm-qiongli/python-runtime/qiongli/__init__.py")
print(match.group(1))
PY
)"

actual_bundled_registry_version="$("${QIONGLI_PYTHON:-python3}" - <<'PY'
import re
from pathlib import Path

path = Path("packages/npm-qiongli/python-runtime/skills/registry.yaml")
if not path.exists():
    raise SystemExit("missing packages/npm-qiongli/python-runtime/skills/registry.yaml")
content = path.read_text(encoding="utf-8")
versions = set(re.findall(r'^\s*version:\s*"?([^"\n]+)"?$', content, re.MULTILINE))
if not versions:
    raise SystemExit("missing version in packages/npm-qiongli/python-runtime/skills/registry.yaml")
if len(versions) != 1:
    raise SystemExit(f"mixed versions in {path}: {sorted(versions)}")
print(versions.pop())
PY
)"

actual_plugin_versions="$("${QIONGLI_PYTHON:-python3}" - <<'PY'
import json
from pathlib import Path

paths = [
    Path("plugins/qiongli/.codex-plugin/plugin.json"),
    Path("plugins/qiongli/.claude-plugin/plugin.json"),
    Path("plugins/qiongli-next/.codex-plugin/plugin.json"),
    Path("plugins/qiongli-next/.claude-plugin/plugin.json"),
]
for path in paths:
    data = json.loads(path.read_text(encoding="utf-8"))
    versions = []

    def visit(value):
        if isinstance(value, dict):
            for key, item in value.items():
                if key == "version":
                    versions.append(item)
                else:
                    visit(item)
        elif isinstance(value, list):
            for item in value:
                visit(item)

    visit(data)
    if not versions:
        raise SystemExit(f"missing version in {path}")
    print(f"{path}:{','.join(versions)}")
PY
)"

[[ "$actual_package_version" == "$expected_package_version" ]] || {
  echo "[verify-release-tag] pyproject version mismatch: tag=$TAG expects $expected_package_version, found $actual_package_version" >&2
  exit 1
}

[[ "$actual_init_version" == "$expected_package_version" ]] || {
  echo "[verify-release-tag] packages/python-qiongli/src/qiongli/__init__.py mismatch: tag=$TAG expects $expected_package_version, found $actual_init_version" >&2
  exit 1
}

[[ "$actual_skill_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] content/skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_skill_version" >&2
  exit 1
}

[[ "$actual_workflow_version" == "$expected_repo_tag" ]] || {
  echo "[verify-release-tag] content/workflow/VERSION mismatch: tag=$TAG expects $expected_repo_tag, found $actual_workflow_version" >&2
  exit 1
}

[[ "$actual_python_payload_workflow_version" == "$expected_repo_tag" ]] || {
  echo "[verify-release-tag] packages/python-qiongli/src/qiongli/payload/qiongli-workflow/VERSION mismatch: tag=$TAG expects $expected_repo_tag, found $actual_python_payload_workflow_version" >&2
  exit 1
}

[[ "$actual_python_payload_workflow_registry_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] packages/python-qiongli/src/qiongli/payload/qiongli-workflow/skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_python_payload_workflow_registry_version" >&2
  exit 1
}

[[ "$actual_python_payload_registry_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] packages/python-qiongli/src/qiongli/payload/skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_python_payload_registry_version" >&2
  exit 1
}

[[ "$actual_bundled_workflow_version" == "$expected_repo_tag" ]] || {
  echo "[verify-release-tag] packages/npm-qiongli/payload/qiongli-workflow/VERSION mismatch: tag=$TAG expects $expected_repo_tag, found $actual_bundled_workflow_version" >&2
  exit 1
}

[[ "$actual_bundled_workflow_registry_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] packages/npm-qiongli/payload/qiongli-workflow/skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_bundled_workflow_registry_version" >&2
  exit 1
}

[[ "$actual_plugin_workflow_version" == "$expected_repo_tag" ]] || {
  echo "[verify-release-tag] plugins/qiongli/skills/qiongli-workflow/VERSION mismatch: tag=$TAG expects $expected_repo_tag, found $actual_plugin_workflow_version" >&2
  exit 1
}

[[ "$actual_plugin_workflow_registry_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] plugins/qiongli/skills/qiongli-workflow/skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_plugin_workflow_registry_version" >&2
  exit 1
}

[[ "$actual_next_plugin_workflow_version" == "$expected_repo_tag" ]] || {
  echo "[verify-release-tag] plugins/qiongli-next/skills/qiongli-workflow/VERSION mismatch: tag=$TAG expects $expected_repo_tag, found $actual_next_plugin_workflow_version" >&2
  exit 1
}

[[ "$actual_next_plugin_workflow_registry_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] plugins/qiongli-next/skills/qiongli-workflow/skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_next_plugin_workflow_registry_version" >&2
  exit 1
}

[[ "$actual_npm_version" == "$expected_npm_version" ]] || {
  echo "[verify-release-tag] packages/npm-qiongli/package.json mismatch: tag=$TAG expects $expected_npm_version, found $actual_npm_version" >&2
  exit 1
}

[[ "$actual_npm_lock_version" == "$expected_npm_version" ]] || {
  echo "[verify-release-tag] package-lock.json mismatch: tag=$TAG expects $expected_npm_version, found $actual_npm_lock_version" >&2
  exit 1
}

[[ "$actual_bundled_init_version" == "$expected_package_version" ]] || {
  echo "[verify-release-tag] packages/npm-qiongli/python-runtime/qiongli/__init__.py mismatch: tag=$TAG expects $expected_package_version, found $actual_bundled_init_version" >&2
  exit 1
}

[[ "$actual_bundled_registry_version" == "$expected_skill_version" ]] || {
  echo "[verify-release-tag] packages/npm-qiongli/python-runtime/skills/registry.yaml mismatch: tag=$TAG expects $expected_skill_version, found $actual_bundled_registry_version" >&2
  exit 1
}

while IFS= read -r plugin_line; do
  [[ -n "$plugin_line" ]] || continue
  plugin_path="${plugin_line%%:*}"
  plugin_versions="${plugin_line#*:}"
  IFS=',' read -r -a version_items <<< "$plugin_versions"
  for actual_plugin_version in "${version_items[@]}"; do
    [[ "$actual_plugin_version" == "$expected_skill_version" ]] || {
      echo "[verify-release-tag] ${plugin_path} mismatch: tag=$TAG expects $expected_skill_version, found $actual_plugin_version" >&2
      exit 1
    }
  done
done <<< "$actual_plugin_versions"

"${QIONGLI_PYTHON:-python3}" scripts/audit_distribution_payloads.py --root "$ROOT_DIR"

echo "[verify-release-tag] tag and repo versions are aligned: $TAG"
