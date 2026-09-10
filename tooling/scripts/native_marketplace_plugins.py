#!/usr/bin/env python3
"""Project native content into public marketplace archives; never install or publish."""
from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import struct
import tarfile

try:
    from .native_registry_packages import regular_bytes
    from .release_version import parse_release_version
except ImportError:
    from native_registry_packages import regular_bytes
    from release_version import parse_release_version

EXPORT = '.qiongli-marketplace-export.json'
RECEIPT = '.qiongli-marketplace.json'
SKILL_ROOT = 'skills/qiongli-workflow/'
BRIDGE = 'mcp/qiongli-native.mjs'
PLATFORMS = ('codex', 'claude')


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def json_bytes(value) -> bytes:
    return (json.dumps(value, indent=2, ensure_ascii=False) + '\n').encode()


def identity(version: str, commit: str) -> None:
    parsed = parse_release_version(version)
    if parsed.release_line != 'native-2x' or not parsed.is_prerelease or parsed.version != version:
        raise ValueError('public qiongli-next requires a canonical native prerelease SemVer')
    if not re.fullmatch(r'[0-9a-f]{40}', commit):
        raise ValueError('source commit must be a full lowercase Git SHA')


def safe_path(value: str) -> str:
    if (not isinstance(value, str) or not value or '\\' in value
            or '\x00' in value or PurePosixPath(value).is_absolute()
            or any(part in ('', '.', '..') for part in value.split('/'))):
        raise ValueError('resource path must be a normalized relative POSIX path')
    return value


def validate_export(metadata: dict, version: str, commit: str) -> dict:
    identity(version, commit)
    if (metadata.get('schema_version') != 1 or metadata.get('version') != version
            or metadata.get('source_commit') != commit):
        raise ValueError('export source/version does not match the release')
    for field, length in [('content_source_commit', 40), ('pack_sha256', 64), ('content_root_sha256', 64)]:
        if not re.fullmatch(r'[0-9a-f]{' + str(length) + r'}', str(metadata.get(field, ''))):
            raise ValueError(f'invalid export {field}')
    entries = metadata.get('entries')
    if not isinstance(entries, list) or not entries:
        raise ValueError('export entries are required')
    expected = {}
    for entry in entries:
        name = safe_path(entry['path'])
        if name == EXPORT or name in expected:
            raise ValueError('duplicate or reserved export path')
        if (type(entry.get('size_bytes')) is not int or entry['size_bytes'] < 0
                or not re.fullmatch(r'[0-9a-f]{64}', str(entry.get('sha256', '')))):
            raise ValueError('invalid export entry digest/size')
        expected[name] = entry
    required = {'workflow/SKILL.md', '.codex-plugin/plugin.json', '.claude-plugin/plugin.json'}
    if not required <= expected.keys():
        raise ValueError('export lacks workflow or platform manifests')
    return expected


def check_bytes(data: bytes, entry: dict) -> None:
    if len(data) != entry['size_bytes'] or digest(data) != entry['sha256']:
        raise ValueError('resource differs from the exported native pack')


def verify_pack(metadata: dict, content: dict[str, bytes]) -> None:
    """Rebuild the existing QLPACK byte stream, using its exact native JSON."""
    manifest_bytes = metadata['pack_manifest_json'].encode()
    manifest = json.loads(manifest_bytes)
    if (manifest.get('format_version') != 1
            or manifest.get('content_version') != metadata['version']
            or manifest.get('source_commit') != metadata['content_source_commit']
            or manifest.get('content_root_sha256') != metadata['content_root_sha256']):
        raise ValueError('native pack manifest identity mismatch')
    expected = {entry['path']: entry for entry in metadata['entries']}
    hasher = hashlib.sha256(b'QLPACK\0\0' + struct.pack('<IQ', 1, len(manifest_bytes)))
    hasher.update(manifest_bytes)
    seen, offset = set(), 0
    for entry in manifest['entries']:
        name = safe_path(entry['path'])
        if name not in expected or name in seen or entry['payload_offset'] != offset:
            raise ValueError('native pack payload is incomplete, duplicated or noncontiguous')
        if any(entry[key] != expected[name][key] for key in ('size_bytes', 'sha256')):
            raise ValueError('export metadata differs from the native pack manifest')
        check_bytes(content[name], entry)
        hasher.update(content[name])
        offset += entry['size_bytes']
        seen.add(name)
    if seen != content.keys() or seen != expected.keys() or hasher.hexdigest() != metadata['pack_sha256']:
        raise ValueError('projected content cannot reproduce the native pack digest')


def read_content(source: Path, version: str, commit: str) -> tuple[dict, dict[str, bytes]]:
    if source.is_symlink() or not source.is_dir():
        raise ValueError('content directory must be a regular directory')
    metadata = json.loads(regular_bytes(source / EXPORT))
    expected = validate_export(metadata, version, commit)
    content = {}
    for path in sorted(source.rglob('*')):
        if path.is_symlink() or not (path.is_dir() or path.is_file()):
            raise ValueError('export contains a link or special file')
        if path.is_file() and path.name != EXPORT:
            name = safe_path(path.relative_to(source).as_posix())
            if name not in expected:
                raise ValueError('export contains an unlisted resource')
            data = regular_bytes(path)
            check_bytes(data, expected[name])
            content[name] = data
    if content.keys() != expected.keys():
        raise ValueError('export resource is missing')
    verify_pack(metadata, content)
    return metadata, content


def bridge(version: str) -> bytes:
    # Windows .cmd dispatch needs a shell. Every token is fixed here; user argv
    # is deliberately ignored, and the version has passed canonical parsing.
    return f'''#!/usr/bin/env node
import {{ spawn }} from 'node:child_process';
const args = ['--yes', 'qiongli@{version}', 'mcp', 'serve', '--profile', 'lite', '--transport', 'stdio'];
const windows = process.platform === 'win32';
const command = windows ? ['npx.cmd', ...args].join(' ') : 'npx';
const child = spawn(command, windows ? [] : args, {{ stdio: 'inherit', shell: windows }});
for (const signal of ['SIGINT', 'SIGTERM', 'SIGHUP']) process.on(signal, () => child.kill(signal));
child.on('error', () => {{ console.error('Unable to start the pinned Qiongli native MCP.'); process.exitCode = 1; }});
child.on('close', (code, signal) => {{
  if (signal) {{ process.removeAllListeners(signal); process.kill(process.pid, signal); }}
  else process.exitCode = code ?? 1;
}});
'''.encode()


def mcp_manifest(platform: str) -> dict:
    if platform == 'claude':
        server = {'command': 'node', 'args': ['${CLAUDE_PLUGIN_ROOT}/' + BRIDGE]}
    else:
        server = {'command': 'node', 'args': ['./' + BRIDGE], 'cwd': '.',
                  'startup_timeout_sec': 60, 'tool_timeout_sec': 60}
    return {'mcpServers': {'qiongli-next': server}}


def project(content: dict[str, bytes], platform: str, version: str) -> dict[str, bytes]:
    manifest_path = f'.{platform}-plugin/plugin.json'
    manifest = json.loads(content[manifest_path])
    if manifest.get('name') != 'qiongli' or manifest.get('version') != version:
        raise ValueError('canonical plugin manifest identity mismatch')
    manifest.update(name='qiongli-next', version=version, skills='./skills/', mcpServers='./.mcp.json')
    manifest['description'] = f'Native academic research workflows for {platform.title()} via the pinned npm CLI.'
    if isinstance(manifest.get('interface'), dict):
        manifest['interface']['displayName'] = 'Qiongli Next'
    files = {manifest_path: json_bytes(manifest), '.mcp.json': json_bytes(mcp_manifest(platform)), BRIDGE: bridge(version)}
    for name, data in content.items():
        if name in ('.codex-plugin/plugin.json', '.claude-plugin/plugin.json'):
            continue
        target = SKILL_ROOT + name.removeprefix('workflow/')
        if target in files:
            raise ValueError('canonical resource projection collision')
        files[target] = data
    return files


def archive_name(platform: str, version: str) -> str:
    return f'qiongli-next-{platform}-plugin-v{version}.tar.gz'


def verify_archive(path: Path, version: str, commit: str) -> dict:
    """Verify projected bytes/identity; this is not a signed product grant."""
    identity(version, commit)
    platform = next((p for p in PLATFORMS if path.name == archive_name(p, version)), None)
    if platform is None:
        raise ValueError('unexpected marketplace archive name')
    prefix = path.name.removesuffix('.tar.gz') + '/plugins/qiongli-next/'
    files = {}
    with tarfile.open(fileobj=io.BytesIO(regular_bytes(path)), mode='r:gz') as archive:
        for member in archive:
            safe_path(member.name)
            if not member.isfile() or not member.name.startswith(prefix):
                raise ValueError('archive contains a nonregular or out-of-root entry')
            name = safe_path(member.name[len(prefix):])
            if name in files:
                raise ValueError('duplicate archive entry')
            files[name] = archive.extractfile(member).read()
    receipt = json.loads(files.pop(RECEIPT))
    if receipt.get('platform') != platform or receipt.get('schema_version') != 1:
        raise ValueError('marketplace receipt identity mismatch')
    metadata = receipt['source']
    expected = validate_export(metadata, version, commit)
    if files.keys() != receipt['files'].keys():
        raise ValueError('marketplace archive file set mismatch')
    for name, data in files.items():
        check_bytes(data, receipt['files'][name])
    # Carry exact source templates so the transformed platform manifest can be
    # reproduced without trusting its self-reported output hash.
    content = {name: receipt['source_manifest_bytes'][name].encode() for name in
               ('.codex-plugin/plugin.json', '.claude-plugin/plugin.json')}
    for name in expected:
        if name not in content:
            content[name] = files[SKILL_ROOT + name.removeprefix('workflow/')]
        check_bytes(content[name], expected[name])
    verify_pack(metadata, content)
    if project(content, platform, version) != files:
        raise ValueError('marketplace wrapper differs from the native projection')
    return {'version': version, 'source_commit': commit, 'platform': platform,
            'pack_sha256': metadata['pack_sha256'], 'content_root_sha256': metadata['content_root_sha256'],
            'sha256': digest(regular_bytes(path)), 'bytes': path.stat().st_size}


def build_plugins(content_dir: Path, out_dir: Path, version: str, commit: str) -> list[Path]:
    metadata, content = read_content(content_dir, version, commit)
    if out_dir.is_symlink() or out_dir.exists():
        raise ValueError('output directory must be new')
    out_dir.mkdir(parents=True)
    archives = []
    for platform in PLATFORMS:
        files = project(content, platform, version)
        source_names = ('.codex-plugin/plugin.json', '.claude-plugin/plugin.json')
        files[RECEIPT] = json_bytes({
            'schema_version': 1, 'platform': platform, 'source': metadata,
            'source_manifest_bytes': {name: content[name].decode() for name in source_names},
            'files': {name: {'size_bytes': len(data), 'sha256': digest(data)} for name, data in sorted(files.items())},
        })
        plugin = out_dir / platform / 'plugins/qiongli-next'
        for name, data in files.items():
            target = plugin / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        archive = out_dir / archive_name(platform, version)
        prefix = archive.name.removesuffix('.tar.gz') + '/plugins/qiongli-next/'
        with archive.open('xb') as raw, gzip.GzipFile(filename='', fileobj=raw, mode='wb', mtime=0) as zipped:
            with tarfile.open(fileobj=zipped, mode='w') as tar:
                for name, data in sorted(files.items()):
                    info = tarfile.TarInfo(prefix + name)
                    info.size, info.mode, info.mtime = len(data), 0o644, 0
                    tar.addfile(info, io.BytesIO(data))
        verify_archive(archive, version, commit)
        archives.append(archive)
    return archives


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--content-dir', required=True, type=Path)
    parser.add_argument('--out-dir', required=True, type=Path)
    parser.add_argument('--version', required=True)
    parser.add_argument('--commit', required=True)
    args = parser.parse_args()
    try:
        archives = build_plugins(args.content_dir, args.out_dir, args.version, args.commit)
        print(json.dumps([dict(path=str(p), **verify_archive(p, args.version, args.commit)) for p in archives], indent=2))
    except (ValueError, KeyError, TypeError, OSError, tarfile.TarError) as error:
        parser.exit(1, f'marketplace projection failed: {error}\n')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
