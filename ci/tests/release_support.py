from pathlib import Path
import subprocess
import tempfile


def git(root, *arguments):
    return subprocess.check_output(["git", "-C", str(root), *arguments], text=True).strip()


def repository(test, version="0.22.0"):
    temporary = tempfile.TemporaryDirectory(prefix="sparky-release-test-")
    test.addCleanup(temporary.cleanup)
    root = Path(temporary.name)
    (root / "packaging/arch").mkdir(parents=True)
    (root / "Cargo.toml").write_text(f'[workspace.package]\nversion = "{version}"\n')
    (root / "Cargo.lock").write_text(f'[[package]]\nname = "sparky"\nversion = "{version}"\n')
    (root / "packaging/arch/PKGBUILD").write_text(f"pkgver={version}\npkgrel=1\n")
    git(root, "init", "-b", "main")
    git(root, "config", "user.name", "Release Test")
    git(root, "config", "user.email", "release@example.invalid")
    git(root, "add", ".")
    git(root, "commit", "-m", "initial")
    git(root, "update-ref", "refs/remotes/origin/main", "HEAD")
    return root
