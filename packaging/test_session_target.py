from pathlib import Path
import configparser
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[1]
UNIT = "data/session/sparky-session.target"


class SessionTargetTest(unittest.TestCase):
    def setUp(self):
        manifest = ROOT / "crates/sparky-compositor/Cargo.toml"
        self.metadata = tomllib.loads(manifest.read_text())["package"]["metadata"]
        self.pkgbuild = (ROOT / "packaging/arch/PKGBUILD").read_text()

    def test_o_alvo_prende_a_sessao_grafica_do_systemd(self):
        unit = configparser.ConfigParser(strict=False)
        unit.read(ROOT / UNIT)
        self.assertEqual(unit["Unit"]["BindsTo"], "graphical-session.target")
        self.assertEqual(unit["Unit"]["Wants"], "graphical-session-pre.target")
        self.assertEqual(unit["Unit"]["After"], "graphical-session-pre.target")

    def test_os_tres_formatos_instalam_o_alvo_do_usuario(self):
        deb = {source.removeprefix("../../"): dest for source, dest, _ in self.metadata["deb"]["assets"]}
        self.assertEqual(deb.get(UNIT), "usr/lib/systemd/user/")
        rpm = {asset["source"].removeprefix("../../"): asset for asset in self.metadata["generate-rpm"]["assets"]}
        self.assertEqual(rpm[UNIT]["dest"], "/usr/lib/systemd/user/sparky-session.target")
        self.assertIn(f'install -Dm644 {UNIT} -t "$pkgdir/usr/lib/systemd/user"', self.pkgbuild)


if __name__ == "__main__":
    unittest.main()
