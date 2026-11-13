from pathlib import Path
import re
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]


class FilesActionsDependenciesTest(unittest.TestCase):
    def test_todos_formatos_incluem_compactador_e_cliente_de_email(self):
        metadata = tomllib.loads((ROOT / "crates/sparky-compositor/Cargo.toml").read_text())["package"]["metadata"]
        pkgbuild = (ROOT / "packaging/arch/PKGBUILD").read_text()
        arch = re.search(r"^depends=\((.*?)^\)", pkgbuild, re.M | re.S).group(1).split()
        for tool in ["tar", "xdg-utils"]:
            self.assertIn(tool, metadata["deb"]["depends"].split(", "))
            self.assertIn(tool, metadata["generate-rpm"]["requires"])
            self.assertIn(f"'{tool}'", arch)

    def test_todos_formatos_incluem_o_bsdtar_para_extrair(self):
        metadata = tomllib.loads((ROOT / "crates/sparky-compositor/Cargo.toml").read_text())["package"]["metadata"]
        pkgbuild = (ROOT / "packaging/arch/PKGBUILD").read_text()
        arch = re.search(r"^depends=\((.*?)^\)", pkgbuild, re.M | re.S).group(1).split()
        self.assertIn("libarchive-tools", metadata["deb"]["depends"].split(", "))
        self.assertIn("bsdtar", metadata["generate-rpm"]["requires"])
        self.assertIn("'libarchive'", arch)

    def test_todos_formatos_sobem_o_file_manager1_so_na_sessao_sparky(self):
        entrada = (ROOT / "data/autostart/sparky-files-manager.desktop").read_text()
        self.assertIn("Exec=sparky-files --file-manager1", entrada)
        self.assertIn("OnlyShowIn=sparky;", entrada)
        cargo = (ROOT / "crates/sparky-compositor/Cargo.toml").read_text()
        pkgbuild = (ROOT / "packaging/arch/PKGBUILD").read_text()
        self.assertIn('"etc/xdg/autostart/"', cargo)
        self.assertIn('dest = "/etc/xdg/autostart/sparky-files-manager.desktop"', cargo)
        self.assertIn('data/autostart/sparky-files-manager.desktop -t "$pkgdir/etc/xdg/autostart"', pkgbuild)


if __name__ == "__main__":
    unittest.main()
