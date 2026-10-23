import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from ci.release_artifacts import prepare, package_names


class ReleaseArtifactsTest(unittest.TestCase):
    def setUp(self):
        metadata = patch("ci.release_artifacts.verify_package")
        self.metadata = metadata.start()
        self.addCleanup(metadata.stop)
        temporary = tempfile.TemporaryDirectory(prefix="sparky-release-artifacts-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.names = package_names("0.22.0", "1")
        for name in self.names:
            (self.root / name).write_bytes(b"package fixture")

    def test_requires_main_and_wallpapers_in_all_three_formats(self):
        self.assertEqual(len(self.names), 6)
        self.assertEqual(sum(name.endswith(".deb") for name in self.names), 2)
        self.assertEqual(sum(name.endswith(".rpm") for name in self.names), 2)
        self.assertEqual(sum(name.endswith(".pkg.tar.zst") for name in self.names), 2)

    def test_writes_checksums_and_commit_provenance(self):
        prepare(self.root, "0.22.0", "1", "a" * 40)
        manifest = (self.root / "SHA256SUMS").read_text()
        digest = hashlib.sha256(b"package fixture").hexdigest()
        self.assertEqual(len(manifest.splitlines()), 6)
        self.assertTrue(all(line.startswith(digest + "  ") for line in manifest.splitlines()))
        metadata = json.loads((self.root / "release.json").read_text())
        self.assertEqual(metadata["tag"], "v0.22.0")
        self.assertEqual(metadata["commit"], "a" * 40)
        self.assertEqual(set(metadata["sha256"]), self.names)

    def test_missing_empty_stale_or_unexpected_package_blocks_release(self):
        cases = ["missing", "empty", "stale", "unexpected", "symlink"]
        for case in cases:
            with self.subTest(case=case), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                for name in self.names:
                    (root / name).write_bytes(b"fixture")
                first = root / sorted(self.names)[0]
                if case == "missing":
                    first.unlink()
                if case == "empty":
                    first.write_bytes(b"")
                if case == "stale":
                    first.rename(root / first.name.replace("0.22.0", "0.21.0"))
                if case == "unexpected":
                    (root / "unreviewed.txt").touch()
                if case == "symlink":
                    first.unlink()
                    first.symlink_to(next(root.iterdir()))
                with self.assertRaises(ValueError):
                    prepare(root, "0.22.0", "1", "a" * 40)
                self.assertFalse((root / "SHA256SUMS").exists())

    def test_includes_optional_arch_debug_symbols(self):
        debug = "sparky-debug-0.22.0-1-x86_64.pkg.tar.zst"
        (self.root / debug).write_bytes(b"debug fixture")
        prepare(self.root, "0.22.0", "1", "a" * 40)
        self.assertIn(debug, (self.root / "SHA256SUMS").read_text())

    def test_rejects_invalid_commit(self):
        with self.assertRaises(ValueError):
            prepare(self.root, "0.22.0", "1", "not-a-commit")

    def test_metadata_failure_prevents_creating_a_manifest(self):
        self.metadata.side_effect = ValueError("versão interna incorreta")
        with self.assertRaises(ValueError):
            prepare(self.root, "0.22.0", "1", "a" * 40)
        self.assertFalse((self.root / "SHA256SUMS").exists())
