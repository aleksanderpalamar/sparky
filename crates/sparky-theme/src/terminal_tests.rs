use crate::rgb::Rgb;
use crate::terminal::TerminalPalette;

const NORD: &str = "background = \"#2e3440\"\nforeground = \"#d8dee9\"\nbright_foreground = \"#eceff4\"\nselection = \"#434c5e\"\nmuted = \"#4c566a\"\nred = \"#bf616a\"\ngreen = \"#a3be8c\"\nyellow = \"#ebcb8b\"\nblue = \"#81a1c1\"\nmagenta = \"#b48ead\"\ncyan = \"#88c0d0\"\nbright_red = \"#d08770\"\nbright_green = \"#b9d79f\"\nbright_yellow = \"#f0d399\"\nbright_blue = \"#a3bcd6\"\nbright_magenta = \"#c895bf\"\nbright_cyan = \"#9fd4e0\"\n";

#[test]
fn as_dezesseis_cores_seguem_a_ordem_do_kitty_do_tema() {
    let paleta = TerminalPalette::from_colors(NORD);
    let esperadas = [
        "#2e3440", "#bf616a", "#a3be8c", "#ebcb8b", "#81a1c1", "#b48ead", "#88c0d0", "#d8dee9",
        "#4c566a", "#d08770", "#b9d79f", "#f0d399", "#a3bcd6", "#c895bf", "#9fd4e0", "#eceff4",
    ];
    for (indice, cor) in esperadas.iter().enumerate() {
        assert_eq!(Some(paleta.ansi[indice]), Rgb::parse(cor), "cor {indice}");
    }
}

#[test]
fn cursor_e_selecao_vem_do_tema() {
    let paleta = TerminalPalette::from_colors(NORD);
    assert_eq!(Some(paleta.foreground), Rgb::parse("#d8dee9"));
    assert_eq!(Some(paleta.background), Rgb::parse("#2e3440"));
    assert_eq!(Some(paleta.cursor), Rgb::parse("#eceff4"));
    assert_eq!(Some(paleta.cursor_text), Rgb::parse("#2e3440"));
    assert_eq!(Some(paleta.selection_background), Rgb::parse("#434c5e"));
    assert_eq!(Some(paleta.selection_foreground), Rgb::parse("#eceff4"));
}

#[test]
fn chaves_ausentes_usam_o_tokyo_night() {
    let paleta = TerminalPalette::from_colors("red = \"#010203\"");
    assert_eq!(paleta.ansi[1], Rgb::new(1, 2, 3));
    assert_eq!(paleta.ansi[2], TerminalPalette::TOKYO_NIGHT.ansi[2]);
    assert_eq!(paleta.background, TerminalPalette::TOKYO_NIGHT.background);
}

#[test]
fn o_tokyo_night_embutido_e_o_do_kitty_do_usuario() {
    let paleta = TerminalPalette::TOKYO_NIGHT;
    assert_eq!(paleta.ansi[0], Rgb::new(0x1a, 0x1b, 0x26));
    assert_eq!(paleta.ansi[8], Rgb::new(0x41, 0x48, 0x68));
    assert_eq!(paleta.ansi[15], Rgb::new(0xc0, 0xca, 0xf5));
    let pacote = include_str!("../../../data/themes/tokyo-night/colors.toml");
    assert_eq!(TerminalPalette::from_colors(pacote), paleta);
}

#[test]
fn as_chaves_do_terminal_valem_so_no_terminal() {
    let cores = "background = \"#2b2b2b\"\nforeground = \"#f7f7f7\"\nred = \"#c7162b\"\nselection = \"#5e2750\"\nterminal_background = \"#300a24\"\nterminal_foreground = \"#ffffff\"\nterminal_red = \"#c01c28\"\nterminal_selection = \"#77216f\"\n";
    let paleta = TerminalPalette::from_colors(cores);
    assert_eq!(Some(paleta.background), Rgb::parse("#300a24"));
    assert_eq!(Some(paleta.cursor_text), Rgb::parse("#300a24"));
    assert_eq!(Some(paleta.ansi[0]), Rgb::parse("#300a24"));
    assert_eq!(Some(paleta.foreground), Rgb::parse("#ffffff"));
    assert_eq!(Some(paleta.ansi[7]), Rgb::parse("#ffffff"));
    assert_eq!(Some(paleta.ansi[1]), Rgb::parse("#c01c28"));
    assert_eq!(Some(paleta.selection_background), Rgb::parse("#77216f"));
    assert_eq!(
        crate::Palette::from_colors(cores).background,
        Rgb::new(0x2b, 0x2b, 0x2b)
    );
}

#[test]
fn sem_chave_do_terminal_vale_a_cor_comum() {
    let paleta = TerminalPalette::from_colors("red = \"#010203\"\nterminal_green = \"#040506\"");
    assert_eq!(paleta.ansi[1], Rgb::new(1, 2, 3));
    assert_eq!(paleta.ansi[2], Rgb::new(4, 5, 6));
}

#[test]
fn os_temas_spark_tem_terminal_proprio() {
    let spark =
        TerminalPalette::from_colors(include_str!("../../../data/themes/spark/colors.toml"));
    assert_eq!(Some(spark.background), Rgb::parse("#300a24"));
    let classico =
        TerminalPalette::from_colors(include_str!("../../../data/themes/spark-98/colors.toml"));
    assert_eq!(Some(classico.background), Rgb::parse("#000000"));
    assert_eq!(Some(classico.foreground), Rgb::parse("#c0c0c0"));
}
