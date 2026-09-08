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
import subprocess
import tarfile
import tomllib

try:
    from .native_registry_install_check import check_cli
    from .native_registry_packages import NATIVE, ROOT, binary_packages, regular_bytes
except ImportError:
    from native_registry_install_check import check_cli
    from native_registry_packages import NATIVE, ROOT, binary_packages, regular_bytes


def archive_cli(path: Path, binary: Path, readme: bytes) -> None:
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
    args = parser.parse_args()
    version = tomllib.loads((NATIVE / 'Cargo.toml').read_text())['workspace']['package']['version']
    if args.version.removeprefix('v') != version or not version.startswith('2.0.0-alpha.'):
        parser.error('this lane requires the current native 2.0.0 Alpha version')
    if (platform.system(), platform.machine()) != ('Darwin', 'arm64'):
        parser.error('only the macOS ARM64 target has package/install qualification')
    out = args.out_dir.expanduser()
    if out.exists() or out.is_symlink() or ROOT in out.resolve().parents or out.resolve() == ROOT:
        parser.error('--out-dir must be new and outside the checkout')
    def git(*arguments):
        return subprocess.check_output(['git', *arguments], cwd=ROOT, text=True).strip()
    if git('status', '--porcelain', '--untracked-files=normal') or git('branch', '--show-current') != '2.x':
        parser.error('qualify a clean, locally merged 2.x candidate')
    commit = git('rev-parse', 'HEAD')
    out = out.resolve()
    out.mkdir(parents=True)
    assets = out / 'assets'
    assets.mkdir()
    env = os.environ.copy()
    # This lane carries no managed-product authority, even on a signing machine.
    for key in ('QIONGLI_NATIVE_RELEASE_AUTHORITY_FILE', 'QIONGLI_MACOS_EXPECTED_TEAM_ID',
                'QIONGLI_NATIVE_SOURCE_COMMIT'):
        env.pop(key, None)
    def run(command, cwd=NATIVE):
        subprocess.run(command, cwd=cwd, env=env, check=True)
    run(['bash', str(ROOT / 'scripts/verify_release_tag_version.sh'), '--root', str(ROOT), '--tag', f'v{version}'])
    run(['cargo', 'fmt', '--all', '--check'])
    for command in ('clippy', 'test'):
        argv = ['cargo', command, '--workspace', '--exclude', 'qiongli-ui',
                '--all-targets', '--no-default-features', '--locked', '--offline']
        run(argv + (['--', '-D', 'warnings'] if command == 'clippy' else []))
    env['QIONGLI_NATIVE_SOURCE_COMMIT'] = commit
    run(['cargo', 'build', '-p', 'qiongli', '--bin', 'qiongli', '--no-default-features',
         '--release', '--locked', '--offline'])
    metadata = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--offline'], cwd=NATIVE, env=env))
    binary = Path(metadata['target_directory']) / 'release/qiongli'
    package_work = out / 'packages'
    package_work.mkdir()
    package_paths = binary_packages(package_work, binary, version)
    for path in package_paths:
        (assets / path.name).write_bytes(regular_bytes(path))
    readme = f'''# Qiongli {version} — CLI Alpha

macOS Apple Silicon, macOS 11+. Download and extract this archive, then run
`./qiongli --version`, `./qiongli --help`, or `./qiongli doctor`.
No App, Cargo, Python or Node is needed for the standalone executable.
To use MCP, configure your Host to run the absolute path to `qiongli` with
arguments `mcp serve --profile full --transport stdio` (or profile `lite`).
The Host owns its model and credentials. MCP stdout contains protocol messages.

The accompanying npm tarball and Python wheel contain this same binary and
can be installed from downloaded files; they are not registry publication.
Verify SHA256SUMS before installation. This Alpha is not Apple notarized.
Existing research preview/approval/CAS checks remain in force.
Managed Plugin/Skill activation, automatic migration and signed self-update
still require their existing product authority and are not enabled by this
download. Do not use the Community Alpha installer for this release.
Windows installation remains partially complete/paused; Linux binaries and
full 1.x replacement are not claimed by this macOS release.

Upgrade by installing a new version in a separate directory and changing your
PATH/Host executable path. Roll back by restoring the old executable/path;
retain research data and backups (this does not reverse data migrations).
Remove only the directory you extracted and any PATH/Host entry you added.

Source commit: {commit}
'''.encode()
    archive = assets / f'qiongli-{version}-aarch64-apple-darwin.tar.gz'
    archive_cli(archive, binary, readme)
    extracted = out / 'extracted'
    with tarfile.open(archive) as packet:
        packet.extractall(extracted, filter='data')
    if regular_bytes(extracted / 'qiongli') != regular_bytes(binary):
        raise ValueError('archive changed the candidate executable')
    home = out / 'home'
    home.mkdir(mode=0o700)
    smoke_env = env | {'HOME': str(home), 'USERPROFILE': str(home),
                       'QIONGLI_CONFIG_HOME': str(home / 'config')}
    smoke = check_cli(extracted / 'qiongli', version=version, root=extracted, env=smoke_env)
    package_receipt = {'version': version, 'artifacts': [
        {'file': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()}
        for p in package_paths]}
    (package_work / 'registry-packages.json').write_text(json.dumps(package_receipt))
    run(['python3', str(ROOT / 'scripts/native_registry_install_check.py'),
         '--packages', str(package_work), '--out-dir', str(out / 'install')], cwd=ROOT)
    if git('rev-parse', 'HEAD') != commit or git('status', '--porcelain', '--untracked-files=normal'):
        raise ValueError('source changed during qualification')
    receipt = {'version': version, 'source_commit': commit, 'target': 'aarch64-apple-darwin',
               'channel': 'github-release', 'status': 'qualified-unpublished',
               'managed_product_authority': False,
               'rustc': subprocess.check_output(['rustc', '--version'], cwd=NATIVE, text=True).strip(),
               'checks': {'cli_clippy': 'passed', 'cli_tests': 'passed', 'archive_smoke': smoke,
                          'npm_wheel_local_install': 'passed'},
               'artifacts': [{'file': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest(),
                              'bytes': p.stat().st_size} for p in sorted(assets.iterdir())]}
    (assets / 'release-manifest.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (assets / 'SHA256SUMS').write_text(''.join(
        f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n' for p in sorted(assets.iterdir())))
    print(f'CLI GitHub Release assets qualified at {assets}; publication is a separate operation.')


if __name__ == '__main__':
    main()
