import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tooling.scripts.native_cli_release import archive_cli
from tooling.scripts.native_registry_packages import TARGETS, wheel
from tooling.scripts.native_release_assets import assemble, verify


class NativeReleaseAssetsTests(unittest.TestCase):
    def test_assembly_requires_one_source_and_refuses_modified_assets(self):
        version, commit = '2.0.0-alpha.7', 'a' * 40
        fixtures = {
            'aarch64-apple-darwin': (bytes.fromhex('cffaedfe0c000001') + bytes(100), 'macosx_11_0_arm64'),
            'x86_64-unknown-linux-gnu': (b'\x7fELF\x02\x01' + bytes(12) + b'\x3e\x00' + bytes(80), 'manylinux_2_35_x86_64'),
            'x86_64-pc-windows-msvc': (b'MZ' + bytes(58) + (64).to_bytes(4, 'little') + b'PE\x00\x00\x64\x86', 'win_amd64'),
        }
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for target, (data, tag) in fixtures.items():
                folder = root / 'targets' / target
                folder.mkdir(parents=True)
                binary = root / target
                binary.write_bytes(data)
                extension = 'zip' if target.endswith('msvc') else 'tar.gz'
                archive = folder / f'qiongli-{version}-{target}.{extension}'
                archive_cli(archive, binary, b'fixture', target)
                whl = wheel(folder, '2.0.0a7', tag, data, 'fixture')
                receipt = {'version': version, 'source_commit': commit, 'target': target,
                           'checks': {'cli_mcp_tests': 'passed', 'cli_clippy': 'passed',
                                      'npm_wheel_local_install': 'passed', 'archive_smoke': {'version': version}},
                           'artifacts': [{'file': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest(),
                                          'bytes': p.stat().st_size} for p in (archive, whl)]}
                (folder / 'release-manifest.json').write_text(json.dumps(receipt))
            with self.assertRaisesRegex(ValueError, 'mixed version or source'):
                assemble(root / 'targets', root / 'wrong-source', version, 'b' * 40)
            assemble(root / 'targets', root / 'assets', version, commit)
            manifest, npm, wheels = verify(root / 'assets', version, commit)
            with patch('tooling.scripts.native_release_assets.NPM_INSTALL_REVIEW', 'unexpected script'):
                with self.assertRaisesRegex(ValueError, 'installation review bytes'):
                    verify(root / 'assets', version, commit)
            self.assertEqual(set(wheels), set(TARGETS))
            self.assertEqual(len(manifest['artifacts']), 7)
            # The Plugin verifier owns archive internals; this owner must bind
            # both Plugin archives to every native executable's observed pack.
            pack_hash = 'b' * 64
            for target in fixtures:
                folder = root / 'targets' / target
                receipt = json.loads((folder / 'release-manifest.json').read_text())
                receipt['checks']['archive_smoke']['content_pack_sha256'] = pack_hash
                if target.endswith('linux-gnu'):
                    for host in ('codex', 'claude'):
                        plugin = folder / f'qiongli-next-{host}-plugin-v{version}.tar.gz'
                        plugin.write_bytes(host.encode())
                        receipt['artifacts'].append({'file': plugin.name,
                            'sha256': hashlib.sha256(plugin.read_bytes()).hexdigest(),
                            'bytes': plugin.stat().st_size})
                (folder / 'release-manifest.json').write_text(json.dumps(receipt))
            plugin_assets = root / 'with-plugins/assets'
            with patch('tooling.scripts.native_release_assets.verify_archive',
                       return_value={'pack_sha256': pack_hash}) as verify_plugin:
                assemble(root / 'targets', plugin_assets, version, commit)
                packet, _, _ = verify(plugin_assets, version, commit)
                self.assertEqual(len(packet['artifacts']), 9)
                verify_plugin.return_value = {'pack_sha256': 'c' * 64}
                with self.assertRaisesRegex(ValueError, 'differs from CLI'):
                    verify(plugin_assets, version, commit)
                verify_plugin.return_value = {'pack_sha256': pack_hash}
                packet['target_evidence'][0]['checks']['archive_smoke']['content_pack_sha256'] = 'c' * 64
                (plugin_assets / 'release-manifest.json').write_text(json.dumps(packet))
                with self.assertRaisesRegex(ValueError, 'all three CLI content packs'):
                    verify(plugin_assets, version, commit)
                packet['artifacts'] = [a for a in packet['artifacts'] if '-claude-plugin-' not in a['file']]
                (plugin_assets / 'release-manifest.json').write_text(json.dumps(packet))
                with self.assertRaisesRegex(ValueError, 'both marketplace Plugin archives'):
                    verify(plugin_assets, version, commit)
            original = npm.read_bytes()
            npm.write_bytes(original + b'tamper')
            with self.assertRaisesRegex(ValueError, 'digest/size mismatch'):
                verify(root / 'assets', version, commit)
            npm.write_bytes(original)
            manifest['target_evidence'][1] = manifest['target_evidence'][0]
            (root / 'assets/release-manifest.json').write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, 'duplicate or missing'):
                verify(root / 'assets', version, commit)


if __name__ == '__main__':
    unittest.main()
