from pathlib import Path
import configparser
import re
import tomllib
import unittest
import xml.etree.ElementTree as ElementTree


ROOT = Path(__file__).resolve().parents[1]
UNIT = "data/session/sparky-resources.service"
POLICY = "data/polkit/org.sparky.resources.policy"
HELPER = "/usr/lib/sparky/sparky-resources-helper"


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
        self.assertEqual(service["Type"], "notify")
        self.assertEqual(service["WatchdogSec"], "30")
        self.assertEqual(service["MemoryMax"], "64M")
        self.assertEqual(service["MemorySwapMax"], "0")
        self.assertEqual(service["Slice"], "background.slice")
        self.assertEqual(service["IOSchedulingClass"], "idle")

    def test_o_servico_so_abre_socket_local_e_ipv4_e_so_eleva_pelo_pkexec(self):
        service = self.unit["Service"]
        self.assertEqual(service["RestrictAddressFamilies"], "AF_UNIX AF_INET")
        self.assertNotEqual(service.get("NoNewPrivileges"), "yes")

    def test_a_politica_do_polkit_so_libera_o_helper_na_sessao_ativa(self):
        actions = ElementTree.parse(ROOT / POLICY).getroot().findall("action")
        self.assertEqual([action.get("id") for action in actions], ["org.sparky.resources.nice"])
        defaults = actions[0].find("defaults")
        self.assertEqual(defaults.findtext("allow_any"), "no")
        self.assertEqual(defaults.findtext("allow_inactive"), "no")
        self.assertEqual(defaults.findtext("allow_active"), "yes")
        annotations = {item.get("key"): item.text for item in actions[0].findall("annotate")}
        self.assertEqual(annotations, {"org.freedesktop.policykit.exec.path": HELPER})

    def test_os_tres_formatos_instalam_o_helper_e_a_politica(self):
        deb = {source.removeprefix("../../"): (dest, mode) for source, dest, mode in self.metadata["deb"]["assets"]}
        self.assertEqual(deb["target/release/sparky-resources-helper"], ("usr/lib/sparky/", "755"))
        self.assertEqual(deb[POLICY], ("usr/share/polkit-1/actions/", "644"))
        rpm = {asset["source"].removeprefix("../../"): asset for asset in self.metadata["generate-rpm"]["assets"]}
        self.assertEqual(rpm["target/release/sparky-resources-helper"]["dest"], HELPER)
        self.assertEqual(rpm["target/release/sparky-resources-helper"]["mode"], "755")
        self.assertEqual(rpm[POLICY]["dest"], "/usr/share/polkit-1/actions/org.sparky.resources.policy")
        self.assertIn('install -Dm755 target/release/sparky-resources-helper -t "$pkgdir/usr/lib/sparky"', self.pkgbuild)
        self.assertIn(f'install -Dm644 {POLICY} -t "$pkgdir/usr/share/polkit-1/actions"', self.pkgbuild)
        build = (ROOT / "packaging/build.sh").read_text()
        binaries = re.search(r"^binaries=\((.*)\)$", build, re.M)
        self.assertIn("sparky-resources-helper", binaries.group(1).split())

    def test_os_tres_formatos_instalam_a_unit(self):
        deb = {source.removeprefix("../../"): dest for source, dest, _ in self.metadata["deb"]["assets"]}
        self.assertEqual(deb.get(UNIT), "usr/lib/systemd/user/")
        rpm = {asset["source"].removeprefix("../../"): asset for asset in self.metadata["generate-rpm"]["assets"]}
        self.assertEqual(rpm[UNIT]["dest"], "/usr/lib/systemd/user/sparky-resources.service")
        self.assertIn(f'install -Dm644 {UNIT} -t "$pkgdir/usr/lib/systemd/user"', self.pkgbuild)


if __name__ == "__main__":
    unittest.main()
