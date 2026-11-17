import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
NATIVE = ("sparky-files", "sparky-image", "sparky-shot", "sparky-monitor", "sparky-terminal")


class NativeAppNames(unittest.TestCase):
    def test_apps_nativos_aparecem_com_o_nome_do_programa_em_todo_idioma(self):
        for program in NATIVE:
            contents = (ROOT / "data/applications" / f"{program}.desktop").read_text()
            names = [line for line in contents.splitlines() if line.startswith("Name")]
            self.assertEqual(names, [f"Name={program}", f"Name[en]={program}"], program)

    def test_o_nome_antigo_continua_achavel_pela_busca(self):
        antigos = {
            "sparky-files": "arquivos",
            "sparky-image": "imagens",
            "sparky-shot": "captura",
            "sparky-monitor": "monitor",
            "sparky-terminal": "terminal",
        }
        for program, palavra in antigos.items():
            contents = (ROOT / "data/applications" / f"{program}.desktop").read_text().lower()
            self.assertIn(palavra, contents, program)


if __name__ == "__main__":
    unittest.main()
