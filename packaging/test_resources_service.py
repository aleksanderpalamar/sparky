from pathlib import Path
import configparser
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[1]
UNIT = "data/session/sparky-resources.service"


class ResourcesServiceTest(unittest.TestCase):
    def setUp(self):
        self.unit = configparser.ConfigParser(strict=False)
        self.unit.optionxform = str
        self.unit.read(ROOT / UNIT)
        manifest = ROOT / "crates/sparky-compositor/Cargo.toml"
        self.metadata = tomllib.loads(manifest.read_text())["package"]["metadata"]
        self.pkgbuild = (ROOT / "packaging/arch/PKGBUILD").read_text()

    def test_o_servico_vive_e_morre_com_a_sessao(self):
        self.assertEqual(self.unit["Unit"]["PartOf"], "sparky-session.target")
        target = configparser.ConfigParser(strict=False)
        target.read(ROOT / "data/session/sparky-session.target")
        self.assertIn("sparky-resources.service", target["Unit"]["Wants"].split())

    def test_o_sistema_impoe_limites_rigidos_e_reinicio(self):
        service = self.unit["Service"]
        self.assertEqual(service["ExecStart"], "/usr/bin/sparky-resources")
        self.assertEqual(service["Restart"], "on-failure")
        self.assertEqual(service["MemoryMax"], "64M")
        self.assertEqual(service["MemorySwapMax"], "0")
        self.assertEqual(service["Slice"], "background.slice")
        self.assertEqual(service["IOSchedulingClass"], "idle")

    def test_o_servico_nao_fala_com_a_rede_nem_ganha_privilegios(self):
        service = self.unit["Service"]
        self.assertEqual(service["RestrictAddressFamilies"], "AF_UNIX")
        self.assertEqual(service["NoNewPrivileges"], "yes")

    def test_os_tres_formatos_instalam_a_unit(self):
        deb = {source.removeprefix("../../"): dest for source, dest, _ in self.metadata["deb"]["assets"]}
        self.assertEqual(deb.get(UNIT), "usr/lib/systemd/user/")
        rpm = {asset["source"].removeprefix("../../"): asset for asset in self.metadata["generate-rpm"]["assets"]}
        self.assertEqual(rpm[UNIT]["dest"], "/usr/lib/systemd/user/sparky-resources.service")
        self.assertIn(f'install -Dm644 {UNIT} -t "$pkgdir/usr/lib/systemd/user"', self.pkgbuild)


if __name__ == "__main__":
    unittest.main()
