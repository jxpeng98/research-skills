from pathlib import Path
import tarfile
import tempfile
import unittest

from tooling.scripts.native_cli_release import archive_cli


class NativeCliReleaseTests(unittest.TestCase):
    def test_archive_preserves_executable_without_host_paths_or_links(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / 'candidate'
            binary.write_bytes(b'candidate executable')
            archive_cli(root / 'cli.tar.gz', binary, b'Installation instructions')
            with tarfile.open(root / 'cli.tar.gz') as archive:
                self.assertEqual(archive.getnames(), ['qiongli', 'README.md', 'LICENSE'])
                self.assertTrue(all(item.isfile() and item.uid == 0 for item in archive))
                self.assertEqual(archive.getmember('qiongli').mode, 0o755)
                archive.extractall(root / 'installed', filter='data')
            self.assertEqual((root / 'installed/qiongli').read_bytes(), binary.read_bytes())
            link = root / 'linked-candidate'
            link.symlink_to(binary)
            with self.assertRaises(ValueError):
                archive_cli(root / 'linked.tar.gz', link, b'no')


if __name__ == '__main__':
    unittest.main()
