import os
from pathlib import Path
import subprocess
import sys
import unittest

from ci.release import validate, version_from_tag
from ci.tests.release_support import git, repository


class ReleaseVersionTest(unittest.TestCase):
    def test_accepts_major_minor_patch_and_multiple_digits(self):
        for tag in ["v1.0.0", "v0.22.0", "v0.0.1", "v12.34.56", "v0.0.0"]:
            with self.subTest(tag=tag):
                self.assertEqual(version_from_tag(tag), tag[1:])

    def test_rejects_noncanonical_tags_and_shell_content(self):
        for tag in ["0.22.0", "v01.2.3", "v1.02.3", "v1.2.03", "v1.2", "v1.2.3-rc.1",
                    "v1.2.3+build", "v1.2.3\n", "v1.2.3$(touch injected)", "v1.a.3"]:
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                version_from_tag(tag)

    def test_validates_lightweight_and_annotated_tags(self):
        for annotated in [False, True]:
            with self.subTest(annotated=annotated):
                root = repository(self)
                arguments = ["-a", "-m", "release"] if annotated else []
                git(root, "tag", *arguments, "v0.22.0")
                self.assertEqual(validate(root, "v0.22.0"), ("0.22.0", "1"))

    def test_rejects_cargo_arch_and_lockfile_version_drift(self):
        for path, source in [
            ("Cargo.toml", '[workspace.package]\nversion = "0.21.0"\n'),
            ("Cargo.lock", '[[package]]\nname = "sparky"\nversion = "0.21.0"\n'),
            ("packaging/arch/PKGBUILD", "pkgver=0.21.0\npkgrel=1\n"),
        ]:
            with self.subTest(path=path):
                root = repository(self)
                (root / path).write_text(source)
                git(root, "add", ".")
                git(root, "commit", "-m", "drift")
                git(root, "update-ref", "refs/remotes/origin/main", "HEAD")
                git(root, "tag", "v0.22.0")
                with self.assertRaises(ValueError):
                    validate(root, "v0.22.0")

    def test_ignores_external_crate_versions(self):
        root = repository(self)
        with (root / "Cargo.lock").open("a") as lock:
            lock.write('[[package]]\nname = "external"\nversion = "9.0.0"\nsource = "registry"\n')
        git(root, "add", ".")
        git(root, "commit", "-m", "external")
        git(root, "update-ref", "refs/remotes/origin/main", "HEAD")
        git(root, "tag", "v0.22.0")
        self.assertEqual(validate(root, "v0.22.0"), ("0.22.0", "1"))

    def test_rejects_tag_outside_main(self):
        root = repository(self)
        (root / "feature").touch()
        git(root, "add", ".")
        git(root, "commit", "-m", "unmerged")
        git(root, "tag", "v0.22.0")
        with self.assertRaisesRegex(ValueError, "main"):
            validate(root, "v0.22.0")

    def test_rejects_checkout_different_from_tag(self):
        root = repository(self)
        git(root, "tag", "v0.22.0")
        (root / "later").touch()
        git(root, "add", ".")
        git(root, "commit", "-m", "later")
        git(root, "update-ref", "refs/remotes/origin/main", "HEAD")
        with self.assertRaisesRegex(ValueError, "commit"):
            validate(root, "v0.22.0")

    def test_rejects_invalid_or_duplicate_arch_release(self):
        for release in ["0", "01", "1\npkgrel=2", "$(echo 1)"]:
            with self.subTest(release=release):
                root = repository(self)
                (root / "packaging/arch/PKGBUILD").write_text(f"pkgver=0.22.0\npkgrel={release}\n")
                git(root, "tag", "v0.22.0")
                with self.assertRaises(ValueError):
                    validate(root, "v0.22.0")

    def test_cli_exports_validated_version_to_github_outputs(self):
        root = repository(self)
        git(root, "tag", "v0.22.0")
        environment = os.environ.copy()
        environment.update(PYTHONPATH=str(Path(__file__).resolve().parents[2]),
                           RELEASE_TAG="v0.22.0", GITHUB_OUTPUT=str(root / "outputs"))
        result = subprocess.run([sys.executable, "-m", "ci.release"], cwd=root, env=environment,
                                capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((root / "outputs").read_text(), "version=0.22.0\nrelease=1\n")
        (root / "outputs").unlink()
        environment["RELEASE_TAG"] = "v01.0.0"
        result = subprocess.run([sys.executable, "-m", "ci.release"], cwd=root, env=environment,
                                capture_output=True, text=True, check=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((root / "outputs").exists())
