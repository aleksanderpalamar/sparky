import subprocess


def read_metadata(path):
    if path.name.endswith(".deb"):
        command = ["dpkg-deb", "--show", "--showformat=${Package}\t${Version}\t${Architecture}", str(path)]
        return tuple(subprocess.check_output(command, text=True).strip().split("\t"))
    if path.name.endswith(".rpm"):
        command = ["rpm", "-qp", "--qf", "%{NAME}\t%{VERSION}-%{RELEASE}\t%{ARCH}", str(path)]
        return tuple(subprocess.check_output(command, text=True).strip().split("\t"))
    source = subprocess.check_output(["bsdtar", "-xOf", str(path), ".PKGINFO"], text=True)
    values = dict(line.split(" = ", 1) for line in source.splitlines() if " = " in line)
    return values["pkgname"], values["pkgver"], values["arch"]


def expected_architectures(path, wallpapers):
    if path.name.endswith(".deb"):
        return {"amd64", "all"} if wallpapers else {"amd64"}
    if path.name.endswith(".rpm"):
        return {"x86_64", "noarch"} if wallpapers else {"x86_64"}
    return {"any"} if wallpapers else {"x86_64"}


def verify_package(path, version, release):
    name, revision, architecture = read_metadata(path)
    expected = "sparky"
    if path.name.startswith("sparky-wallpapers"):
        expected = "sparky-wallpapers"
    if path.name.startswith("sparky-debug"):
        expected = "sparky-debug"
    architectures = expected_architectures(path, expected == "sparky-wallpapers")
    if name != expected or revision != f"{version}-{release}" or architecture not in architectures:
        raise ValueError(f"Metadados incompatíveis com a release: {path.name}.")
