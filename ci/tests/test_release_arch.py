import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
COMMAND = '''import json
import os
from pathlib import Path
import sys

name = Path(sys.argv[0]).name
with Path(os.environ["SETUP_CALLS"]).open("a") as log:
    log.write(json.dumps([name, *sys.argv[1:]]) + "\\n")
sys.exit(int(os.environ.get("BUILDER_EXIT", "0")) if name == "runuser" else 0)
'''


class ReleaseArchTest(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="sparky-arch-setup-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / "packaging/arch").mkdir(parents=True)
        (self.root / "packaging/arch/PKGBUILD").write_text("depends=('test-library' 'test-shell')\n")
        for name in ["pacman", "git", "useradd", "chown", "runuser"]:
            command = self.root / name
            command.write_text(f"#!{sys.executable}\n" + COMMAND)
            command.chmod(0o755)
        self.environment = os.environ.copy()
        self.environment.update(PATH=str(self.root) + os.pathsep + os.environ["PATH"],
                                GITHUB_ACTIONS="true", SPARKY_BUILDER_UID="1001",
                                SETUP_CALLS=str(self.root / "calls"))

    def run_setup(self):
        return subprocess.run(["bash", str(ROOT / "ci/release-arch.sh")], cwd=self.root,
                              env=self.environment, capture_output=True, text=True, check=False)

    def calls(self):
        return [json.loads(line) for line in (self.root / "calls").read_text().splitlines()]

    def test_installs_manifest_dependencies_and_runs_makepkg_as_checkout_owner(self):
        result = self.run_setup()
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = self.calls()
        packages = [call for call in calls if call[0] == "pacman"][-1]
        self.assertIn("test-library", packages)
        self.assertIn("test-shell", packages)
        self.assertIn(["useradd", "--create-home", "--uid", "1001", "builder"], calls)
        self.assertFalse(any(call[0] == "chown" for call in calls))
        runner = next(call for call in calls if call[0] == "runuser")
        self.assertEqual(runner[1:4], ["-u", "builder", "--"])
        self.assertIn("bash packaging/build.sh pacman", runner[-1])
        self.assertIn("DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent", runner[-1])

    def test_build_failure_is_propagated(self):
        self.environment["BUILDER_EXIT"] = "17"
        self.assertEqual(self.run_setup().returncode, 17)

    def test_refuses_to_install_packages_outside_github_actions(self):
        self.environment.pop("GITHUB_ACTIONS")
        self.assertNotEqual(self.run_setup().returncode, 0)
        self.assertFalse((self.root / "calls").exists())
