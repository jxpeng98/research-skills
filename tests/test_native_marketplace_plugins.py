from __future__ import annotations

import io
import json
from pathlib import Path
import subprocess
import struct
import tarfile
import tempfile
import unittest

from tooling.scripts import native_marketplace_plugins as plugins


VERSION = '2.0.0-alpha.8'
COMMIT = 'a' * 40
TARGET = 'aarch64-apple-darwin'
BINARIES = {
    TARGET: bytes.fromhex('cffaedfe0c000001') + bytes(100),
    'x86_64-unknown-linux-gnu': b'\x7fELF\x02\x01' + bytes(12) + b'\x3e\x00' + bytes(80),
    'x86_64-pc-windows-msvc': b'MZ' + bytes(58) + (64).to_bytes(4, 'little') + b'PE\x00\x00\x64\x86',
}


class NativeMarketplacePluginsTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.source = self.root / 'content'
        self.source.mkdir()
        self.content = {
            'workflow/SKILL.md': b'---\nname: qiongli\n---\nUse supplied evidence; the Host owns models.\n',
            'workflow/references/platform-routing.md': b'Unavailable tools confer no execution evidence.\n',
            'skills/A_framing/question-refiner.md': 'Canonical evidence 中文.\n'.encode(),
        }
        for platform in plugins.PLATFORMS:
            self.content[f'.{platform}-plugin/plugin.json'] = plugins.json_bytes({
                'name': 'qiongli', 'version': VERSION, 'skills': './',
            })
        self.metadata = {
            'schema_version': 1, 'version': VERSION, 'source_commit': COMMIT,
            'content_source_commit': 'b' * 40, 'pack_sha256': 'c' * 64,
            'content_root_sha256': 'd' * 64,
            'entries': [{'path': name, 'size_bytes': len(data), 'sha256': plugins.digest(data)}
                        for name, data in self.content.items()],
        }
        for name, data in self.content.items():
            path = self.source / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        offset, entries = 0, []
        for entry in self.metadata['entries']:
            entries.append(dict(entry, payload_offset=offset, resource_kind='workflow', mode='regular'))
            offset += entry['size_bytes']
        manifest = dict(format_version=1, content_version=VERSION, source_commit='b' * 40,
                        content_root_sha256='d' * 64, entries=entries)
        raw = json.dumps(manifest, separators=(',', ':'), sort_keys=True).encode()
        self.metadata['pack_manifest_json'] = raw.decode()
        pack = b'QLPACK\0\0' + struct.pack('<IQ', 1, len(raw)) + raw + b''.join(self.content.values())
        self.metadata['pack_sha256'] = plugins.digest(pack)
        self.write_metadata()
        self.binary = self.root / 'qiongli'
        self.binary.write_bytes(BINARIES[TARGET])

    def write_metadata(self):
        (self.source / plugins.EXPORT).write_bytes(plugins.json_bytes(self.metadata))

    def build(self, name='out', target=TARGET):
        return plugins.build_plugins(self.source, self.root / name, VERSION, COMMIT, self.binary, target)

    def test_both_archives_preserve_canonical_bytes_and_bundle_runtime(self):
        archives = self.build()
        self.assertEqual([p.name for p in archives], [plugins.archive_name(p, VERSION, TARGET) for p in plugins.PLATFORMS])
        for platform, archive in zip(plugins.PLATFORMS, archives):
            verified = plugins.verify_archive(archive, VERSION, COMMIT)
            self.assertEqual(verified['pack_sha256'], self.metadata['pack_sha256'])
            root = self.root / 'out' / platform / 'plugins' / plugins.plugin_name(TARGET)
            for name, data in self.content.items():
                if name.startswith(('.codex-plugin/', '.claude-plugin/')):
                    continue
                self.assertEqual((root / plugins.SKILL_ROOT / name.removeprefix('workflow/')).read_bytes(), data)
            self.assertFalse((root / plugins.SKILL_ROOT / plugins.EXPORT).exists())
            manifest = json.loads((root / f'.{platform}-plugin/plugin.json').read_text())
            self.assertEqual(manifest['name'], plugins.plugin_name(TARGET))
            self.assertEqual(manifest['skills'], './skills/')
            self.assertEqual(manifest['mcpServers'], './.mcp.json')
            mcp = json.loads((root / '.mcp.json').read_text())['mcpServers']['qiongli-next']
            self.assertEqual(mcp, plugins.mcp_manifest(platform, TARGET)['mcpServers']['qiongli-next'])
            self.assertFalse((root / plugins.BRIDGE).exists())
            self.assertEqual((root / plugins.binary_path(TARGET)).read_bytes(), BINARIES[TARGET])
            self.assertEqual(verified['binary_sha256'], plugins.digest(BINARIES[TARGET]))
        again = self.build('again')
        self.assertEqual([p.read_bytes() for p in archives], [p.read_bytes() for p in again])

    def test_reject_noncanonical_version_or_unpinned_source(self):
        for version in ['2.0.0a8', 'v2.0.0-alpha.8', '1.19.0-beta.1', '2.0.0', '2.0.0-alpha.8;echo bad']:
            with self.subTest(version=version), self.assertRaises(ValueError):
                plugins.build_plugins(self.source, self.root / 'bad', version, COMMIT, self.binary, TARGET)
        for commit in ['main', 'a' * 39, 'A' * 40, 'b' * 40]:
            with self.subTest(commit=commit), self.assertRaises(ValueError):
                plugins.build_plugins(self.source, self.root / 'bad', VERSION, commit, self.binary, TARGET)
        self.assertFalse((self.root / 'bad').exists())

    def test_source_missing_modified_unlisted_or_linked_fails(self):
        path = self.source / 'workflow/SKILL.md'
        original = path.read_bytes()
        path.write_bytes(b'changed')
        with self.assertRaises(ValueError): self.build()
        path.unlink()
        with self.assertRaises(ValueError): self.build()
        path.write_bytes(original)
        extra = self.source / 'unlisted'
        extra.write_bytes(b'extra')
        with self.assertRaises(ValueError): self.build()
        extra.unlink()
        extra.symlink_to(path)
        with self.assertRaises(ValueError): self.build()
        extra.unlink()
        self.metadata['entries'].append(dict(self.metadata['entries'][0]))
        self.write_metadata()
        with self.assertRaises(ValueError): self.build()

    def test_existing_output_and_projection_collision_fail(self):
        self.build()
        with self.assertRaises(ValueError): self.build()
        self.content['SKILL.md'] = b'collision'
        with self.assertRaises(ValueError): plugins.project(self.content, 'codex', VERSION)
        self.metadata['entries'][0]['path'] = '../outside'
        self.write_metadata()
        with self.assertRaises(ValueError): self.build('bad')

    def rewrite_archive(self, archive, change):
        with tarfile.open(archive) as old:
            members = [(m, old.extractfile(m).read()) for m in old]
        change(members)
        with tarfile.open(archive, 'w:gz') as out:
            for member, data in members:
                member.size = len(data)
                out.addfile(member, io.BytesIO(data))

    def test_archive_changed_bytes_links_duplicates_and_foreign_source_fail(self):
        archive = self.build()[0]
        original = archive.read_bytes()
        for mutate in [
            lambda rows: rows.__setitem__(-1, (rows[-1][0], b'changed')),
            lambda rows: rows.append(rows[-1]),
            lambda rows: setattr(rows[-1][0], 'type', tarfile.SYMTYPE),
            lambda rows: (rows[-1][0].pax_headers.clear(), setattr(rows[-1][0], 'name', '../outside')),
        ]:
            archive.write_bytes(original)
            self.rewrite_archive(archive, mutate)
            with self.assertRaises(ValueError): plugins.verify_archive(archive, VERSION, COMMIT)
        archive.write_bytes(original)
        with self.assertRaises(ValueError): plugins.verify_archive(archive, VERSION, 'b' * 40)

    def test_self_consistent_runtime_change_is_rejected(self):
        archive = self.build()[0]
        def mutate(rows):
            bridge_index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith('/.mcp.json'))
            member, data = rows[bridge_index]
            changed = data.replace(b'"lite"', b'"full"')
            rows[bridge_index] = member, changed
            receipt_index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith(plugins.RECEIPT))
            member, data = rows[receipt_index]
            receipt = json.loads(data)
            receipt['files']['.mcp.json'] = {'size_bytes': len(changed), 'sha256': plugins.digest(changed)}
            rows[receipt_index] = member, plugins.json_bytes(receipt)
        self.rewrite_archive(archive, mutate)
        with self.assertRaises(ValueError): plugins.verify_archive(archive, VERSION, COMMIT)

    def test_coordinated_resource_and_receipt_change_cannot_keep_native_pack_hash(self):
        archive = self.build()[0]
        def mutate(rows):
            name = 'workflow/SKILL.md'
            target = plugins.SKILL_ROOT + 'SKILL.md'
            index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith('/' + target))
            member, data = rows[index]
            changed = data.replace(b'Use supplied evidence', b'Invent evidence')
            rows[index] = member, changed
            receipt_index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith(plugins.RECEIPT))
            member, data = rows[receipt_index]
            receipt = json.loads(data)
            fields = {'size_bytes': len(changed), 'sha256': plugins.digest(changed)}
            receipt['files'][target] = fields
            next(e for e in receipt['source']['entries'] if e['path'] == name).update(fields)
            # Even an updated embedded manifest must reproduce the externally
            # checked native pack hash, rather than merely echoing that hash.
            manifest = json.loads(receipt['source']['pack_manifest_json'])
            offset = 0
            for entry in manifest['entries']:
                if entry['path'] == name:
                    entry.update(fields)
                entry['payload_offset'] = offset
                offset += entry['size_bytes']
            receipt['source']['pack_manifest_json'] = json.dumps(manifest, separators=(',', ':'), sort_keys=True)
            rows[receipt_index] = member, plugins.json_bytes(receipt)
        self.rewrite_archive(archive, mutate)
        with self.assertRaisesRegex(ValueError, 'native pack digest'):
            plugins.verify_archive(archive, VERSION, COMMIT)

    def test_missing_payload_or_wrong_native_manifest_is_rejected(self):
        original = self.metadata['pack_manifest_json']
        for update in [dict(source_commit='e' * 40), dict(content_version='2.0.0-alpha.7'),
                       dict(content_root_sha256='e' * 64), dict(entries=[])]:
            manifest = json.loads(original)
            manifest.update(update)
            self.metadata['pack_manifest_json'] = json.dumps(manifest)
            self.write_metadata()
            with self.subTest(update=update), self.assertRaises(ValueError): self.build()

    def test_native_platforms_modes_and_wrong_binary(self):
        for target, data in BINARIES.items():
            self.binary.write_bytes(data)
            archive = self.build(target, target)[0]
            info = plugins.verify_archive(archive, VERSION, COMMIT)
            self.assertEqual(info['target'], target)
            with tarfile.open(archive) as packet:
                executable = next(m for m in packet if m.name.endswith('/' + plugins.binary_path(target)))
                self.assertEqual(executable.mode, 0o755)
            self.rewrite_archive(archive, lambda rows: setattr(
                next(m for m, _ in rows if m.name.endswith('/' + plugins.binary_path(target))), 'mode', 0o644))
            with self.assertRaisesRegex(ValueError, 'permissions'):
                plugins.verify_archive(archive, VERSION, COMMIT)
        with self.assertRaisesRegex(ValueError, 'architecture'):
            self.build('wrong-binary', TARGET)
        self.binary.unlink()
        with self.assertRaisesRegex(ValueError, 'regular file'):
            self.build('missing-binary')

    def test_legacy_npm_bridge_archive_still_verifies(self):
        files = plugins.project(self.content, 'codex', VERSION)
        source_names = ('.codex-plugin/plugin.json', '.claude-plugin/plugin.json')
        files[plugins.RECEIPT] = plugins.json_bytes({
            'schema_version': 1, 'platform': 'codex', 'source': self.metadata,
            'source_manifest_bytes': {n: self.content[n].decode() for n in source_names},
            'files': {n: {'size_bytes': len(d), 'sha256': plugins.digest(d)} for n, d in files.items()},
        })
        archive = self.root / plugins.archive_name('codex', VERSION)
        prefix = archive.name.removesuffix('.tar.gz') + '/plugins/qiongli-next/'
        with tarfile.open(archive, 'w:gz') as packet:
            for name, data in files.items():
                member = tarfile.TarInfo(prefix + name)
                member.size = len(data)
                packet.addfile(member, io.BytesIO(data))
        self.assertIsNone(plugins.verify_archive(archive, VERSION, COMMIT)['target'])

    def test_complete_platform_packet_binds_plugins_binaries_index_and_smoke(self):
        import shutil
        from tooling.scripts.native_cli_release import archive_cli
        from tooling.scripts.native_registry_packages import wheel
        from tooling.scripts.native_release_assets import assemble, verify
        tags = ['macosx_11_0_arm64', 'manylinux_2_35_x86_64', 'win_amd64']
        for (target, data), tag in zip(BINARIES.items(), tags):
            self.binary.write_bytes(data)
            folder = self.root / 'targets' / target
            folder.mkdir(parents=True)
            extension = 'zip' if target.endswith('msvc') else 'tar.gz'
            archive = folder / f'qiongli-{VERSION}-{target}.{extension}'
            archive_cli(archive, self.binary, b'fixture', target)
            whl = wheel(folder, '2.0.0a8', tag, data, 'fixture')
            archives = self.build('plugins-' + target, target)
            checks = {}
            for host, path in zip(plugins.PLATFORMS, archives):
                shutil.copyfile(path, folder / path.name)
                checks[host] = dict(plugins.verify_archive(path, VERSION, COMMIT),
                                    status='passed', runtime_path='empty', mcp_tools=14)
            receipt = {'version': VERSION, 'source_commit': COMMIT, 'target': target,
                       'checks': {'cli_mcp_tests': 'passed', 'cli_clippy': 'passed',
                                  'npm_wheel_local_install': 'passed',
                                  'archive_smoke': {'version': VERSION, 'content_pack_sha256': self.metadata['pack_sha256']},
                                  'marketplace_plugins': checks},
                       'artifacts': [{'file': p.name, 'sha256': plugins.digest(p.read_bytes()),
                                      'bytes': p.stat().st_size} for p in folder.iterdir()]}
            (folder / 'release-manifest.json').write_text(json.dumps(receipt))
        assets = self.root / 'combined/assets'
        assemble(self.root / 'targets', assets, VERSION, COMMIT)
        packet, _, _ = verify(assets, VERSION, COMMIT)
        self.assertEqual(len(packet['artifacts']), 14)
        index = json.loads((assets / 'marketplace-plugins.json').read_text())
        self.assertEqual(len(index['plugins']), 6)
        self.assertEqual({p['name'] for p in index['plugins']}, {plugins.plugin_name(t) for t in BINARIES})
        manifest = assets / 'release-manifest.json'
        cli_archive = assets / f'qiongli-{VERSION}-{TARGET}.tar.gz'
        original = cli_archive.read_bytes()
        self.rewrite_archive(cli_archive, lambda rows: rows.__setitem__(0, (rows[0][0], BINARIES[TARGET] + b'changed')))
        changed_packet = json.loads(json.dumps(packet))
        entry = next(p for p in changed_packet['artifacts'] if p['file'] == cli_archive.name)
        entry.update(sha256=plugins.digest(cli_archive.read_bytes()), bytes=cli_archive.stat().st_size)
        manifest.write_text(json.dumps(changed_packet))
        with self.assertRaisesRegex(ValueError, 'CLI executable differs'):
            verify(assets, VERSION, COMMIT)
        cli_archive.write_bytes(original)
        for change, error in [
            (lambda p: p['target_evidence'][0]['checks']['marketplace_plugins']['codex'].update(runtime_path='inherited'), 'smoke evidence'),
            (lambda p: p['artifacts'].remove(next(a for a in p['artifacts'] if '-codex-plugin-' in a['file'])), 'all six'),
            (lambda p: p.update(artifacts=[a for a in p['artifacts'] if '-plugin-' not in a['file']]), 'all six'),
        ]:
            modified = json.loads(json.dumps(packet))
            change(modified)
            manifest.write_text(json.dumps(modified))
            with self.assertRaisesRegex(ValueError, error): verify(assets, VERSION, COMMIT)
        manifest.write_text(json.dumps(packet))
        target_index = assets / 'marketplace-plugins.json'
        index['plugins'][0]['plugin_path'] = 'plugins/wrong-platform'
        target_index.write_text(json.dumps(index))
        entry = next(p for p in packet['artifacts'] if p['file'] == target_index.name)
        entry.update(sha256=plugins.digest(target_index.read_bytes()), bytes=target_index.stat().st_size)
        manifest.write_text(json.dumps(packet))
        with self.assertRaisesRegex(ValueError, 'platform index mismatch'):
            verify(assets, VERSION, COMMIT)
        # Even self-consistent archive hashes cannot substitute another executable.
        packet = json.loads(manifest.read_text())
        changed_archive = assets / plugins.archive_name('codex', VERSION, TARGET)
        def mutate(rows):
            path = plugins.binary_path(TARGET)
            binary_index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith('/' + path))
            member, data = rows[binary_index]
            changed = data + b'foreign executable'
            rows[binary_index] = member, changed
            receipt_index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith(plugins.RECEIPT))
            member, data = rows[receipt_index]
            receipt = json.loads(data)
            receipt['files'][path] = {'size_bytes': len(changed), 'sha256': plugins.digest(changed)}
            rows[receipt_index] = member, plugins.json_bytes(receipt)
        self.rewrite_archive(changed_archive, mutate)
        entry = next(p for p in packet['artifacts'] if p['file'] == changed_archive.name)
        entry.update(sha256=plugins.digest(changed_archive.read_bytes()), bytes=changed_archive.stat().st_size)
        manifest.write_text(json.dumps(packet))
        with self.assertRaisesRegex(ValueError, 'executable differs'):
            verify(assets, VERSION, COMMIT)

    def test_bridge_windows_shell_has_only_fixed_tokens_and_preserves_exit(self):
        harness = r'''
import vm from 'node:vm';
const source = JSON.parse(process.argv[1]).replace("import { spawn } from 'node:child_process';", '');
const rows = [];
for (const platform of ['linux', 'win32']) {
  const events = {};
  const fakeProcess = {platform, argv: ['node', 'bridge', '; injected'], on() {}, exitCode: 0};
  const spawn = (...args) => {rows.push(args); return {on(name, fn) {events[name] = fn;}, kill() {}};};
  vm.runInNewContext(source, {process: fakeProcess, spawn, console});
  events.close(7, null);
  if (fakeProcess.exitCode !== 7) throw new Error('exit code lost');
}
console.log(JSON.stringify(rows));
'''
        result = subprocess.run(['node', '--input-type=module', '--eval', harness,
                                 json.dumps(plugins.bridge(VERSION).decode())], check=True, capture_output=True, text=True)
        linux, windows = json.loads(result.stdout)
        args = ['--yes', f'qiongli@{VERSION}', 'mcp', 'serve', '--profile', 'lite', '--transport', 'stdio']
        self.assertEqual(linux, ['npx', args, {'stdio': 'inherit', 'shell': False}])
        self.assertEqual(windows, ['npx.cmd ' + ' '.join(args), [], {'stdio': 'inherit', 'shell': True}])
        self.assertNotIn('injected', result.stdout)


if __name__ == '__main__':
    unittest.main()
