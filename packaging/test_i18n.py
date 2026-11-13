import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOMAINS = {"login", "keyring", "polkit", "launcher", "settings", "bar", "files", "image", "shot", "monitor", "terminal", "notifications", "fetch", "store"}


class CatalogPackages(unittest.TestCase):
    def test_cada_idioma_contem_os_mesmos_dominios(self):
        for language in ("pt-BR", "en"):
            names = {path.stem for path in (ROOT / "data/i18n" / language).glob("*.ftl")}
            self.assertEqual(names, DOMAINS)

    def test_os_idiomas_tem_as_mesmas_chaves_e_parametros(self):
        for domain in DOMAINS:
            portuguese = messages(ROOT / "data/i18n/pt-BR" / f"{domain}.ftl")
            english = messages(ROOT / "data/i18n/en" / f"{domain}.ftl")
            self.assertEqual(set(portuguese), set(english), domain)
            for key, variables in portuguese.items():
                self.assertEqual(variables, english[key], f"{domain}: {key}")

    def test_pacotes_preservam_as_pastas_de_idioma(self):
        manifest = (ROOT / "crates/sparky-compositor/Cargo.toml").read_text()
        arch = (ROOT / "packaging/arch/PKGBUILD").read_text()
        for language in ("pt-BR", "en"):
            self.assertIn(
                f'../../data/i18n/{language}/*.ftl", "usr/share/sparky/i18n/{language}/',
                manifest,
            )
            self.assertIn(
                f'../../data/i18n/{language}/*.ftl", dest = "/usr/share/sparky/i18n/{language}/',
                manifest,
            )
        self.assertIn('for file in data/i18n/*/*.ftl; do', arch)
        self.assertIn('"$pkgdir/usr/share/sparky/${file#data/}"', arch)

    def test_entradas_desktop_tem_nome_e_descricao_em_ingles(self):
        entries = list((ROOT / "data/applications").glob("sparky-*.desktop"))
        entries.append(ROOT / "data/session/sparky.desktop")
        for entry in entries:
            contents = entry.read_text()
            self.assertIn("Name[en]=", contents, entry.name)
            self.assertIn("Comment[en]=", contents, entry.name)


def messages(path):
    found = {}
    key = None
    for line in path.read_text().splitlines():
        start = re.match(r"^([a-z][a-z0-9-]*)\s*=", line)
        if start:
            key = start.group(1)
            if key in found:
                raise ValueError(f"duplicate catalog key {key} in {path}")
            found[key] = set()
        if key:
            found[key] |= set(re.findall(r"\{\s*\$([a-z_]+)", line))
    return found


if __name__ == "__main__":
    unittest.main()
