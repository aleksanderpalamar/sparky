import subprocess
from pathlib import Path


def git(*arguments):
    return subprocess.check_output(["git", *arguments], text=True)


def parse_changes(output):
    fields = iter(output.rstrip("\0").split("\0"))
    changes = []
    for status in fields:
        if not status:
            continue
        path = next(fields)
        if status.startswith(("R", "C")):
            changes.append((next(fields), path))
            continue
        changes.append((path, None if status == "A" else path))
    return changes


def changed_paths(base, head=None):
    revision = [base, head] if head else [base]
    output = git("diff", "--name-status", "-z", "--find-renames", "--diff-filter=ACMRT", *revision)
    changes = parse_changes(output)
    if head is not None:
        return changes
    untracked = git("ls-files", "--others", "--exclude-standard", "-z").split("\0")
    return changes + [(path, None) for path in untracked if path]


def contents(revision, path):
    if revision is None:
        return Path(path).read_text()
    return git("show", f"{revision}:{path}")


def check_format(path, edition):
    result = subprocess.run([
        "rustfmt", "--check", "--edition", edition,
        "--config", "skip_children=true", str(Path(path).resolve()),
    ], check=False)
    return result.returncode == 0
