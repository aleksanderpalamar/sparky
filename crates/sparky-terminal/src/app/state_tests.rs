use sparky_theme::{Palette, Skin, TerminalPalette};

use crate::app::state::{Changed, Terminal};
use crate::domain::grid::Cell;
use crate::domain::launch::{Launch, Program};

const NORD: &str =
    "background = \"#2e3440\"\nforeground = \"#d8dee9\"\naccent = \"#88c0d0\"\nred = \"#bf616a\"\n";

fn terminal() -> Terminal {
    let launch = Launch {
        program: Program::Shell,
        directory: None,
    };
    Terminal::new(
        launch,
        Palette::TOKYO_NIGHT,
        TerminalPalette::TOKYO_NIGHT,
        Cell {
            width: 8.0,
            height: 16.0,
        },
    )
}

#[test]
fn trocar_o_tema_no_sparky_muda_as_cores_do_terminal_aberto() {
    let mut aberto = terminal();
    let (paleta, cores) = (
        Palette::from_colors(NORD),
        TerminalPalette::from_colors(NORD),
    );
    assert_eq!(aberto.observe_colors(paleta, cores), Changed::Yes);
    assert_eq!(aberto.palette, cores);
    assert_eq!(aberto.skin, Skin::new(paleta));
    assert_eq!(
        aberto.palette.ansi[1],
        TerminalPalette::from_colors(NORD).ansi[1]
    );
}

#[test]
fn as_mesmas_cores_nao_redesenham() {
    let mut aberto = terminal();
    assert_eq!(
        aberto.observe_colors(Palette::TOKYO_NIGHT, TerminalPalette::TOKYO_NIGHT),
        Changed::No
    );
}

#[test]
fn a_janela_define_a_grade_com_a_margem_do_sparky() {
    let mut aberto = terminal();
    aberto.fit(900.0, 560.0);
    assert_eq!((aberto.size.columns, aberto.size.lines), (108, 32));
    assert_eq!(aberto.title, "Terminal");
}
