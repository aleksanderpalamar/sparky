use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::vte::ansi::{Color, NamedColor};
use sparky_theme::{Rgb, TerminalPalette};

use crate::domain::color::dim;
use crate::domain::style::{Decoration, Selected, Slant, Style, Weight, style};

const PALETA: TerminalPalette = TerminalPalette::TOKYO_NIGHT;

fn de(flags: Flags, selected: Selected) -> Style {
    let sem_trocas = |_: usize| -> Option<Rgb> { None };
    style(
        Color::Named(NamedColor::Red),
        Color::Named(NamedColor::Background),
        flags,
        selected,
        &PALETA,
        &sem_trocas,
    )
}

#[test]
fn celula_simples_usa_as_cores_dela() {
    let estilo = de(Flags::empty(), Selected::No);
    assert_eq!((estilo.fg, estilo.bg), (PALETA.ansi[1], PALETA.background));
    assert_eq!(
        (estilo.weight, estilo.slant, estilo.decoration),
        (Weight::Normal, Slant::Upright, Decoration::None)
    );
}

#[test]
fn inverso_troca_frente_e_fundo_e_oculto_some() {
    let inverso = de(Flags::INVERSE, Selected::No);
    assert_eq!(
        (inverso.fg, inverso.bg),
        (PALETA.background, PALETA.ansi[1])
    );
    let oculto = de(Flags::HIDDEN, Selected::No);
    assert_eq!(oculto.fg, oculto.bg);
}

#[test]
fn negrito_italico_sublinhado_riscado_e_esmaecido() {
    assert_eq!(de(Flags::BOLD, Selected::No).weight, Weight::Bold);
    assert_eq!(de(Flags::ITALIC, Selected::No).slant, Slant::Italic);
    assert_eq!(
        de(Flags::UNDERLINE, Selected::No).decoration,
        Decoration::Underline
    );
    assert_eq!(
        de(Flags::UNDERCURL, Selected::No).decoration,
        Decoration::Underline
    );
    assert_eq!(
        de(Flags::STRIKEOUT, Selected::No).decoration,
        Decoration::Strikeout
    );
    assert_eq!(de(Flags::DIM, Selected::No).fg, dim(PALETA.ansi[1]));
}

#[test]
fn a_selecao_usa_as_cores_de_selecao_do_tema() {
    let selecionada = de(Flags::empty(), Selected::Yes);
    assert_eq!(
        (selecionada.fg, selecionada.bg),
        (PALETA.selection_foreground, PALETA.selection_background)
    );
}
