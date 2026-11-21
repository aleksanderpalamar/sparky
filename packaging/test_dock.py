from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]


class DockPackagingTest(unittest.TestCase):
    def test_todos_formatos_incluem_o_dock(self):
        metadata = tomllib.loads((ROOT / "crates/sparky-compositor/Cargo.toml").read_text())["package"]["metadata"]
        self.assertIn("target/release/sparky-dock", [item[0] for item in metadata["deb"]["assets"]])
        self.assertIn("target/release/sparky-dock", [item["source"] for item in metadata["generate-rpm"]["assets"]])
        for path in ["packaging/arch/PKGBUILD", "packaging/build.sh"]:
            self.assertIn("sparky-dock", (ROOT / path).read_text())

    def test_traducoes_do_dock_existem_nos_dois_idiomas(self):
        for language in ["pt-BR", "en"]:
            self.assertTrue((ROOT / f"data/i18n/{language}/dock.ftl").is_file())


if __name__ == "__main__":
    unittest.main()
