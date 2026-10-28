from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]


class NativeNotificationsTest(unittest.TestCase):
    def test_todos_formatos_incluem_daemon_nativo(self):
        metadata = tomllib.loads((ROOT / "crates/sparky-compositor/Cargo.toml").read_text())["package"]["metadata"]
        self.assertIn("target/release/sparky-notifications", [item[0] for item in metadata["deb"]["assets"]])
        self.assertIn("target/release/sparky-notifications", [item["source"] for item in metadata["generate-rpm"]["assets"]])
        self.assertNotIn("mako", metadata["deb"]["recommends"])
        self.assertNotIn("mako", metadata["generate-rpm"]["recommends"])
        for path in ["packaging/arch/PKGBUILD", "packaging/build.sh"]:
            self.assertIn("sparky-notifications", (ROOT / path).read_text())
        self.assertNotIn("mako", (ROOT / "packaging/arch/PKGBUILD").read_text())


if __name__ == "__main__":
    unittest.main()
