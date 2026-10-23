use alacritty_terminal::vte::ansi::{Color, NamedColor, Rgb as Spec};
use sparky_theme::{Rgb, TerminalPalette};

use crate::domain::color::{dim, resolve};

const PALETA: TerminalPalette = TerminalPalette::TOKYO_NIGHT;

fn sem_trocas(_: usize) -> Option<Rgb> {
    None
}

#[test]
fn as_cores_com_nome_vem_do_tema() {
    assert_eq!(
        resolve(Color::Named(NamedColor::Red), &PALETA, &sem_trocas),
        PALETA.ansi[1]
    );
    assert_eq!(
        resolve(Color::Named(NamedColor::BrightBlue), &PALETA, &sem_trocas),
        PALETA.ansi[12]
    );
    assert_eq!(
        resolve(Color::Named(NamedColor::Foreground), &PALETA, &sem_trocas),
        PALETA.foreground
    );
    assert_eq!(
        resolve(Color::Named(NamedColor::Background), &PALETA, &sem_trocas),
        PALETA.background
    );
    assert_eq!(
        resolve(Color::Named(NamedColor::Cursor), &PALETA, &sem_trocas),
        PALETA.cursor
    );
    assert_eq!(
        resolve(
            Color::Named(NamedColor::BrightForeground),
            &PALETA,
            &sem_trocas
        ),
        PALETA.ansi[15]
    );
}

#[test]
fn as_256_cores_seguem_o_cubo_e_os_cinzas_do_xterm() {
    assert_eq!(
        resolve(Color::Indexed(4), &PALETA, &sem_trocas),
        PALETA.ansi[4]
    );
    assert_eq!(
        resolve(Color::Indexed(16), &PALETA, &sem_trocas),
        Rgb::new(0, 0, 0)
    );
    assert_eq!(
        resolve(Color::Indexed(196), &PALETA, &sem_trocas),
        Rgb::new(255, 0, 0)
    );
    assert_eq!(
        resolve(Color::Indexed(110), &PALETA, &sem_trocas),
        Rgb::new(135, 175, 215)
    );
    assert_eq!(
        resolve(Color::Indexed(231), &PALETA, &sem_trocas),
        Rgb::new(255, 255, 255)
    );
    assert_eq!(
        resolve(Color::Indexed(232), &PALETA, &sem_trocas),
        Rgb::new(8, 8, 8)
    );
    assert_eq!(
        resolve(Color::Indexed(255), &PALETA, &sem_trocas),
        Rgb::new(238, 238, 238)
    );
}

#[test]
fn truecolor_passa_direto() {
    assert_eq!(
        resolve(Color::Spec(Spec { r: 1, g: 2, b: 3 }), &PALETA, &sem_trocas),
        Rgb::new(1, 2, 3)
    );
}

#[test]
fn cores_trocadas_por_osc_tem_prioridade() {
    let trocas = |indice: usize| (indice == 1).then_some(Rgb::new(0x10, 0x20, 0x30));
    assert_eq!(
        resolve(Color::Named(NamedColor::Red), &PALETA, &trocas),
        Rgb::new(0x10, 0x20, 0x30)
    );
    assert_eq!(
        resolve(Color::Indexed(1), &PALETA, &trocas),
        Rgb::new(0x10, 0x20, 0x30)
    );
    assert_eq!(
        resolve(Color::Named(NamedColor::Green), &PALETA, &trocas),
        PALETA.ansi[2]
    );
}

#[test]
fn as_cores_esmaecidas_ficam_com_dois_tercos() {
    assert_eq!(dim(Rgb::new(90, 150, 255)), Rgb::new(60, 100, 170));
    assert_eq!(
        resolve(Color::Named(NamedColor::DimRed), &PALETA, &sem_trocas),
        dim(PALETA.ansi[1])
    );
    assert_eq!(
        resolve(
            Color::Named(NamedColor::DimForeground),
            &PALETA,
            &sem_trocas
        ),
        dim(PALETA.foreground)
    );
}

#[test]
fn as_respostas_de_cor_por_indice_usam_o_tema() {
    use crate::domain::color::by_index;
    assert_eq!(by_index(1, &PALETA), PALETA.ansi[1]);
    assert_eq!(by_index(196, &PALETA), Rgb::new(255, 0, 0));
    assert_eq!(by_index(256, &PALETA), PALETA.foreground);
    assert_eq!(by_index(257, &PALETA), PALETA.background);
    assert_eq!(by_index(258, &PALETA), PALETA.cursor);
    assert_eq!(by_index(999, &PALETA), PALETA.foreground);
}
