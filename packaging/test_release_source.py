from pathlib import Path
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[1]


class ReleaseSourceTest(unittest.TestCase):
    def test_arch_source_is_pinned_to_the_checkout_commit(self):
        result = subprocess.check_output([
            "bash", "-c", 'startdir="$PWD/packaging/arch"; source "$startdir/PKGBUILD"; printf "%s" "${source[0]}"',
        ], cwd=ROOT, text=True)
        commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
        self.assertTrue(result.endswith(f"#commit={commit}"), result)
