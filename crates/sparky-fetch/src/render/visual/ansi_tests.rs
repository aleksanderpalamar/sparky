use sparky_theme::Rgb;

use crate::render::visual::ansi;

#[test]
fn pinta_com_cor_de_24_bits_e_reinicia_no_fim() {
    assert_eq!(
        ansi::paint("Sparky", Rgb::new(0x7a, 0xa2, 0xf7)),
        "\x1b[38;2;122;162;247mSparky\x1b[0m"
    );
}

#[test]
fn limpa_sequencias_de_escape_e_caracteres_de_controle() {
    assert_eq!(ansi::strip("\x1b[1;31mvermelho\x1b[0m"), "vermelho");
    assert_eq!(ansi::strip("\x1b]0;titulo\x07texto"), "texto");
    assert_eq!(ansi::strip("\x1b]0;titulo\x1b\\texto"), "texto");
    assert_eq!(ansi::strip("um\tdois\nfim\u{7}"), "um dois fim");
}

#[test]
fn mede_a_largura_visivel_ignorando_escapes() {
    assert_eq!(ansi::width("\x1b[38;2;1;2;3mMemória\x1b[0m"), 7);
    assert_eq!(ansi::width("日本"), 4);
    assert_eq!(ansi::width("███ …"), 5);
}
