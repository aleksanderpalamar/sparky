import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class HeadlessCargoTest(unittest.TestCase):
    def test_graphical_environment_is_isolated_and_exit_code_is_preserved(self):
        with tempfile.TemporaryDirectory(prefix="sparky-cargo-fixture-") as temporary:
            cargo = Path(temporary) / "cargo"
            cargo.write_text(f"#!{sys.executable}\n" + """import json, os, sys
keys = ['DISPLAY', 'WAYLAND_DISPLAY', 'DBUS_SESSION_BUS_ADDRESS', 'DBUS_SYSTEM_BUS_ADDRESS', 'XDG_RUNTIME_DIR', 'SPARKY_SOCKET']
print(json.dumps({'args': sys.argv[1:], 'env': {key: os.environ.get(key) for key in keys}}))
sys.exit(23)
""")
            cargo.chmod(0o755)
            environment = os.environ.copy()
            environment.update({key: "fixture-do-not-inherit" for key in [
                "DISPLAY", "WAYLAND_DISPLAY", "DBUS_SESSION_BUS_ADDRESS",
                "DBUS_SYSTEM_BUS_ADDRESS", "XDG_RUNTIME_DIR", "SPARKY_SOCKET",
            ]})
            environment["PATH"] = temporary + os.pathsep + environment["PATH"]
            wrapper = Path(__file__).resolve().parents[1] / "cargo.sh"
            result = subprocess.run(["bash", str(wrapper), "test", "--locked"], env=environment, capture_output=True, text=True)
            self.assertEqual(result.returncode, 23)
            record = json.loads(result.stdout)
            self.assertEqual(record["args"], ["test", "--locked"])
            expected = {key: None for key in record["env"]}
            expected.update(DBUS_SESSION_BUS_ADDRESS="unix:path=/nonexistent", DBUS_SYSTEM_BUS_ADDRESS="unix:path=/nonexistent")
            self.assertEqual(record["env"], expected)
