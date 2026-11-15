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

    def test_todos_formatos_instalam_o_portal_de_escolha_de_arquivos(self):
        portal = (ROOT / "data/portals/sparky.portal").read_text()
        self.assertIn("DBusName=org.freedesktop.impl.portal.desktop.sparky", portal)
        self.assertIn("Interfaces=org.freedesktop.impl.portal.FileChooser", portal)
        self.assertIn("UseIn=sparky", portal)
        servico = (ROOT / "data/dbus/org.freedesktop.impl.portal.desktop.sparky.service").read_text()
        self.assertIn("Name=org.freedesktop.impl.portal.desktop.sparky", servico)
        self.assertIn("Exec=/usr/bin/sparky-files --file-chooser", servico)
        self.assertIn("SystemdService=xdg-desktop-portal-sparky.service", servico)
        unidade = (ROOT / "data/session/xdg-desktop-portal-sparky.service").read_text()
        self.assertIn("PartOf=graphical-session.target", unidade)
        self.assertIn("BusName=org.freedesktop.impl.portal.desktop.sparky", unidade)
        self.assertIn("ExecStart=/usr/bin/sparky-files --file-chooser", unidade)
        preferidos = (ROOT / "data/portals/sparky-portals.conf").read_text()
        self.assertIn("org.freedesktop.impl.portal.FileChooser=sparky", preferidos)
        cargo = (ROOT / "crates/sparky-compositor/Cargo.toml").read_text()
        pkgbuild = (ROOT / "packaging/arch/PKGBUILD").read_text()
        self.assertIn('dest = "/usr/share/xdg-desktop-portal/portals/sparky.portal"', cargo)
        self.assertIn('dest = "/usr/share/dbus-1/services/org.freedesktop.impl.portal.desktop.sparky.service"', cargo)
        self.assertIn('"usr/share/dbus-1/services/"', cargo)
        self.assertIn('data/portals/sparky.portal -t "$pkgdir/usr/share/xdg-desktop-portal/portals"', pkgbuild)
        self.assertIn('-t "$pkgdir/usr/share/dbus-1/services"', pkgbuild)
        self.assertIn('dest = "/usr/lib/systemd/user/xdg-desktop-portal-sparky.service"', cargo)
        self.assertIn('data/session/xdg-desktop-portal-sparky.service -t "$pkgdir/usr/lib/systemd/user"', pkgbuild)


if __name__ == "__main__":
    unittest.main()
