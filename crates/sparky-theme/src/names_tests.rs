use std::fs;
use std::path::PathBuf;

use crate::locations::theme_roots;
use crate::names::{installed_themes, theme_name};
use crate::palette::Palette;
use crate::rgb::Rgb;

const AZUL: &str = "accent = \"#0000ff\"\n";
const VERDE: &str = "accent = \"#00ff00\"\n";

fn temas() -> Vec<(String, String)> {
    vec![
        ("azul".to_owned(), AZUL.to_owned()),
        ("verde".to_owned(), VERDE.to_owned()),
    ]
}

#[test]
fn o_tema_em_uso_e_o_que_tem_a_mesma_paleta() {
    let atual = Palette::from_colors(VERDE);
    assert_eq!(theme_name(&atual, &temas()).as_deref(), Some("verde"));
}

#[test]
fn paleta_editada_a_mao_nao_tem_nome() {
    let atual = Palette::from_colors("accent = \"#123456\"\n");
    assert_eq!(theme_name(&atual, &temas()), None);
}

#[test]
fn os_temas_instalados_saem_em_ordem_e_o_do_usuario_vem_primeiro() {
    let pasta = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let usuario = pasta.path().join("usuario");
    let sistema = pasta.path().join("sistema");
    for (raiz, nome, cores) in [
        (&sistema, "verde", VERDE),
        (&sistema, "azul", AZUL),
        (&usuario, "zeta", AZUL),
    ] {
        fs::create_dir_all(raiz.join(nome)).unwrap_or_else(|error| panic!("{error}"));
        fs::write(raiz.join(nome).join("colors.toml"), cores)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    fs::create_dir_all(sistema.join("vazio")).unwrap_or_else(|error| panic!("{error}"));
    let nomes: Vec<String> = installed_themes(&[usuario, sistema])
        .into_iter()
        .map(|(nome, _)| nome)
        .collect();
    assert_eq!(nomes, ["zeta", "azul", "verde"]);
}

#[test]
fn as_pastas_de_temas_seguem_as_do_painel() {
    assert_eq!(
        theme_roots(Some("/dados"), "/home/p", Some("/sparky")),
        vec![
            PathBuf::from("/dados/sparky/themes"),
            PathBuf::from("/sparky/themes")
        ]
    );
    assert_eq!(
        theme_roots(None, "/home/p", None),
        vec![
            PathBuf::from("/home/p/.local/share/sparky/themes"),
            PathBuf::from("/usr/share/sparky/themes")
        ]
    );
}

#[test]
fn a_cor_vira_hexadecimal() {
    assert_eq!(Rgb::new(0x7a, 0xa2, 0xf7).hex(), "#7aa2f7");
}
