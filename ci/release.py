import os
from pathlib import Path
import re
import subprocess
import sys
import tomllib


def version_from_tag(tag):
    if not re.fullmatch(r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", tag):
        raise ValueError("A tag deve seguir vMAJOR.MINOR.PATCH, sem zeros iniciais ou sufixos.")
    return tag[1:]


def arch_value(source, key, pattern):
    values = re.findall(rf"^{key}=(.*)$", source, re.M)
    if len(values) != 1 or not re.fullmatch(pattern, values[0]):
        raise ValueError(f"PKGBUILD deve declarar {key} uma única vez, em formato literal.")
    return values[0]


def package_versions(root, version):
    manifest = tomllib.loads((root / "Cargo.toml").read_text())
    if manifest["workspace"]["package"]["version"] != version:
        raise ValueError("A versão do Cargo.toml difere da tag.")
    source = (root / "packaging/arch/PKGBUILD").read_text()
    if arch_value(source, "pkgver", r"[0-9]+\.[0-9]+\.[0-9]+") != version:
        raise ValueError("A versão do PKGBUILD difere da tag.")
    lock = tomllib.loads((root / "Cargo.lock").read_text())
    local = [package for package in lock["package"] if "source" not in package]
    if not local or any(package["version"] != version for package in local):
        raise ValueError("As versões locais do Cargo.lock diferem da tag.")
    return arch_value(source, "pkgrel", r"[1-9][0-9]*")


def git(root, *arguments):
    return subprocess.check_output(["git", "-C", str(root), *arguments], text=True).strip()


def validate(root, tag):
    version = version_from_tag(tag)
    release = package_versions(root, version)
    head = git(root, "rev-parse", "HEAD")
    if git(root, "rev-parse", f"refs/tags/{tag}^{{commit}}") != head:
        raise ValueError("O checkout precisa corresponder ao commit da tag.")
    result = subprocess.run(
        ["git", "-C", str(root), "merge-base", "--is-ancestor", head, "origin/main"],
        check=False,
    )
    if result.returncode != 0:
        raise ValueError("O commit da tag precisa estar integrado na main remota.")
    return version, release


def main():
    try:
        version, release = validate(Path.cwd(), os.environ["RELEASE_TAG"])
        with Path(os.environ["GITHUB_OUTPUT"]).open("a") as output:
            output.write(f"version={version}\nrelease={release}\n")
        print(f"Tag v{version} validada; revisão dos pacotes: {release}.")
        return 0
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        print(f"Release recusada: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
