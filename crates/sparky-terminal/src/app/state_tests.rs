use sparky_theme::{Palette, Skin, TerminalPalette};

use crate::app::state::{Changed, Terminal};
use crate::domain::grid::Cell;
use crate::domain::launch::{Launch, Program};

const NORD: &str =
    "background = \"#2e3440\"\nforeground = \"#d8dee9\"\naccent = \"#88c0d0\"\nred = \"#bf616a\"\n";

pub fn terminal() -> Terminal {
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

#[test]
fn a_escala_da_janela_chega_ao_tamanho_do_pty() {
    let mut aberto = terminal();
    aberto.rescale(2.0);
    assert_eq!(aberto.size.scale, 2.0);
    assert_eq!(aberto.size.window().cell_width, 16);
}

#[test]
fn o_ime_abre_na_celula_do_cursor() {
    use crate::domain::screen::CursorSpot;
    let mut aberto = terminal();
    aberto.screen.cursor = Some(CursorSpot {
        line: 2,
        column: 3,
        character: ' ',
        color: sparky_theme::Rgb::new(0, 0, 0),
    });
    let area = aberto.ime_area();
    assert_eq!(
        (area.x, area.y, area.width, area.height),
        (18.0 + 24.0, 18.0 + 32.0, 8.0, 16.0)
    );
}

#[test]
fn alternar_entre_spark_e_spark_98_troca_o_fundo_do_terminal_aberto() {
    let spark = include_str!("../../../../data/themes/spark/colors.toml");
    let classico = include_str!("../../../../data/themes/spark-98/colors.toml");
    let mut aberto = terminal();
    for (tema, fundo) in [
        (spark, "#300a24"),
        (classico, "#000000"),
        (spark, "#300a24"),
    ] {
        let cores = TerminalPalette::from_colors(tema);
        assert_eq!(
            aberto.observe_colors(Palette::from_colors(tema), cores),
            Changed::Yes
        );
        assert_eq!(
            Some(aberto.palette.background),
            sparky_theme::Rgb::parse(fundo)
        );
    }
    assert_ne!(
        Palette::from_colors(spark).background,
        aberto.palette.background
    );
}

#[test]
fn mudar_so_as_cores_do_terminal_redesenha() {
    let mut aberto = terminal();
    let cores = TerminalPalette::from_colors("terminal_background = \"#300a24\"");
    assert_eq!(
        aberto.observe_colors(Palette::TOKYO_NIGHT, cores),
        Changed::Yes
    );
    assert_eq!(
        aberto.palette.background,
        sparky_theme::Rgb::new(0x30, 0x0a, 0x24)
    );
}
