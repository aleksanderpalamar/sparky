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


if __name__ == "__main__":
    unittest.main()
