#!/usr/bin/env python3
"""Stage native Cargo sources and macOS ARM64 npm/PyPI packages; never publish."""
from __future__ import annotations

import argparse
import base64
import csv
import hashlib
import io
import json
from pathlib import Path
import re
import shutil
import stat
import subprocess
import tarfile
import tomllib
import zipfile

try:
    from .release_version import parse_release_version
except ImportError:
    from release_version import parse_release_version

ROOT = Path(__file__).resolve().parents[2]
NATIVE = ROOT / 'packages/qiongli-native'


def regular_bytes(path: Path) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f'expected regular file: {path}')
    return path.read_bytes()


def copy_tree(source: Path, destination: Path) -> None:
    for path in sorted(source.rglob('*')):
        if path.is_symlink():
            raise ValueError(f'symlinks are not package inputs: {path}')
        if path.is_file():
            target = destination / path.relative_to(source)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(regular_bytes(path))


def stage_cargo(out: Path, version: str) -> Path:
    """Cargo owns archive normalization and dependency ordering after staging."""
    workspace = out / 'cargo-source'
    workspace.mkdir()
    manifest = regular_bytes(NATIVE / 'Cargo.toml').decode()
    (workspace / 'Cargo.toml').write_text(manifest.replace('publish = false', 'publish = ["crates-io"]'))
    shutil.copyfile(NATIVE / 'Cargo.lock', workspace / 'Cargo.lock')
    members = tomllib.loads(manifest)['workspace']['members']
    for member in members:
        source, dest = NATIVE / member, workspace / member
        dest.mkdir(parents=True)
        text = regular_bytes(source / 'Cargo.toml').decode()
        text = re.sub(r'(\{ path = "[^"]+")', rf'\1, version = "={version}"', text)
        # Only the CLI is a registry product. Repository-only tests/examples stay
        # in the checkout; their fixtures are not dependencies of cargo install.
        text = text.replace('[package]\n', '[package]\nautoexamples = false\nautotests = false\nautobenches = false\ninclude = ["Cargo.toml", "src/**", "build.rs", "resources/**", "schemas/**", "icons/**", "package-assets/**", "LICENSE", "README.md"]\n', 1)
        (dest / 'Cargo.toml').write_text(text)
        for directory in ('src', 'resources', 'schemas', 'icons'):
            if (source / directory).is_dir():
                copy_tree(source / directory, dest / directory)
        if (source / 'build.rs').exists():
            shutil.copyfile(source / 'build.rs', dest / 'build.rs')
        shutil.copyfile(ROOT / 'LICENSE', dest / 'LICENSE')
        (dest / 'README.md').write_text(f'# {source.name}\n\nQiongli {version} native CLI source. See https://github.com/jxpeng98/qiongli.\n\nCargo installation builds the CLI from source; desktop packaging is maintained separately.\n')
    assets = workspace / 'apps/qiongli/package-assets'
    copy_tree(ROOT / 'content', assets / 'content')
    # Keep the Rust composer's declared source list as the authority.
    contract = (NATIVE / 'crates/qiongli-platform/src/zotero_companion.rs').read_text()
    paths = re.search(r'ZOTERO_COMPANION_SOURCE_PATHS:.*?= \[(.*?)\];', contract, re.S)
    if paths is None:
        raise ValueError('Zotero source manifest unavailable')
    for relative in re.findall(r'"([^"]+)"', paths[1]):
        target = assets / 'qiongli-zotero-companion' / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(regular_bytes(ROOT / 'packages/qiongli-zotero-companion' / relative))
    return workspace


NPM_LAUNCHER = '''#!/usr/bin/env node
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
if (process.platform !== 'darwin' || process.arch !== 'arm64') {
  console.error('This Qiongli package supports macOS Apple Silicon only.');
  process.exit(1);
}
const child = spawn(fileURLToPath(new URL('../native/qiongli', import.meta.url)), process.argv.slice(2), { stdio: 'inherit' });
for (const signal of ['SIGINT', 'SIGTERM', 'SIGHUP']) process.on(signal, () => child.kill(signal));
child.on('error', () => { console.error('Unable to start the packaged Qiongli executable.'); process.exitCode = 1; });
child.on('close', (code, signal) => {
  if (signal) { process.removeAllListeners(signal); process.kill(process.pid, signal); }
  else process.exitCode = code ?? 1;
});
'''
PYTHON_LAUNCHER = '''import os
from pathlib import Path
import sys


def main():
    executable = Path(__file__).parent / "bin" / "qiongli"
    os.execv(str(executable), [str(executable), *sys.argv[1:]])
'''


def wheel(out: Path, version: str, platform_tag: str, binary: bytes, readme: str) -> Path:
    dist = f'qiongli-{version}.dist-info'
    files = {
        'qiongli_native/__init__.py': PYTHON_LAUNCHER.encode(),
        'qiongli_native/bin/qiongli': binary,
        f'{dist}/METADATA': (f'Metadata-Version: 2.4\nName: qiongli\nVersion: {version}\nSummary: Native academic research CLI\nRequires-Python: >=3.9\nLicense-Expression: MIT\nLicense-File: LICENSE\nDescription-Content-Type: text/markdown\n\n{readme}').encode(),
        f'{dist}/WHEEL': f'Wheel-Version: 1.0\nGenerator: qiongli-native-registry-packages\nRoot-Is-Purelib: false\nTag: py3-none-{platform_tag}\n'.encode(),
        f'{dist}/entry_points.txt': b'[console_scripts]\nqiongli = qiongli_native:main\nql = qiongli_native:main\n',
        f'{dist}/licenses/LICENSE': regular_bytes(ROOT / 'LICENSE'),
    }
    record = io.StringIO(newline='')
    writer = csv.writer(record, lineterminator='\n')
    for name, data in sorted(files.items()):
        digest = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b'=').decode()
        writer.writerow([name, f'sha256={digest}', len(data)])
    writer.writerow([f'{dist}/RECORD', '', ''])
    files[f'{dist}/RECORD'] = record.getvalue().encode()
    path = out / f'qiongli-{version}-py3-none-{platform_tag}.whl'
    with zipfile.ZipFile(path, 'x', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in sorted(files.items()):
            info = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            info.create_system = 3
            mode = 0o755 if name == 'qiongli_native/bin/qiongli' else 0o644
            info.external_attr = (stat.S_IFREG | mode) << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, data)
    return path


def binary_packages(out: Path, binary_path: Path, version: str) -> list[Path]:
    binary = regular_bytes(binary_path)
    # Reject mislabeled native bytes, then derive the deployment floor from Mach-O.
    if binary[:8] != bytes.fromhex('cffaedfe0c000001'):
        raise ValueError('binary must be a thin macOS ARM64 Mach-O executable')
    load_commands = subprocess.check_output(['otool', '-l', str(binary_path)], text=True)
    versions = re.findall(r'^\s*minos (\d+)\.(\d+)(?:\.\d+)?$', load_commands, re.M)
    if len(versions) != 1:
        raise ValueError('cannot determine one macOS deployment target')
    major, minor = versions[0]
    reported = subprocess.check_output([str(binary_path), '--version'], text=True).strip()
    if reported != f'qiongli {version}':
        raise ValueError('executable version does not match the native workspace')
    identity = parse_release_version(version)
    readme = f'''# Qiongli {version}\n\nNative CLI prerelease for macOS {major}.{minor}+ on Apple Silicon.\nIncludes the existing native CLI and embedded content, MCP and Zotero resources.\nModels and credentials remain in your chosen Host. No download runs at install time.\n\nRun `qiongli --help` and `qiongli doctor` after installation.\nProject writes retain preview, explicit approval and revision checks.\nRegistry installation does not grant managed-update or signed Plugin activation\nauthority. Those commands still require their existing signed release inputs.\n\nThis package does not claim full 1.19 replacement, Windows support, or an App.\nUse your package manager to pin, upgrade or remove this prerelease.\n'''
    npm = out / 'npm'
    (npm / 'bin').mkdir(parents=True)
    (npm / 'native').mkdir()
    (npm / 'bin/qiongli.mjs').write_text(NPM_LAUNCHER)
    (npm / 'bin/qiongli.mjs').chmod(0o755)
    (npm / 'native/qiongli').write_bytes(binary)
    (npm / 'native/qiongli').chmod(0o755)
    (npm / 'README.md').write_text(readme)
    shutil.copyfile(ROOT / 'LICENSE', npm / 'LICENSE')
    (npm / 'package.json').write_text(json.dumps({
        'name': 'qiongli', 'version': version, 'description': 'Native academic research CLI',
        'type': 'module', 'license': 'MIT', 'repository': 'github:jxpeng98/qiongli',
        'bin': {'qiongli': 'bin/qiongli.mjs', 'ql': 'bin/qiongli.mjs'},
        'os': ['darwin'], 'cpu': ['arm64'], 'engines': {'node': '>=18'},
        'files': ['bin/', 'native/', 'README.md', 'LICENSE'],
        'publishConfig': {'access': 'public', 'tag': identity.channel},
    }, indent=2) + '\n')
    # npm owns tarball conventions and executable entrypoint handling.
    packed = json.loads(subprocess.check_output([
        'npm', 'pack', '--json', '--ignore-scripts', '--pack-destination', str(out),
        '--cache', str(out / 'npm-cache'),
    ], cwd=npm, text=True))
    tarball = out / packed[0]['filename']
    with tarfile.open(tarball) as archive:
        if archive.extractfile('package/native/qiongli').read() != binary:
            raise ValueError('npm archive changed the candidate executable')
    whl = wheel(out, identity.package_version, f'macosx_{major}_{minor}_arm64', binary, readme)
    return [tarball, whl]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out-dir', required=True, type=Path)
    parser.add_argument('--package-cargo', action='store_true', help='also create Cargo archives without claiming verification')
    parser.add_argument('--binary', type=Path, help='optional macOS ARM64 candidate; omit for Cargo staging only')
    args = parser.parse_args()
    out = args.out_dir.expanduser().absolute()
    if out.exists() or out.is_symlink() or ROOT == out.resolve() or ROOT in out.resolve().parents:
        parser.error('--out-dir must be a new directory outside the source checkout')
    version = tomllib.loads((NATIVE / 'Cargo.toml').read_text())['workspace']['package']['version']
    if parse_release_version(version).release_line != 'native-2x':
        parser.error('expected native 2.x version')
    out.mkdir(parents=True)
    workspace = stage_cargo(out, version)
    artifacts = binary_packages(out, args.binary.absolute(), version) if args.binary else []
    if args.package_cargo:
        subprocess.run(['cargo', 'package', '--manifest-path', str(workspace / 'Cargo.toml'),
                        '--workspace', '--no-default-features', '--no-verify', '--offline',
                        '--allow-dirty', '--target-dir', str(out / 'cargo-build')], check=True)
        for archive in sorted((out / 'cargo-build/package').glob('*.crate')):
            target = out / archive.name
            shutil.copyfile(archive, target)
            artifacts.append(target)
    receipt = {'version': version, 'status': 'staged-unpublished', 'cargo_source': str(workspace),
               'binary_target': 'aarch64-apple-darwin' if args.binary else None,
               'binary_sha256': hashlib.sha256(regular_bytes(args.binary)).hexdigest() if args.binary else None,
               'cargo_verification': 'not-run',
               'artifacts': [{'file': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest(), 'bytes': p.stat().st_size} for p in artifacts]}
    (out / 'registry-packages.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
