use sparky_theme::Rgb;

use crate::domain::runs::{Glyph, Run, Width, rows};
use crate::domain::style::{Decoration, Slant, Style, Weight};

fn estilo(fg: u8) -> Style {
    Style {
        fg: Rgb::new(fg, fg, fg),
        bg: Rgb::new(0, 0, 0),
        weight: Weight::Normal,
        slant: Slant::Upright,
        decoration: Decoration::None,
    }
}

fn glifo(line: usize, column: usize, character: char, fg: u8) -> Glyph {
    Glyph {
        line,
        column,
        character,
        marks: Vec::new(),
        width: Width::Single,
        style: estilo(fg),
    }
}

fn trechos(linha: &[Run]) -> Vec<(usize, usize, &str)> {
    linha
        .iter()
        .map(|run| (run.column, run.cells, run.text.as_str()))
        .collect()
}

#[test]
fn letras_seguidas_com_o_mesmo_estilo_viram_um_trecho() {
    let linhas = rows(
        [
            glifo(0, 0, 'l', 1),
            glifo(0, 1, 's', 1),
            glifo(0, 2, ' ', 1),
        ],
        1,
    );
    assert_eq!(trechos(&linhas[0]), [(0, 3, "ls ")]);
}

#[test]
fn mudar_o_estilo_ou_a_linha_corta_o_trecho() {
    let linhas = rows(
        [
            glifo(0, 0, 'a', 1),
            glifo(0, 1, 'b', 2),
            glifo(1, 0, 'c', 2),
        ],
        2,
    );
    assert_eq!(trechos(&linhas[0]), [(0, 1, "a"), (1, 1, "b")]);
    assert_eq!(trechos(&linhas[1]), [(0, 1, "c")]);
}

#[test]
fn simbolos_fora_do_ascii_ficam_sozinhos_na_coluna_certa() {
    let linhas = rows(
        [
            glifo(0, 0, 'a', 1),
            glifo(0, 1, '─', 1),
            glifo(0, 2, '\u{f418}', 1),
            glifo(0, 3, 'b', 1),
        ],
        1,
    );
    assert_eq!(
        trechos(&linhas[0]),
        [(0, 1, "a"), (1, 1, "─"), (2, 1, "\u{f418}"), (3, 1, "b")]
    );
}

#[test]
fn caractere_largo_ocupa_duas_celulas() {
    let largo = Glyph {
        width: Width::Double,
        ..glifo(0, 0, '日', 1)
    };
    let linhas = rows([largo, glifo(0, 2, 'x', 1)], 1);
    assert_eq!(trechos(&linhas[0]), [(0, 2, "日"), (2, 1, "x")]);
}

#[test]
fn linhas_fora_da_tela_sao_ignoradas() {
    let linhas = rows([glifo(5, 0, 'z', 1)], 2);
    assert_eq!(linhas.len(), 2);
    assert!(linhas.iter().all(Vec::is_empty));
}

#[test]
fn marcas_de_largura_zero_ficam_junto_do_caractere() {
    let acentuado = Glyph {
        marks: vec!['\u{301}'],
        ..glifo(0, 1, 'e', 1)
    };
    let linhas = rows([glifo(0, 0, 'a', 1), acentuado, glifo(0, 2, 'b', 1)], 1);
    assert_eq!(
        trechos(&linhas[0]),
        [(0, 1, "a"), (1, 1, "e\u{301}"), (2, 1, "b")]
    );
}
