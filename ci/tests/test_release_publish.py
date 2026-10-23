import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
GH = '''import json
import os
from pathlib import Path
import sys

arguments = sys.argv[1:]
with Path(os.environ["GH_CALLS"]).open("a") as log:
    log.write(json.dumps(arguments) + "\\n")
if arguments[:2] == ["release", "view"]:
    state = os.environ.get("EXISTING_RELEASE", "missing")
    print(state)
    sys.exit(1 if state == "missing" else 0)
if arguments[:2] == ["release", "upload"]:
    sys.exit(int(os.environ.get("UPLOAD_EXIT", "0")))
'''


class ReleasePublishTest(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="sparky-release-publish-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        assets = self.root / "release-assets"
        assets.mkdir()
        (assets / "sparky.deb").write_bytes(b"fixture")
        checksum = hashlib.sha256(b"fixture").hexdigest()
        (assets / "SHA256SUMS").write_text(f"{checksum}  sparky.deb\n")
        command = self.root / "gh"
        command.write_text(f"#!{sys.executable}\n" + GH)
        command.chmod(0o755)
        self.environment = os.environ.copy()
        self.environment.update(PATH=str(self.root) + os.pathsep + os.environ["PATH"],
                                RELEASE_TAG="v0.22.0", GH_CALLS=str(self.root / "calls"))

    def run_publish(self):
        return subprocess.run(["bash", str(ROOT / "ci/release-publish.sh")], cwd=self.root,
                              env=self.environment, capture_output=True, text=True, check=False)

    def calls(self):
        return [json.loads(line) for line in (self.root / "calls").read_text().splitlines()]

    def test_creates_verified_tag_draft_uploads_then_publishes(self):
        result = self.run_publish()
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = self.calls()
        self.assertEqual([call[1] for call in calls], ["view", "create", "upload", "edit"])
        self.assertIn("--verify-tag", calls[1])
        self.assertIn("--draft", calls[1])
        self.assertIn("--draft=false", calls[-1])

    def test_resumes_only_an_existing_draft(self):
        self.environment["EXISTING_RELEASE"] = "true"
        self.assertEqual(self.run_publish().returncode, 0)
        self.assertEqual([call[1] for call in self.calls()], ["view", "upload", "edit"])

    def test_published_release_is_not_overwritten(self):
        self.environment["EXISTING_RELEASE"] = "false"
        self.assertNotEqual(self.run_publish().returncode, 0)
        self.assertEqual([call[1] for call in self.calls()], ["view"])

    def test_failed_upload_does_not_publish_partial_release(self):
        self.environment["UPLOAD_EXIT"] = "17"
        self.assertEqual(self.run_publish().returncode, 17)
        self.assertEqual([call[1] for call in self.calls()], ["view", "create", "upload"])

    def test_invalid_tag_or_checksum_blocks_every_github_operation(self):
        for failure in ["tag", "checksum"]:
            with self.subTest(failure=failure):
                self.environment["RELEASE_TAG"] = "v01.0.0" if failure == "tag" else "v0.22.0"
                (self.root / "release-assets/sparky.deb").write_bytes(b"corrupt")
                self.assertNotEqual(self.run_publish().returncode, 0)
                self.assertFalse((self.root / "calls").exists())
