#!/usr/bin/env python3
"""Qualify CLI-only GitHub Release assets locally; never push or publish."""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import sys
import zipfile
import subprocess
import tarfile
import tomllib

try:
    from .native_registry_install_check import check_cli
    from .native_registry_packages import NATIVE, ROOT, TARGETS, binary_packages, regular_bytes, parse_release_version, package_readme
except ImportError:
    from native_registry_install_check import check_cli
    from native_registry_packages import NATIVE, ROOT, TARGETS, binary_packages, regular_bytes, parse_release_version, package_readme


def archive_cli(path: Path, binary: Path, readme: bytes, target: str = 'aarch64-apple-darwin') -> None:
    if target.endswith('msvc'):
        with zipfile.ZipFile(path, 'x', compression=zipfile.ZIP_DEFLATED) as archive:
            archive.writestr('qiongli.exe', regular_bytes(binary))
            archive.writestr('README.md', readme)
            archive.writestr('LICENSE', regular_bytes(ROOT / 'LICENSE'))
        return
    with tarfile.open(path, 'x:gz') as archive:
        for name, data, mode in [
            ('qiongli', regular_bytes(binary), 0o755),
            ('README.md', readme, 0o644),
            ('LICENSE', regular_bytes(ROOT / 'LICENSE'), 0o644),
        ]:
            entry = tarfile.TarInfo(name)
            entry.size, entry.mode = len(data), mode
            archive.addfile(entry, io.BytesIO(data))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--version', required=True)
    parser.add_argument('--out-dir', required=True, type=Path)
    parser.add_argument('--target', choices=TARGETS)
    args = parser.parse_args()
    version = tomllib.loads((NATIVE / 'Cargo.toml').read_text())['workspace']['package']['version']
    identity = parse_release_version(args.version)
    if identity.version != version or identity.release_line != 'native-2x':
        parser.error('version must match the current native SemVer version')
    host_target = {('Darwin', 'arm64'): 'aarch64-apple-darwin',
                   ('Linux', 'x86_64'): 'x86_64-unknown-linux-gnu',
                   ('Windows', 'AMD64'): 'x86_64-pc-windows-msvc'}.get((platform.system(), platform.machine()))
    target = args.target or host_target
    if not target or target != host_target:
        parser.error('installation qualification must run on the declared target OS/architecture')
    out = args.out_dir.expanduser()
    if out.exists() or out.is_symlink() or ROOT in out.resolve().parents or out.resolve() == ROOT:
        parser.error('--out-dir must be new and outside the checkout')
    def git(*arguments):
        return subprocess.check_output(['git', *arguments], cwd=ROOT, text=True).strip()
    commit = git('rev-parse', 'HEAD')
    ci = os.environ.get('GITHUB_ACTIONS') == 'true' and os.environ.get('GITHUB_SHA') == commit
    if git('status', '--porcelain', '--untracked-files=normal') or (not ci and git('branch', '--show-current') != '2.x'):
        parser.error('qualify clean local 2.x or the exact GitHub Actions source commit')
    out = out.resolve()
    out.mkdir(parents=True)
    assets = out / 'assets'
    assets.mkdir()
    env = os.environ.copy()
    env['QIONGLI_PYTHON'] = sys.executable
    # This lane carries no managed-product authority, even on a signing machine.
    for key in ('QIONGLI_NATIVE_RELEASE_AUTHORITY_FILE', 'QIONGLI_MACOS_EXPECTED_TEAM_ID',
                'QIONGLI_NATIVE_SOURCE_COMMIT'):
        env.pop(key, None)
    def run(command, cwd=NATIVE):
        subprocess.run(command, cwd=cwd, env=env, check=True)
    run(['bash', str(ROOT / 'scripts/verify_release_tag_version.sh'), '--root', str(ROOT), '--tag', f'v{version}'])
    cargo_args = ['--no-default-features', '--locked'] + ([] if ci else ['--offline'])
    lint = not ci or platform.system() == 'Linux'
    if lint:
        run(['cargo', 'fmt', '--all', '--check'])
        run(['cargo', 'clippy', '--workspace', '--exclude', 'qiongli-ui', '--all-targets', *cargo_args, '--', '-D', 'warnings'])
    run(['cargo', 'test', '-p', 'qiongli', '--release', '--target', target,
         '--test', 'cli', '--test', 'mcp_stdio', *cargo_args])
    env['QIONGLI_NATIVE_SOURCE_COMMIT'] = commit
    run(['cargo', 'build', '-p', 'qiongli', '--bin', 'qiongli', '--no-default-features',
         '--release', '--target', target, *cargo_args])
    metadata = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--offline'], cwd=NATIVE, env=env))
    binary = Path(metadata['target_directory']) / target / 'release' / TARGETS[target][2]
    package_work = out / 'packages'
    package_work.mkdir()
    package_paths = binary_packages(package_work, binary, version, target)
    for path in package_paths:
        (assets / path.name).write_bytes(regular_bytes(path))
    readme = (package_readme(version) + f'\nSource commit: {commit}\nTarget: {target}\n').encode()
    extension = 'zip' if target.endswith('msvc') else 'tar.gz'
    archive = assets / f'qiongli-{version}-{target}.{extension}'
    archive_cli(archive, binary, readme, target)
    extracted = out / 'extracted'
    if extension == 'zip':
        with zipfile.ZipFile(archive) as packet:
            packet.extractall(extracted)
    else:
        with tarfile.open(archive) as packet:
            packet.extractall(extracted, filter='data')
    if regular_bytes(extracted / TARGETS[target][2]) != regular_bytes(binary):
        raise ValueError('archive changed the candidate executable')
    home = out / 'home'
    home.mkdir(mode=0o700)
    smoke_env = env | {'HOME': str(home), 'USERPROFILE': str(home),
                       'QIONGLI_CONFIG_HOME': str(home / 'config')}
    smoke = check_cli(extracted / TARGETS[target][2], version=version, root=extracted, env=smoke_env)
    package_receipt = {'version': version, 'artifacts': [
        {'file': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()}
        for p in package_paths]}
    (package_work / 'registry-packages.json').write_text(json.dumps(package_receipt))
    run([sys.executable, str(ROOT / 'scripts/native_registry_install_check.py'),
         '--packages', str(package_work), '--out-dir', str(out / 'install')], cwd=ROOT)
    if git('rev-parse', 'HEAD') != commit or git('status', '--porcelain', '--untracked-files=normal'):
        raise ValueError('source changed during qualification')
    receipt = {'version': version, 'source_commit': commit, 'target': target,
               'channel': 'github-release', 'status': 'qualified-unpublished',
               'managed_product_authority': False,
               'rustc': subprocess.check_output(['rustc', '--version'], cwd=NATIVE, text=True).strip(),
               'checks': {'cli_clippy': 'passed' if lint else 'covered-by-linux-job', 'cli_mcp_tests': 'passed', 'archive_smoke': smoke,
                          'npm_wheel_local_install': 'passed'},
               'artifacts': [{'file': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest(),
                              'bytes': p.stat().st_size} for p in sorted(assets.iterdir())]}
    (assets / 'release-manifest.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (assets / 'SHA256SUMS').write_text(''.join(
        f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n' for p in sorted(assets.iterdir())))
    print(f'CLI GitHub Release assets qualified at {assets}; publication is a separate operation.')


if __name__ == '__main__':
    main()
