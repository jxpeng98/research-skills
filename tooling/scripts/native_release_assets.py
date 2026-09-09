#!/usr/bin/env python3
"""Assemble or verify one three-platform CLI release packet; never publish."""
import argparse
import hashlib
import json
import platform
import os
import subprocess
import re
from pathlib import Path
import shutil
import tarfile
import zipfile

try:
    from .native_registry_packages import TARGETS, npm_package, parse_release_version, regular_bytes, validate_binary
except ImportError:
    from native_registry_packages import TARGETS, npm_package, parse_release_version, regular_bytes, validate_binary


def checked_assets(root, manifest):
    assets = {}
    for item in manifest['artifacts']:
        name = item['file']
        if Path(name).name != name or '/' in name or '\\' in name or name in {'.', '..', ''} or name in assets:
            raise ValueError('unsafe or duplicate artifact name')
        path = root / name
        data = regular_bytes(path)
        if hashlib.sha256(data).hexdigest() != item['sha256'] or len(data) != item['bytes']:
            raise ValueError(f'artifact digest/size mismatch: {name}')
        assets[name] = path
    return assets


def check_identity(manifest, version, commit):
    if manifest['version'] != version or manifest['source_commit'] != commit:
        raise ValueError('mixed version or source commit')


def assemble(root, out, version, commit):
    out.mkdir(parents=True, exist_ok=False)
    binaries, receipts = {}, []
    for target in TARGETS:
        source = root / target
        receipt = json.loads(regular_bytes(source / 'release-manifest.json'))
        check_identity(receipt, version, commit)
        checks = receipt['checks']
        if (receipt['target'] != target or checks['cli_mcp_tests'] != 'passed'
                or checks['npm_wheel_local_install'] != 'passed'
                or checks['archive_smoke']['version'] != version):
            raise ValueError(f'missing target-native checks: {target}')
        if target.endswith('linux-gnu') and checks['cli_clippy'] != 'passed':
            raise ValueError('missing Linux Clippy gate')
        files = checked_assets(source, receipt)
        extension = 'zip' if target.endswith('msvc') else 'tar.gz'
        archive_name = f'qiongli-{version}-{target}.{extension}'
        archive = files[archive_name]
        binary_name = TARGETS[target][2]
        if extension == 'zip':
            with zipfile.ZipFile(archive) as packet:
                data = packet.read(binary_name)
        else:
            with tarfile.open(archive) as packet:
                member = packet.getmember(binary_name)
                if not member.isfile():
                    raise ValueError('archive executable is not regular')
                data = packet.extractfile(member).read()
        validate_binary(data, target)
        binary = out.parent / 'npm-binaries' / target / binary_name
        binary.parent.mkdir(parents=True, exist_ok=True)
        binary.write_bytes(data)
        binary.chmod(0o755)
        binaries[target] = binary
        wheels = [p for name, p in files.items() if name.endswith('.whl')]
        if len(wheels) != 1:
            raise ValueError('expected one wheel per target')
        for path in [archive, *wheels]:
            shutil.copyfile(path, out / path.name)
        receipts.append(receipt)
    npm_work = out.parent / 'npm-combined'
    npm_work.mkdir(exist_ok=False)
    packed = npm_package(npm_work, binaries, version)
    shutil.copyfile(packed, out / packed.name)
    manifest = {'version': version, 'source_commit': commit, 'targets': list(TARGETS),
                'status': 'three-platform-packaged', 'target_evidence': receipts,
                'artifacts': [{'file': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest(),
                               'bytes': p.stat().st_size} for p in sorted(out.iterdir())]}
    (out / 'release-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    (out / 'SHA256SUMS').write_text(''.join(f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n'
                                         for p in sorted(out.iterdir())))


def verify(root, version, commit):
    manifest = json.loads(regular_bytes(root / 'release-manifest.json'))
    check_identity(manifest, version, commit)
    if set(manifest['targets']) != set(TARGETS) or len(manifest['target_evidence']) != len(TARGETS):
        raise ValueError('all three platforms are required before registry publication')
    seen = set()
    for receipt in manifest['target_evidence']:
        check_identity(receipt, version, commit)
        seen.add(receipt['target'])
        if receipt['checks']['cli_mcp_tests'] != 'passed' or receipt['checks']['npm_wheel_local_install'] != 'passed':
            raise ValueError('target-native qualification is incomplete')
    if seen != set(TARGETS):
        raise ValueError('duplicate or missing target evidence')
    files = checked_assets(root, manifest)
    identity = parse_release_version(version)
    npm = files[f'qiongli-{identity.npm_version}.tgz']
    with tarfile.open(npm) as packet:
        metadata = json.load(packet.extractfile('package/package.json'))
        if metadata['version'] != identity.npm_version or metadata['publishConfig']['tag'] != identity.npm_dist_tag:
            raise ValueError('npm version/channel mismatch')
        if metadata['name'] != 'qiongli' or metadata.get('scripts'):
            raise ValueError('unexpected npm package or install scripts')
        for target, (_, _, binary) in TARGETS.items():
            validate_binary(packet.extractfile(f'package/native/{target}/{binary}').read(), target)
    expected = {'aarch64-apple-darwin': 'macosx_', 'x86_64-unknown-linux-gnu': 'manylinux_2_35_',
                'x86_64-pc-windows-msvc': 'win_amd64'}
    wheels = {}
    for target, marker in expected.items():
        matches = [p for name, p in files.items() if name.endswith('.whl') and marker in name]
        if len(matches) != 1 or not matches[0].name.startswith(f'qiongli-{identity.package_version}-'):
            raise ValueError('wheel version/target mismatch')
        with zipfile.ZipFile(matches[0]) as packet:
            metadata = packet.read(f'qiongli-{identity.package_version}.dist-info/METADATA').decode()
            if f'\nVersion: {identity.package_version}\n' not in metadata:
                raise ValueError('wheel metadata version mismatch')
        wheels[target] = matches[0]
    if len(files) != 7:
        raise ValueError('expected three archives, three wheels and one npm package')
    return manifest, npm, wheels


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['assemble', 'verify', 'install-input'])
    parser.add_argument('--root', required=True, type=Path)
    parser.add_argument('--out', type=Path)
    parser.add_argument('--version', required=True)
    parser.add_argument('--commit', required=True)
    parser.add_argument('--require-ci', action='store_true')
    args = parser.parse_args()
    identity = parse_release_version(args.version)
    if identity.release_line != 'native-2x':
        parser.error('native release required')
    if not re.fullmatch(r'(?:[a-f0-9]{40}|[a-f0-9]{64})', args.commit):
        parser.error('an exact source commit is required')
    if args.mode == 'assemble':
        if not args.out:
            parser.error('--out is required')
        assemble(args.root, args.out, identity.version, args.commit)
        return
    manifest, npm, wheels = verify(args.root, identity.version, args.commit)
    if args.require_ci:
        repo = os.environ['GITHUB_REPOSITORY']
        runs = json.loads(subprocess.check_output(['gh', 'api',
            f'repos/{repo}/actions/workflows/native-cli-distribution.yml/runs?head_sha={args.commit}&status=success'], text=True))
        if not any(r['head_sha'] == args.commit and r['conclusion'] == 'success' for r in runs['workflow_runs']):
            raise ValueError('no successful three-platform combined-install CI run at this source')
    if args.mode == 'install-input':
        target = {('Darwin', 'arm64'): 'aarch64-apple-darwin', ('Linux', 'x86_64'): 'x86_64-unknown-linux-gnu',
                  ('Windows', 'AMD64'): 'x86_64-pc-windows-msvc'}[(platform.system(), platform.machine())]
        selected = {npm.name, wheels[target].name}
        (args.root / 'registry-packages.json').write_text(json.dumps({
            'version': identity.version, 'artifacts': [a for a in manifest['artifacts'] if a['file'] in selected]}))
    print(f'Verified three-platform packet {identity.version} at {args.commit}')


if __name__ == '__main__':
    main()
