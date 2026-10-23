import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

from ci.release import version_from_tag
from ci.release_metadata import verify_package


def package_names(version, release):
    version_from_tag(f"v{version}")
    if not re.fullmatch(r"[1-9][0-9]*", release):
        raise ValueError("Revisão de pacote inválida.")
    revision = f"{version}-{release}"
    return {
        f"sparky_{revision}_amd64.deb",
        f"sparky-wallpapers_{revision}_amd64.deb",
        f"sparky-{revision}.x86_64.rpm",
        f"sparky-wallpapers-{revision}.x86_64.rpm",
        f"sparky-{revision}-x86_64.pkg.tar.zst",
        f"sparky-wallpapers-{revision}-any.pkg.tar.zst",
    }


def digest(path):
    with path.open("rb") as package:
        return hashlib.file_digest(package, "sha256").hexdigest()


def prepare(directory, version, release, commit):
    if not re.fullmatch(r"[a-f0-9]{40}", commit):
        raise ValueError("Commit de origem inválido.")
    required = package_names(version, release)
    debug = f"sparky-debug-{version}-{release}-x86_64.pkg.tar.zst"
    files = {path.name: path for path in directory.iterdir()}
    if not required.issubset(files) or set(files) - required - {debug}:
        raise ValueError("O conjunto de pacotes está incompleto ou contém arquivos inesperados.")
    if any(path.is_symlink() or not path.is_file() or path.stat().st_size == 0 for path in files.values()):
        raise ValueError("Os pacotes devem ser arquivos regulares e não vazios.")
    for path in files.values():
        verify_package(path, version, release)
    checksums = {name: digest(files[name]) for name in sorted(files)}
    manifest = "".join(f"{digest}  {name}\n" for name, digest in checksums.items())
    (directory / "SHA256SUMS").write_text(manifest)
    metadata = {"tag": f"v{version}", "version": version, "revision": release, "commit": commit, "sha256": checksums}
    (directory / "release.json").write_text(json.dumps(metadata, indent=2) + "\n")


def main():
    try:
        prepare(Path("release-assets"), os.environ["RELEASE_VERSION"],
                os.environ["PACKAGE_RELEASE"], os.environ["RELEASE_COMMIT"])
        return 0
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        print(f"Artefatos recusados: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
