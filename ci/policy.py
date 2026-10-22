import re
from pathlib import Path


CHECKS = (
    "Escopo e branch",
    "SRP e DIP",
    "Código e erros",
    "Testes",
    "Versão e pacotes",
    "Sessão gráfica",
)
SOURCE_EXTENSIONS = {".rs", ".py", ".sh", ".yml", ".yaml"}


def size_problem(path, before, after):
    if Path(path).suffix not in SOURCE_EXTENSIONS and Path(path).name != "PKGBUILD":
        return None
    previous = len(before.splitlines()) if before is not None else 0
    current = len(after.splitlines())
    limit = max(200, previous)
    if current <= limit:
        return None
    return f"{path}: {current} linhas; limite {limit}. Divida responsabilidades em módulos coesos."


def next_fence(current, marker):
    if current is None:
        return marker
    if marker[0] == current[0] and len(marker) >= len(current):
        return None
    return current


def unfenced(body):
    fence = None
    for line in body.splitlines():
        marker = re.match(r"^\s*(`{3,}|~{3,})", line)
        if marker:
            fence = next_fence(fence, marker[1])
            continue
        if fence is None:
            yield line


def missing_checks(body):
    visible = re.sub(r"<!--[\s\S]*?(?:-->|\Z)", "", body)
    text = "\n".join(unfenced(visible))
    return [
        label for label in CHECKS
        if not re.search(rf"^ {{0,3}}- \[[xX]\] {re.escape(label)}:", text, re.M)
    ]
