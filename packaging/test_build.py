import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
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
        self.environment = os.environ.copy()
        self.environment["PATH"] = str(self.directory) + os.pathsep + os.environ["PATH"]
        self.environment["SPARKY_BUILD_LOG"] = str(self.log)
        self.executable("cargo", CARGO)
        self.executable("objdump", OBJDUMP)

    def executable(self, name, source):
        path = self.directory / name
        path.write_text(f"#!{sys.executable}\n" + source)
        path.chmod(0o755)

    def run_rpm(self):
        return subprocess.run(
            ["bash", str(ROOT / "packaging/build.sh"), "rpm"],
            cwd=self.directory,
            env=self.environment,
            capture_output=True,
            text=True,
            check=False,
        )

    def commands(self):
        return [json.loads(line) for line in self.log.read_text().splitlines()]

    def test_rpm_localiza_manifesto_do_crate_a_partir_de_outra_pasta(self):
        result = self.run_rpm()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([command[0] for command in self.commands()], ["zigbuild", "generate-rpm", "generate-rpm"])
        self.assertIn("--variant", self.commands()[2])
        self.assertEqual(self.commands()[2][self.commands()[2].index("--variant") + 1], "wallpapers")

    def test_compilacao_recusada_impede_geracao_do_rpm(self):
        self.environment["SPARKY_COMPILE_EXIT"] = "17"
        result = self.run_rpm()
        self.assertEqual(result.returncode, 17)
        self.assertEqual([command[0] for command in self.commands()], ["zigbuild"])

    def test_glibc_acima_do_limite_impede_geracao_do_rpm(self):
        self.environment["SPARKY_TEST_GLIBC"] = "2.37"
        result = self.run_rpm()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("acima de 2.36", result.stderr)
        self.assertEqual([command[0] for command in self.commands()], ["zigbuild"])


if __name__ == "__main__":
    unittest.main()
