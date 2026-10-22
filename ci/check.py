import argparse
import json
import os
import subprocess
import sys
import tomllib
from pathlib import Path

from ci.changes import changed_paths, check_format, contents, git
from ci.policy import SOURCE_EXTENSIONS, missing_checks, size_problem
from ci.refs import event_base, pull_request_problem


def context(arguments):
    event_path = os.environ.get("GITHUB_EVENT_PATH")
    event = json.loads(Path(event_path).read_text()) if event_path else {}
    base = arguments.base or event_base(os.environ.get("GITHUB_EVENT_NAME"), event)
    base = base or git("merge-base", "origin/main", "HEAD").strip()
    return base, None if arguments.worktree else "HEAD", event


def inspect_files(base, head):
    edition = str(tomllib.loads(Path("Cargo.toml").read_text())["workspace"]["package"]["edition"])
    problems = []
    for path, old_path in changed_paths(base, head):
        if Path(path).suffix not in SOURCE_EXTENSIONS and Path(path).name != "PKGBUILD":
            continue
        before = contents(base, old_path) if old_path else None
        problem = size_problem(path, before, contents(head, path))
        if problem:
            problems.append(problem)
        if Path(path).suffix == ".rs" and not check_format(path, edition):
            problems.append(f"{path}: formatação divergente do rustfmt.")
    return problems


def inspect_request(event):
    problem = pull_request_problem(event)
    problems = [problem] if problem else []
    request = event.get("pull_request")
    if request is None:
        return problems
    missing = missing_checks(request.get("body") or "")
    problems.extend(f"Checklist do PR pendente: {label}." for label in missing)
    return problems


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--base")
    parser.add_argument("--worktree", action="store_true")
    arguments = parser.parse_args()
    base, head, event = context(arguments)
    revision = [base, head] if head else [base]
    subprocess.run(["git", "diff", "--check", *revision], check=True)
    problems = inspect_files(base, head) + inspect_request(event)
    for problem in problems:
        print(problem, file=sys.stderr)
    if problems:
        return 1
    print("Convenções do projeto e checklist do PR aprovados.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
