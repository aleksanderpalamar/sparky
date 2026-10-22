from pathlib import Path
import re
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[1]


class AuthenticationDependenciesTest(unittest.TestCase):
    def setUp(self):
        manifest = ROOT / "crates/sparky-compositor/Cargo.toml"
        self.metadata = tomllib.loads(manifest.read_text())["package"]["metadata"]

    def test_arch_usa_polkit_do_sistema_e_empacota_o_agente_proprio(self):
        pkgbuild = (ROOT / "packaging/arch/PKGBUILD").read_text()
        dependencies = re.search(r"^depends=\((.*?)^\)", pkgbuild, re.M | re.S)
        self.assertIsNotNone(dependencies)
        self.assertIn("'polkit'", dependencies.group(1))
        self.assertNotIn("polkit-gnome", pkgbuild)
        self.assertIn("polkit-sparky; do", pkgbuild)

    def test_deb_exige_biblioteca_e_servico_polkit_sem_interface_estrangeira(self):
        dependencies = self.metadata["deb"]["depends"].split(", ")
        self.assertIn("polkitd", dependencies)
        self.assertIn("libpolkit-agent-1-0", dependencies)
        self.assertNotIn("policykit-1-gnome", self.metadata["deb"]["depends"])
        self.assertIn("target/release/polkit-sparky", [asset[0] for asset in self.metadata["deb"]["assets"]])

    def test_rpm_usa_bibliotecas_do_sistema_e_empacota_o_agente_proprio(self):
        metadata = self.metadata["generate-rpm"]
        self.assertIn("polkit", metadata["requires"])
        self.assertIn("polkit-libs", metadata["requires"])
        self.assertNotIn("polkit-gnome", metadata["requires"])
        self.assertIn("target/release/polkit-sparky", [asset["source"] for asset in metadata["assets"]])

    def test_build_portavel_confere_glibc_do_agente_proprio(self):
        source = (ROOT / "packaging/build.sh").read_text()
        self.assertRegex(source, r"binaries=\([^\n]*\bpolkit-sparky\)")


if __name__ == "__main__":
    unittest.main()
