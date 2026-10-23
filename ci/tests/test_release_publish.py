import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest

from ci.tests.release_support import git, repository


ROOT = Path(__file__).resolve().parents[2]
GH = (ROOT / "ci/tests/release_gh.py").read_text()


class ReleasePublishTest(unittest.TestCase):
    def setUp(self):
        self.root = repository(self)
        self.commit = git(self.root, "rev-parse", "HEAD")
        git(self.root, "tag", "v0.22.0")
        git(self.root, "clone", "--bare", "--quiet", ".", "remote.git")
        self.remote = self.root / "remote.git"
        git(self.root, "remote", "add", "origin", str(self.remote))
        assets = self.root / "release-assets"
        assets.mkdir()
        (assets / "sparky.deb").write_bytes(b"fixture")
        checksum = hashlib.sha256(b"fixture").hexdigest()
        (assets / "SHA256SUMS").write_text(f"{checksum}  sparky.deb\n")
        (assets / "release.json").write_text(json.dumps({"commit": self.commit}))
        self.assets = self.root / "remote-assets.json"
        self.assets.write_text("[]")
        command = self.root / "gh"
        command.write_text(f"#!{sys.executable}\n" + GH)
        command.chmod(0o755)
        self.environment = os.environ.copy()
        self.environment.update(PATH=str(self.root) + os.pathsep + os.environ["PATH"],
                                RELEASE_TAG="v0.22.0", RELEASE_COMMIT=self.commit,
                                GH_CALLS=str(self.root / "calls"), TEST_REMOTE=str(self.remote),
                                GH_ASSETS=str(self.assets))

    def run_publish(self):
        return subprocess.run(["bash", str(ROOT / "ci/release-publish.sh")], cwd=self.root,
                              env=self.environment, capture_output=True, text=True, check=False)

    def calls(self):
        return [json.loads(line) for line in (self.root / "calls").read_text().splitlines()]

    def later_commit(self):
        git(self.root, "commit", "--allow-empty", "-m", "later")
        git(self.root, "push", "--quiet", "origin", "HEAD:refs/heads/main")
        return git(self.root, "rev-parse", "HEAD")

    def test_creates_verified_tag_draft_uploads_then_publishes(self):
        result = self.run_publish()
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = self.calls()
        self.assertEqual([call[1] for call in calls], ["view", "create", "view", "upload", "view", "edit"])
        self.assertIn("--verify-tag", calls[1])
        self.assertIn("--draft", calls[1])
        self.assertIn("--draft=false", calls[-1])

    def test_resumes_only_an_existing_draft(self):
        self.environment["EXISTING_RELEASE"] = "true"
        self.assertEqual(self.run_publish().returncode, 0)
        self.assertEqual([call[1] for call in self.calls()], ["view", "view", "upload", "view", "edit"])

    def test_published_release_is_not_overwritten(self):
        self.environment["EXISTING_RELEASE"] = "false"
        self.assertNotEqual(self.run_publish().returncode, 0)
        self.assertEqual([call[1] for call in self.calls()], ["view"])

    def test_failed_upload_does_not_publish_partial_release(self):
        self.environment["UPLOAD_EXIT"] = "17"
        self.assertEqual(self.run_publish().returncode, 17)
        self.assertEqual([call[1] for call in self.calls()], ["view", "create", "view", "upload"])

    def test_invalid_tag_or_checksum_blocks_every_github_operation(self):
        for failure in ["tag", "checksum"]:
            with self.subTest(failure=failure):
                self.environment["RELEASE_TAG"] = "v01.0.0" if failure == "tag" else "v0.22.0"
                (self.root / "release-assets/sparky.deb").write_bytes(b"corrupt")
                self.assertNotEqual(self.run_publish().returncode, 0)
                self.assertFalse((self.root / "calls").exists())

    def test_moved_tag_blocks_creation_and_resuming_a_draft(self):
        later = self.later_commit()
        git(self.root, "--git-dir", str(self.remote), "update-ref", "refs/tags/v0.22.0", later)
        for existing in ["missing", "true"]:
            with self.subTest(existing=existing):
                self.environment["EXISTING_RELEASE"] = existing
                self.assertNotEqual(self.run_publish().returncode, 0)
                self.assertFalse((self.root / "calls").exists())

    def test_deleted_tag_blocks_every_github_operation(self):
        git(self.root, "--git-dir", str(self.remote), "update-ref", "-d", "refs/tags/v0.22.0")
        self.assertNotEqual(self.run_publish().returncode, 0)
        self.assertFalse((self.root / "calls").exists())

    def test_tag_changed_during_upload_never_publishes_the_release(self):
        self.environment["OTHER_COMMIT"] = self.later_commit()
        self.environment["UPLOAD_TAG_CHANGE"] = "move"
        self.assertNotEqual(self.run_publish().returncode, 0)
        self.assertNotIn("edit", [call[1] for call in self.calls()])

    def test_recorded_commit_must_match_the_original_workflow_commit(self):
        (self.root / "release-assets/release.json").write_text(json.dumps({"commit": "a" * 40}))
        self.assertNotEqual(self.run_publish().returncode, 0)
        self.assertFalse((self.root / "calls").exists())

    def test_accepts_an_annotated_tag_with_the_same_peeled_commit(self):
        git(self.root, "tag", "-f", "-a", "v0.22.0", "-m", "release")
        git(self.root, "push", "--force", "--quiet", "origin", "refs/tags/v0.22.0")
        self.assertEqual(self.run_publish().returncode, 0)

    def test_tag_deleted_during_upload_never_publishes(self):
        self.environment["UPLOAD_TAG_CHANGE"] = "delete"
        self.assertNotEqual(self.run_publish().returncode, 0)
        self.assertNotIn("edit", [call[1] for call in self.calls()])

    def test_resumed_draft_removes_only_assets_absent_from_the_validated_set(self):
        self.environment["EXISTING_RELEASE"] = "true"
        stale = "sparky-debug-0.22.0-1-x86_64.pkg.tar.zst"
        self.assets.write_text(json.dumps([stale, "sparky.deb"]))
        self.assertEqual(self.run_publish().returncode, 0)
        calls = self.calls()
        deleted = [call for call in calls if call[1] == "delete-asset"]
        self.assertEqual(len(deleted), 1)
        self.assertEqual(deleted[0][-1], stale)
        self.assertLess(calls.index(deleted[0]), next(index for index, call in enumerate(calls) if call[1] == "upload"))
        self.assertEqual(set(json.loads(self.assets.read_text())), {"sparky.deb", "SHA256SUMS", "release.json"})

    def test_failed_asset_removal_blocks_upload_and_publication(self):
        self.assets.write_text(json.dumps(["stale package.pkg.tar.zst"]))
        self.environment["DELETE_ASSET_EXIT"] = "17"
        self.assertNotEqual(self.run_publish().returncode, 0)
        self.assertFalse({"upload", "edit"} & {call[1] for call in self.calls()})

    def test_assets_added_during_upload_are_not_published(self):
        self.environment["POST_UPLOAD_ASSET"] = "unverified.pkg.tar.zst"
        self.assertNotEqual(self.run_publish().returncode, 0)
        self.assertNotIn("edit", [call[1] for call in self.calls()])

    def test_remote_asset_query_failure_blocks_publication(self):
        self.environment["LIST_ASSETS_EXIT"] = "13"
        self.assertNotEqual(self.run_publish().returncode, 0)
        self.assertNotIn("edit", [call[1] for call in self.calls()])
