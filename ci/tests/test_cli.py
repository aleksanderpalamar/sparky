import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from ci.policy import CHECKS


ROOT = Path(__file__).resolve().parents[2]


class GateIntegrationTest(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory(prefix="sparky-gate-test-")
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        self.environment = os.environ.copy()
        self.environment.update(PYTHONPATH=str(ROOT), GIT_CONFIG_GLOBAL="/dev/null", GIT_CONFIG_NOSYSTEM="1")
        self.environment.pop("GITHUB_EVENT_PATH", None)
        self.environment.pop("GITHUB_EVENT_NAME", None)
        self.git("init", "-q", "-b", "main")
        self.write("Cargo.toml", '[workspace.package]\nedition = "2024"\n')
        self.write("ready.rs", "pub fn ready() {}\n")
        self.commit()
        self.base = self.git("rev-parse", "HEAD").strip()

    def git(self, *arguments):
        return subprocess.check_output([
            "git", "-c", "core.hooksPath=/dev/null", "-c", "commit.gpgsign=false",
            "-c", "user.name=Gate Test", "-c", "user.email=ci@example.test", *arguments,
        ], cwd=self.root, env=self.environment, text=True)

    def write(self, path, content):
        (self.root / path).write_text(content)

    def commit(self):
        self.git("add", ".")
        self.git("commit", "-qm", "fixture")

    def check(self, *arguments):
        return subprocess.run([
            sys.executable, "-m", "ci.check", "--base", self.base, *arguments,
        ], cwd=self.root, env=self.environment, capture_output=True, text=True, timeout=30)

    def test_committed_oversized_source_is_rejected(self):
        self.write("new.py", "value = 1\n" * 201)
        self.commit()
        result = self.check()
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("new.py: 201 linhas", result.stderr)

    def test_worktree_rustfmt_is_read_only_and_skips_unchanged_modules(self):
        bad = "pub fn unformatted(){ }\n"
        self.write("child.rs", bad)
        self.write("ready.rs", "mod child;\n\npub fn ready() {}\n")
        self.commit()
        self.base = self.git("rev-parse", "HEAD").strip()
        self.write("ready.rs", "mod child;\n\npub fn ready() {}\n\npub fn added() {}\n")
        self.assertEqual(self.check("--worktree").returncode, 0)
        self.assertEqual((self.root / "child.rs").read_text(), bad)
        self.write("ready.rs", bad)
        result = self.check("--worktree")
        self.assertEqual(result.returncode, 1)
        self.assertEqual((self.root / "ready.rs").read_text(), bad)

    def test_pull_request_body_is_data_and_all_checkboxes_are_required(self):
        body = "\n".join(f"- [x] {label}: conferido" for label in CHECKS)
        event = {"pull_request": {
            "base": {"ref": "main", "sha": self.base}, "head": {"ref": "ci/gate"},
            "body": body + "\n$(touch injected)\n",
        }}
        self.write("event.json", json.dumps(event))
        self.environment.update(GITHUB_EVENT_NAME="pull_request", GITHUB_EVENT_PATH=str(self.root / "event.json"))
        self.assertEqual(self.check().returncode, 0)
        self.assertFalse((self.root / "injected").exists())
        event["pull_request"]["body"] = body.replace("[x]", "[ ]", 1)
        self.write("event.json", json.dumps(event))
        self.assertEqual(self.check().returncode, 1)
