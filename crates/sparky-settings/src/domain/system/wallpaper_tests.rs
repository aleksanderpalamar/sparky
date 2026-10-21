use std::path::PathBuf;

use sparky_theme::Palette;

use crate::domain::system::themes::{Catalog, Theme, Themes};
use crate::domain::system::wallpaper::{row, step};
use crate::domain::system::{Control, RowId};
use crate::domain::value::Step;

fn tema(nome: &str, cor: &str, fundos: &[&str]) -> Theme {
    let Ok(tema) = Theme::parse(nome, &format!("accent = \"{cor}\"")) else { panic!("tema inválido") };
    tema.with_wallpapers(fundos.iter().map(PathBuf::from).collect())
}

fn temas(atual: &str) -> Themes {
    let nord = tema("nord", "#010203", &["/w/nord/1-a.jpg", "/w/nord/2-b.webp", "/w/nord/3-c.png"]);
    let liso = tema("liso", "#040506", &[]);
    let current = match atual {
        "nord" => nord.palette(),
        _ => liso.palette(),
    };
    Themes::Ready(Catalog { themes: vec![nord, liso], current })
}

#[test]
fn a_linha_mostra_a_posicao_do_fundo_no_tema_em_uso() {
    let Some(linha) = row(&temas("nord"), Some("/w/nord/2-b.webp")) else { panic!("sem linha") };
    assert_eq!(linha.id, RowId::Wallpaper);
    assert_eq!(linha.label, "Papel de parede");
    assert_eq!(linha.value, "2 de 3 · 2-b.webp");
    assert_eq!(linha.control, Control::Choice);
}

#[test]
fn fundo_de_fora_do_tema_aparece_como_outro() {
    let linha = row(&temas("nord"), Some("/home/ana/meu.jpg")).map(|linha| linha.value);
    assert_eq!(linha.as_deref(), Some("3 do tema · outro em uso"));
}

#[test]
fn tema_sem_fundos_nao_tem_linha() {
    assert!(row(&temas("liso"), None).is_none());
    assert!(row(&Themes::Unavailable("erro".to_owned()), None).is_none());
}

#[test]
fn h_e_l_percorrem_os_fundos_em_roda() {
    let temas = temas("nord");
    assert_eq!(step(&temas, Some("/w/nord/2-b.webp"), Step::More), Some(PathBuf::from("/w/nord/3-c.png")));
    assert_eq!(step(&temas, Some("/w/nord/3-c.png"), Step::More), Some(PathBuf::from("/w/nord/1-a.jpg")));
    assert_eq!(step(&temas, Some("/w/nord/1-a.jpg"), Step::Less), Some(PathBuf::from("/w/nord/3-c.png")));
    assert_eq!(step(&temas, None, Step::More), Some(PathBuf::from("/w/nord/1-a.jpg")));
    assert_eq!(step(&temas, Some("/outro.jpg"), Step::Less), Some(PathBuf::from("/w/nord/3-c.png")));
}

#[test]
fn o_tema_aplicado_traz_o_primeiro_fundo() {
    let nord = tema("nord", "#010203", &["/w/nord/1-a.jpg"]);
    assert_eq!(nord.wallpapers().first(), Some(&PathBuf::from("/w/nord/1-a.jpg")));
    assert_eq!(Theme::builtin().wallpapers(), &[] as &[PathBuf]);
    assert_ne!(nord.palette(), Palette::TOKYO_NIGHT);
}
