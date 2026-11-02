from pathlib import Path
import re
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[1]
COMMON = {
    "data/login/sparky-login.service": "usr/lib/systemd/system/sparky-login.service",
    "data/login/sparky-login.sysusers": "usr/lib/sysusers.d/sparky-login.conf",
    "data/login/sparky-login.tmpfiles": "usr/lib/tmpfiles.d/sparky-login.conf",
    "data/login/sparky-login-greeter.pam": "etc/pam.d/sparky-login-greeter",
}


class LoginPackagingTest(unittest.TestCase):
    def setUp(self):
        manifest = ROOT / "crates/sparky-compositor/Cargo.toml"
        self.metadata = tomllib.loads(manifest.read_text())["package"]["metadata"]
        self.pkgbuild = (ROOT / "packaging/arch/PKGBUILD").read_text()

    def deb_assets(self):
        return {source.removeprefix("../../"): dest for source, dest, _ in self.metadata["deb"]["assets"]}

    def rpm_assets(self):
        assets = self.metadata["generate-rpm"]["assets"]
        return {asset["source"].removeprefix("../../"): asset["dest"].lstrip("/") for asset in assets}

    def test_deb_leva_servico_usuario_pastas_e_o_pam_do_debian(self):
        assets = self.deb_assets()
        for source, dest in {**COMMON, "packaging/pam/debian/sparky-login": "etc/pam.d/sparky-login"}.items():
            self.assertEqual(assets.get(source), dest, source)
        self.assertEqual(assets.get("target/release/sparky-login"), "usr/bin/")

    def test_rpm_leva_servico_usuario_pastas_e_o_pam_do_fedora(self):
        assets = self.rpm_assets()
        for source, dest in {**COMMON, "packaging/pam/fedora/sparky-login": "etc/pam.d/sparky-login"}.items():
            self.assertEqual(assets.get(source), dest, source)
        self.assertEqual(assets.get("target/release/sparky-login"), "usr/bin/sparky-login")

    def test_arch_instala_os_mesmos_arquivos_com_o_pam_do_arch(self):
        for source, dest in {**COMMON, "packaging/pam/arch/sparky-login": "etc/pam.d/sparky-login"}.items():
            self.assertIn(f'install -Dm644 {source} "$pkgdir/{dest}"', self.pkgbuild)
        self.assertIn("sparky-login sparky-login-greeter", self.pkgbuild)
        backup = re.search(r"^backup=\((.*?)\)", self.pkgbuild, re.M)
        self.assertIsNotNone(backup)
        self.assertIn("'etc/pam.d/sparky-login'", backup.group(1))
        self.assertIn("'etc/pam.d/sparky-login-greeter'", backup.group(1))

    def test_os_tres_formatos_dependem_do_pam(self):
        dependencies = re.search(r"^depends=\((.*?)^\)", self.pkgbuild, re.M | re.S)
        self.assertIn("'pam'", dependencies.group(1))
        self.assertIn("libpam0g", self.metadata["deb"]["depends"].split(", "))
        self.assertIn("pam", self.metadata["generate-rpm"]["requires"])

    def test_deb_e_rpm_criam_usuario_e_pastas_sem_habilitar_o_servico(self):
        postinst = (ROOT / "packaging/debian/postinst").read_text()
        rpm_post = self.metadata["generate-rpm"]["post_install_script"]
        for script in (postinst, rpm_post):
            self.assertIn("systemd-sysusers sparky-login.conf", script)
            self.assertIn("systemd-tmpfiles --create sparky-login.conf", script)
            self.assertNotIn("enable", script)
        self.assertEqual(self.metadata["deb"]["maintainer-scripts"], "../../packaging/debian/")
        self.assertNotIn("systemd-units", self.metadata["deb"])

    def test_o_servico_ocupa_o_terminal_1_e_vale_como_gerenciador_de_login(self):
        unit = (ROOT / "data/login/sparky-login.service").read_text()
        for line in ["ExecStart=/usr/bin/sparky-login", "Conflicts=getty@tty1.service", "Alias=display-manager.service", "StateDirectory=sparky-login", "RuntimeDirectory=sparky-login"]:
            self.assertIn(line, unit)

    def test_o_pam_proprio_nao_liga_o_gnome_keyring(self):
        for distro in ["arch", "debian", "fedora"]:
            text = (ROOT / f"packaging/pam/{distro}/sparky-login").read_text()
            self.assertNotIn("pam_gnome_keyring", text, distro)


if __name__ == "__main__":
    unittest.main()
