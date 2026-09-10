from pathlib import Path
import tarfile
import tempfile
import unittest
import zipfile
from unittest.mock import patch

from tooling.scripts.native_cli_release import archive_cli, archive_readme, check_windows_imports
from tooling.scripts.native_registry_packages import TARGETS


class NativeCliReleaseTests(unittest.TestCase):
    def test_windows_import_check_rejects_extra_runtimes_and_unreadable_tables(self):
        with patch('tooling.scripts.native_cli_release.shutil.which', return_value='llvm-objdump'), \
             patch('tooling.scripts.native_cli_release.subprocess.check_output') as inspect:
            inspect.return_value = '    DLL Name: KERNEL32.dll\n    DLL Name: api-ms-win-core-synch-l1-2-0.dll\n'
            self.assertEqual(len(check_windows_imports(Path('qiongli.exe'))), 2)
            for extra in ('VCRUNTIME140.dll', 'MSVCP140.dll', 'python312.dll', 'libssl-3-x64.dll'):
                inspect.return_value = f'DLL Name: kernel32.dll\nDLL Name: {extra}\n'
                with self.subTest(extra=extra), self.assertRaisesRegex(ValueError, 'only system DLLs'):
                    check_windows_imports(Path('qiongli.exe'))
            inspect.return_value = 'file format not recognized'
            with self.assertRaisesRegex(ValueError, 'unreadable import table'):
                check_windows_imports(Path('qiongli.exe'))

    def test_archive_preserves_executable_without_host_paths_or_links(self):
        for target, (_, _, executable) in TARGETS.items():
            with self.subTest(target=target), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                binary = root / 'candidate'
                binary.write_bytes(b'candidate executable')
                readme = archive_readme('2.0.0-beta.2', target, 'a' * 40)
                windows = target.endswith('msvc')
                path = root / ('cli.zip' if windows else 'cli.tar.gz')
                archive_cli(path, binary, readme, target)
                if windows:
                    with zipfile.ZipFile(path) as archive:
                        self.assertEqual(archive.namelist(), [executable, 'README.md', 'LICENSE'])
                        archive.extractall(root / 'installed')
                else:
                    with tarfile.open(path) as archive:
                        self.assertEqual(archive.getnames(), [executable, 'README.md', 'LICENSE'])
                        self.assertTrue(all(item.isfile() and item.uid == 0 for item in archive))
                        self.assertEqual(archive.getmember(executable).mode, 0o755)
                        archive.extractall(root / 'installed', filter='data')
                self.assertEqual((root / 'installed' / executable).read_bytes(), binary.read_bytes())
                self.assertEqual((root / 'installed/README.md').read_bytes(), readme)
                command = f'.\\{executable}' if windows else f'./{executable}'
                self.assertIn(f'{command} --version'.encode(), readme)
                self.assertIn(f'{command} content list'.encode(), readme)
                self.assertIn(target.encode(), readme)
                self.assertIn(b'v2.0.0-beta.2', readme)
                self.assertIn(('blob/' + 'a' * 40 + '/docs/guide/cli-2x.md').encode(), readme)
                self.assertNotIn(str(root).encode(), readme)
                link = root / 'linked-candidate'
                link.symlink_to(binary)
                with self.assertRaises(ValueError):
                    archive_cli(root / path.name.replace('cli.', 'linked.'), link, readme, target)


if __name__ == '__main__':
    unittest.main()
