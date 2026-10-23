import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[1]

CARGO = """import json
import os
from pathlib import Path
import sys

arguments = sys.argv[1:]
with Path(os.environ["SPARKY_BUILD_LOG"]).open("a") as log:
    log.write(json.dumps(arguments) + "\\n")
if arguments[0] == "zigbuild":
    sys.exit(int(os.environ.get("SPARKY_COMPILE_EXIT", "0")))
if arguments[0] == "deb":
    sys.exit(0)
if arguments[0] != "generate-rpm":
    sys.exit(41)
package = arguments[arguments.index("--package") + 1]
manifest = Path(package) / "Cargo.toml"
if not manifest.is_file():
    print(f"manifesto inexistente: {manifest}", file=sys.stderr)
    sys.exit(42)
"""

OBJDUMP = """import os
print("GLIBC_" + os.environ.get("SPARKY_TEST_GLIBC", "2.36"))
"""


class BuildPackagesTest(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="sparky-packaging-")
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name)
        self.log = self.directory / "commands.jsonl"
        self.project = self.directory / "project"
        (self.project / "packaging/arch").mkdir(parents=True)
        (self.project / "packaging/build.sh").write_bytes((ROOT / "packaging/build.sh").read_bytes())
        (self.project / "packaging/arch/PKGBUILD").write_text("pkgrel=1\n")
        manifest = self.project / "crates/sparky-compositor/Cargo.toml"
        manifest.parent.mkdir(parents=True)
        manifest.touch()
        self.environment = os.environ.copy()
        self.environment["PATH"] = str(self.directory) + os.pathsep + os.environ["PATH"]
        self.environment["SPARKY_BUILD_LOG"] = str(self.log)
        self.executable("cargo", CARGO)
        self.executable("objdump", OBJDUMP)

    def executable(self, name, source):
        path = self.directory / name
        path.write_text(f"#!{sys.executable}\n" + source)
        path.chmod(0o755)

    def run_build(self, mode="rpm"):
        return subprocess.run(
            ["bash", str(self.project / "packaging/build.sh"), mode],
            cwd=self.directory,
            env=self.environment,
            capture_output=True,
            text=True,
            check=False,
        )

    def commands(self):
        return [json.loads(line) for line in self.log.read_text().splitlines()]

    def test_rpm_localiza_manifesto_do_crate_a_partir_de_outra_pasta(self):
        result = self.run_build()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([command[0] for command in self.commands()], ["zigbuild", "generate-rpm", "generate-rpm"])
        self.assertIn("--variant", self.commands()[2])
        self.assertEqual(self.commands()[2][self.commands()[2].index("--variant") + 1], "wallpapers")

    def test_compilacao_recusada_impede_geracao_do_rpm(self):
        self.environment["SPARKY_COMPILE_EXIT"] = "17"
        result = self.run_build()
        self.assertEqual(result.returncode, 17)
        self.assertEqual([command[0] for command in self.commands()], ["zigbuild"])

    def test_glibc_acima_do_limite_impede_geracao_do_rpm(self):
        self.environment["SPARKY_TEST_GLIBC"] = "2.37"
        result = self.run_build()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("acima de 2.36", result.stderr)
        self.assertEqual([command[0] for command in self.commands()], ["zigbuild"])

    def test_pkgrel_acima_de_um_chega_ao_deb_rpm_e_wallpapers(self):
        (self.project / "packaging/arch/PKGBUILD").write_text("pkgrel=7\n")
        for mode in ["deb", "rpm"]:
            with self.subTest(mode=mode):
                self.log.unlink(missing_ok=True)
                result = self.run_build(mode)
                self.assertEqual(result.returncode, 0, result.stderr)
                commands = self.commands()[1:]
                self.assertEqual(len(commands), 2)
                option = "--deb-revision" if mode == "deb" else "--set-metadata"
                for command in commands:
                    self.assertIn(option, command)
                    value = command[command.index(option) + 1]
                    revision = value if mode == "deb" else tomllib.loads(value)["release"]
                    self.assertEqual(revision, "7")

    def test_pkgrel_invalido_impede_compilacao_e_nao_executa_conteudo(self):
        for release in ["0", "01", "1\npkgrel=2", "$(touch injected)"]:
            with self.subTest(release=release):
                self.log.unlink(missing_ok=True)
                (self.project / "packaging/arch/PKGBUILD").write_text(f"pkgrel={release}\n")
                self.assertNotEqual(self.run_build().returncode, 0)
                self.assertFalse(self.log.exists())
                self.assertFalse((self.directory / "injected").exists())


if __name__ == "__main__":
    unittest.main()
