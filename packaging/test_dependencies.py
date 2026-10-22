from pathlib import Path
import re
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[1]


class AuthenticationDependenciesTest(unittest.TestCase):
    def setUp(self):
        manifest = ROOT / "crates/sparky-compositor/Cargo.toml"
        self.metadata = tomllib.loads(manifest.read_text())["package"]["metadata"]

    def test_arch_instala_o_agente_grafico_em_vez_de_apenas_sugeri_lo(self):
        pkgbuild = (ROOT / "packaging/arch/PKGBUILD").read_text()
        dependencies = re.search(r"^depends=\((.*?)^\)", pkgbuild, re.M | re.S)
        self.assertIsNotNone(dependencies)
        self.assertIn("'polkit-gnome'", dependencies.group(1))

    def test_deb_exige_um_dos_agentes_que_a_sessao_sabe_abrir(self):
        dependencies = self.metadata["deb"]["depends"].split(", ")
        self.assertIn("policykit-1-gnome | mate-polkit | lxpolkit", dependencies)

    def test_rpm_instala_o_agente_grafico_em_vez_de_apenas_recomenda_lo(self):
        self.assertIn("polkit-gnome", self.metadata["generate-rpm"]["requires"])


if __name__ == "__main__":
    unittest.main()
