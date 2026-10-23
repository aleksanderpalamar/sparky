import json
import os
from pathlib import Path
import subprocess
import sys

from ci.release import version_from_tag


def asset_names(tag):
    output = subprocess.check_output([
        "gh", "release", "view", tag, "--json", "assets,isDraft",
    ], text=True)
    release = json.loads(output)
    if release["isDraft"] is not True:
        raise ValueError("Os anexos só podem ser reconciliados em um rascunho.")
    return {asset["name"] for asset in release["assets"]}


def prune(tag, expected):
    for name in sorted(asset_names(tag) - expected):
        subprocess.run(["gh", "release", "delete-asset", "--yes", "--", tag, name], check=True)


def verify(tag, expected):
    if asset_names(tag) != expected:
        raise ValueError("Os anexos remotos diferem do conjunto local conferido.")


def main():
    try:
        tag = os.environ["RELEASE_TAG"]
        version_from_tag(tag)
        expected = {path.name for path in Path("release-assets").iterdir()}
        match sys.argv[1:]:
            case ["prune"]:
                prune(tag, expected)
            case ["verify"]:
                verify(tag, expected)
            case _:
                raise ValueError("Use prune ou verify para conferir os anexos do rascunho.")
        return 0
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        print(f"Publicação recusada: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
