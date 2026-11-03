from pathlib import Path
import re
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[1]


class LockPackagingTest(unittest.TestCase):
    def setUp(self):
        manifest = ROOT / "crates/sparky-compositor/Cargo.toml"
        self.metadata = tomllib.loads(manifest.read_text())["package"]["metadata"]
        self.pkgbuild = (ROOT / "packaging/arch/PKGBUILD").read_text()

    def test_deb_leva_o_bloqueio_com_o_pam_do_debian(self):
        assets = {source.removeprefix("../../"): dest for source, dest, _ in self.metadata["deb"]["assets"]}
        self.assertEqual(assets.get("target/release/sparky-lock"), "usr/bin/")
        self.assertEqual(assets.get("packaging/pam/debian/sparky-lock"), "etc/pam.d/sparky-lock")

    def test_rpm_leva_o_bloqueio_com_o_pam_do_fedora(self):
        assets = {asset["source"].removeprefix("../../"): asset for asset in self.metadata["generate-rpm"]["assets"]}
        self.assertEqual(assets["target/release/sparky-lock"]["dest"], "/usr/bin/sparky-lock")
        pam = assets["packaging/pam/fedora/sparky-lock"]
        self.assertEqual(pam["dest"], "/etc/pam.d/sparky-lock")
        self.assertEqual(pam.get("config"), "noreplace")

    def test_arch_instala_o_bloqueio_com_o_pam_do_arch(self):
        self.assertIn('install -Dm644 packaging/pam/arch/sparky-lock "$pkgdir/etc/pam.d/sparky-lock"', self.pkgbuild)
        self.assertIn("sparky-keyring-prompt sparky-lock sparky-lockd sparky-login", self.pkgbuild)
        backup = re.search(r"^backup=\((.*?)\)", self.pkgbuild, re.M)
        self.assertIsNotNone(backup)
        self.assertIn("'etc/pam.d/sparky-lock'", backup.group(1))

    def test_o_swaylock_deixa_de_ser_sugerido(self):
        self.assertNotIn("swaylock", self.pkgbuild)
        self.assertNotIn("swaylock", self.metadata["deb"]["recommends"])
        self.assertNotIn("swaylock", self.metadata["generate-rpm"].get("recommends", {}))

    def test_o_guarda_do_bloqueio_vai_nos_tres_formatos(self):
        deb = {source.removeprefix("../../"): dest for source, dest, _ in self.metadata["deb"]["assets"]}
        self.assertEqual(deb.get("target/release/sparky-lockd"), "usr/bin/")
        rpm = {asset["source"].removeprefix("../../"): asset for asset in self.metadata["generate-rpm"]["assets"]}
        self.assertEqual(rpm["target/release/sparky-lockd"]["dest"], "/usr/bin/sparky-lockd")
        self.assertIn(" sparky-lock sparky-lockd ", self.pkgbuild)
        build = (ROOT / "packaging/build.sh").read_text()
        self.assertRegex(build, r"binaries=\([^\n]*\bsparky-lockd\b")

    def test_o_pam_do_bloqueio_so_confere_a_senha(self):
        for distro in ["arch", "debian", "fedora"]:
            lines = [line for line in (ROOT / f"packaging/pam/{distro}/sparky-lock").read_text().splitlines() if line and not line.startswith("#")]
            self.assertTrue(lines, distro)
            self.assertTrue(all("auth" in line for line in lines), distro)


if __name__ == "__main__":
    unittest.main()
