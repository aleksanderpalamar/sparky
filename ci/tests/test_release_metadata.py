from pathlib import Path
import unittest
from unittest.mock import patch

from ci.release_metadata import read_metadata, verify_package


class ReleaseMetadataTest(unittest.TestCase):
    @patch("ci.release_metadata.subprocess.check_output")
    def test_reads_the_three_package_formats(self, output):
        for filename in ["package.deb", "package.rpm", "package.pkg.tar.zst"]:
            with self.subTest(filename=filename):
                output.return_value = "sparky\t0.22.0-1\tx86_64"
                if filename.endswith(".zst"):
                    output.return_value = "pkgname = sparky\npkgver = 0.22.0-1\narch = x86_64\ndepend = library\n"
                self.assertEqual(read_metadata(Path(filename)), ("sparky", "0.22.0-1", "x86_64"))

    @patch("ci.release_metadata.read_metadata")
    def test_accepts_wallpapers_and_debug_packages(self, metadata):
        for name, architecture in [("sparky-wallpapers", "any"), ("sparky-debug", "x86_64")]:
            metadata.return_value = (name, "0.22.0-1", architecture)
            verify_package(Path(name + ".pkg.tar.zst"), "0.22.0", "1")

    @patch("ci.release_metadata.read_metadata")
    def test_rejects_incorrect_name_version_revision_or_architecture(self, metadata):
        for result in [("other", "0.22.0-1", "amd64"), ("sparky", "0.21.0-1", "amd64"),
                       ("sparky", "0.22.0-2", "amd64"), ("sparky", "0.22.0-1", "aarch64"),
                       ("sparky", "0.22.0-1", "any")]:
            with self.subTest(result=result), self.assertRaises(ValueError):
                metadata.return_value = result
                verify_package(Path("sparky.deb"), "0.22.0", "1")

    @patch("ci.release_metadata.read_metadata")
    def test_requires_architecture_spelling_for_each_package_format(self, metadata):
        for filename, name, architecture in [("sparky.deb", "sparky", "x86_64"),
                                             ("sparky.rpm", "sparky", "amd64"),
                                             ("sparky-wallpapers.pkg.tar.zst", "sparky-wallpapers", "amd64")]:
            with self.subTest(filename=filename), self.assertRaises(ValueError):
                metadata.return_value = (name, "0.22.0-1", architecture)
                verify_package(Path(filename), "0.22.0", "1")
