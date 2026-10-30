from pathlib import Path
import re
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
DESKTOP = "../../data/applications/sparky-monitor.desktop"


class NativeSystemMonitorTest(unittest.TestCase):
    def setUp(self):
        manifest = ROOT / "crates/sparky-compositor/Cargo.toml"
        self.metadata = tomllib.loads(manifest.read_text())["package"]["metadata"]

    def test_deb_e_rpm_incluem_executavel_e_atalho(self):
        deb = [item[0] for item in self.metadata["deb"]["assets"]]
        rpm = [item["source"] for item in self.metadata["generate-rpm"]["assets"]]
        for assets in [deb, rpm]:
            self.assertIn("target/release/sparky-monitor", assets)
            self.assertIn(DESKTOP, assets)

    def test_arch_e_build_portavel_compilam_o_monitor(self):
        pkgbuild = (ROOT / "packaging/arch/PKGBUILD").read_text()
        loop = re.search(r"for binary in ([^;]*); do", pkgbuild)
        self.assertIsNotNone(loop)
        self.assertIn("sparky-monitor", loop.group(1).split())
        source = (ROOT / "packaging/build.sh").read_text()
        binaries = re.search(r"^binaries=\((.*)\)$", source, re.M)
        self.assertIsNotNone(binaries)
        self.assertIn("sparky-monitor", binaries.group(1).split())

    def test_atalho_abre_o_monitor_nativo(self):
        desktop = (ROOT / "data/applications/sparky-monitor.desktop").read_text()
        self.assertIn("Exec=sparky-monitor\n", desktop)
        self.assertIn("Categories=System;Monitor;", desktop)


if __name__ == "__main__":
    unittest.main()
