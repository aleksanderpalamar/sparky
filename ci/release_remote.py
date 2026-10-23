import json
import os
from pathlib import Path
import re
import subprocess
import sys

from ci.release import version_from_tag


def remote_commit(tag, remote="origin"):
    version_from_tag(tag)
    reference = f"refs/tags/{tag}"
    output = subprocess.check_output([
        "git", "ls-remote", "--exit-code", remote, reference, f"{reference}^{{}}",
    ], text=True)
    references = dict(reversed(line.split("\t", 1)) for line in output.splitlines())
    commit = references.get(f"{reference}^{{}}") or references.get(reference)
    if not commit:
        raise ValueError("A tag não existe mais no remoto.")
    return commit


def verify(tag, expected, recorded, remote="origin"):
    if not re.fullmatch(r"[a-f0-9]{40}", expected) or recorded != expected:
        raise ValueError("O commit registrado nos artefatos difere do commit original do workflow.")
    if remote_commit(tag, remote) != expected:
        raise ValueError("A tag remota foi movida e não aponta para o commit dos pacotes.")


def main():
    try:
        metadata = json.loads(Path("release-assets/release.json").read_text())
        verify(os.environ["RELEASE_TAG"], os.environ["RELEASE_COMMIT"], metadata["commit"])
        return 0
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        print(f"Publicação recusada: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
