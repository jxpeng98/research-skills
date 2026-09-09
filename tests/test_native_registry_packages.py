from __future__ import annotations

import base64
import csv
import hashlib
import io
from pathlib import Path
import tempfile
import tarfile
import tomllib
import unittest
import zipfile

from tooling.scripts import native_registry_packages as packages


class NativeRegistryPackagesTests(unittest.TestCase):
    def test_three_platform_npm_and_windows_wheel(self):
        binary_data = {
            'aarch64-apple-darwin': bytes.fromhex('cffaedfe0c000001') + bytes(100),
            'x86_64-unknown-linux-gnu': b'\x7fELF\x02\x01' + bytes(12) + b'\x3e\x00' + bytes(80),
            'x86_64-pc-windows-msvc': b'MZ' + bytes(58) + (64).to_bytes(4, 'little') + b'PE\x00\x00\x64\x86',
        }
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binaries = {}
            for target, data in binary_data.items():
                path = root / target
                path.write_bytes(data)
                binaries[target] = path
                packages.validate_binary(data, target)
                with self.assertRaises(ValueError):
                    packages.validate_binary(b'invalid', target)
            tarball = packages.npm_package(root / 'npm-out', binaries, '2.0.0-beta.1')
            with tarfile.open(tarball) as archive:
                import json
                manifest = json.load(archive.extractfile('package/package.json'))
                self.assertEqual(manifest['version'], '2.0.0-beta.1')
                self.assertEqual(manifest['publishConfig']['tag'], 'next')
                self.assertEqual(set(manifest['os']), {'darwin', 'linux', 'win32'})
                for target, data in binary_data.items():
                    self.assertEqual(archive.extractfile(f'package/native/{target}/{packages.TARGETS[target][2]}').read(), data)
            whl = packages.wheel(root, '2.0.0b1', 'win_amd64', binary_data['x86_64-pc-windows-msvc'], 'test')
            with zipfile.ZipFile(whl) as archive:
                self.assertIn('qiongli_native/bin/qiongli.exe', archive.namelist())
                self.assertIn(b'Version: 2.0.0b1', archive.read('qiongli-2.0.0b1.dist-info/METADATA'))

    def test_source_assets_and_wheel_record_preserve_the_native_inputs(self):
        version = tomllib.loads((packages.NATIVE / 'Cargo.toml').read_text())['workspace']['package']['version']
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = packages.stage_cargo(root, version)
            manifest = tomllib.loads((source / 'apps/qiongli/Cargo.toml').read_text())
            self.assertEqual(manifest['dependencies']['qiongli-content']['version'], f'={version}')
            self.assertEqual((source / 'apps/qiongli/package-assets/content/workflow/SKILL.md').read_bytes(), (packages.ROOT / 'content/workflow/SKILL.md').read_bytes())
            self.assertTrue((source / 'apps/qiongli/package-assets/qiongli-zotero-companion/manifest.json').is_file())
            wheel = packages.wheel(root, '2.0.0a6', 'macosx_11_0_arm64', b'native-bytes', 'Test package')
            with zipfile.ZipFile(wheel) as archive:
                records = csv.reader(io.StringIO(archive.read('qiongli-2.0.0a6.dist-info/RECORD').decode()))
                for name, digest, size in records:
                    if not digest:
                        self.assertTrue(name.endswith('/RECORD'))
                        continue
                    data = archive.read(name)
                    self.assertEqual(int(size), len(data))
                    self.assertEqual(digest, 'sha256=' + base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b'=').decode())
                self.assertEqual(archive.getinfo('qiongli_native/bin/qiongli').external_attr >> 16 & 0o777, 0o755)
            (root / 'link').symlink_to(wheel)
            with self.assertRaises(ValueError):
                packages.regular_bytes(root / 'link')
            with self.assertRaises(ValueError):
                packages.binary_packages(root, wheel, version)


if __name__ == '__main__':
    unittest.main()
