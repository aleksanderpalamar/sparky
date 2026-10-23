from pathlib import Path
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[1]


class MtpDependenciesTest(unittest.TestCase):
    def test_arch_disponibiliza_backend_mtp_e_biblioteca_gio(self):
        source = (ROOT / "packaging/arch/PKGBUILD").read_text()
        self.assertIn("'glib2'", source)
        self.assertIn("'gvfs-mtp:", source)

    def test_deb_disponibiliza_backend_e_exportacao_fuse(self):
        source = (ROOT / "crates/sparky-compositor/Cargo.toml").read_text()
        metadata = tomllib.loads(source)["package"]["metadata"]["deb"]
        self.assertIn("libglib2.0-0", metadata["depends"])
        self.assertIn("gvfs-backends", metadata["recommends"])
        self.assertIn("gvfs-fuse", metadata["recommends"])

    def test_rpm_disponibiliza_backend_e_exportacao_fuse(self):
        source = (ROOT / "crates/sparky-compositor/Cargo.toml").read_text()
        metadata = tomllib.loads(source)["package"]["metadata"]["generate-rpm"]
        self.assertIn("glib2", metadata["requires"])
        self.assertIn("gvfs-mtp", metadata["recommends"])
        self.assertIn("gvfs-fuse", metadata["recommends"])


if __name__ == "__main__":
    unittest.main()
