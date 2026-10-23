import json
import os
from pathlib import Path
import subprocess
import sys


def assets():
    return set(json.loads(Path(os.environ["GH_ASSETS"]).read_text()))


def save_assets(names):
    Path(os.environ["GH_ASSETS"]).write_text(json.dumps(sorted(names)))


def view(arguments):
    field = arguments[arguments.index("--json") + 1]
    if "assets" in field.split(","):
        print(json.dumps({"assets": [{"name": name} for name in sorted(assets())], "isDraft": True}))
        return int(os.environ.get("LIST_ASSETS_EXIT", "0"))
    state = os.environ.get("EXISTING_RELEASE", "missing")
    print(state)
    return 1 if state == "missing" else 0


def change_tag():
    change = os.environ.get("UPLOAD_TAG_CHANGE")
    if not change:
        return
    reference = "refs/tags/v0.22.0"
    suffix = [reference, os.environ["OTHER_COMMIT"]] if change == "move" else ["-d", reference]
    subprocess.run(["git", "--git-dir", os.environ["TEST_REMOTE"], "update-ref", *suffix], check=True)


def upload(arguments):
    change_tag()
    result = int(os.environ.get("UPLOAD_EXIT", "0"))
    if result:
        return result
    names = {Path(value).name for value in arguments[3:] if value != "--clobber"}
    extra = os.environ.get("POST_UPLOAD_ASSET")
    save_assets(assets() | names | ({extra} if extra else set()))
    return 0


def delete_asset(arguments):
    result = int(os.environ.get("DELETE_ASSET_EXIT", "0"))
    if result:
        return result
    save_assets(assets() - {arguments[-1]})
    return 0


def main():
    arguments = sys.argv[1:]
    with Path(os.environ["GH_CALLS"]).open("a") as log:
        log.write(json.dumps(arguments) + "\n")
    match arguments[:2]:
        case ["release", "view"]:
            return view(arguments)
        case ["release", "upload"]:
            return upload(arguments)
        case ["release", "delete-asset"]:
            return delete_asset(arguments)
        case ["release", "create"] | ["release", "edit"]:
            return 0
        case _:
            return 99


if __name__ == "__main__":
    sys.exit(main())
