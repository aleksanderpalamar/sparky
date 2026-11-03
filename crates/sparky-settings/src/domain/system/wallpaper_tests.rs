use std::path::PathBuf;

use sparky_theme::Palette;

use crate::domain::system::themes::{Catalog, Theme, Themes};
use crate::domain::system::wallpaper::{chosen, row, step};
use crate::domain::system::{Control, RowId};
use crate::domain::value::Step;

fn tema(nome: &str, cor: &str, fundos: &[&str]) -> Theme {
    let Ok(tema) = Theme::parse(nome, &format!("accent = \"{cor}\"")) else {
        panic!("tema inválido")
    };
    tema.with_wallpapers(fundos.iter().map(PathBuf::from).collect())
}

fn temas(atual: &str) -> Themes {
    let nord = tema(
        "nord",
        "#010203",
        &["/w/nord/1-a.jpg", "/w/nord/2-b.webp", "/w/nord/3-c.png"],
    );
    let liso = tema("liso", "#040506", &[]);
    let current = match atual {
        "nord" => nord.palette(),
        _ => liso.palette(),
    };
    Themes::Ready(Catalog {
        themes: vec![nord, liso],
        current,
    })
}

#[test]
fn a_linha_mostra_a_posicao_do_fundo_no_tema_em_uso() {
    let Some(linha) = row(&temas("nord"), Some("/w/nord/2-b.webp"), RowId::Wallpaper) else {
        panic!("sem linha")
    };
    assert_eq!(linha.id, RowId::Wallpaper);
    assert_eq!(linha.label, "Papel de parede");
    assert_eq!(linha.value, "2 de 3 · 2-b.webp");
    let nomes = ["1-a.jpg", "2-b.webp", "3-c.png"]
        .map(str::to_owned)
        .to_vec();
    assert_eq!(linha.control, Control::Choice(nomes));
}

#[test]
fn fundo_de_fora_do_tema_aparece_como_outro() {
    let linha =
        row(&temas("nord"), Some("/home/ana/meu.jpg"), RowId::Wallpaper).map(|linha| linha.value);
    assert_eq!(linha.as_deref(), Some("3 do tema · outro em uso"));
}

#[test]
fn tema_sem_fundos_nao_tem_linha() {
    assert!(row(&temas("liso"), None, RowId::Wallpaper).is_none());
    assert!(
        row(
            &Themes::Unavailable("erro".to_owned()),
            None,
            RowId::Wallpaper
        )
        .is_none()
    );
}

#[test]
fn h_e_l_percorrem_os_fundos_em_roda() {
    let temas = temas("nord");
    assert_eq!(
        step(&temas, Some("/w/nord/2-b.webp"), Step::More),
        Some(PathBuf::from("/w/nord/3-c.png"))
    );
    assert_eq!(
        step(&temas, Some("/w/nord/3-c.png"), Step::More),
        Some(PathBuf::from("/w/nord/1-a.jpg"))
    );
    assert_eq!(
        step(&temas, Some("/w/nord/1-a.jpg"), Step::Less),
        Some(PathBuf::from("/w/nord/3-c.png"))
    );
    assert_eq!(
        step(&temas, None, Step::More),
        Some(PathBuf::from("/w/nord/1-a.jpg"))
    );
    assert_eq!(
        step(&temas, Some("/outro.jpg"), Step::Less),
        Some(PathBuf::from("/w/nord/3-c.png"))
    );
}

#[test]
fn o_tema_aplicado_traz_o_primeiro_fundo() {
    let nord = tema("nord", "#010203", &["/w/nord/1-a.jpg"]);
    assert_eq!(
        nord.wallpapers().first(),
        Some(&PathBuf::from("/w/nord/1-a.jpg"))
    );
    assert_eq!(Theme::builtin().wallpapers(), &[] as &[PathBuf]);
    assert_ne!(nord.palette(), Palette::TOKYO_NIGHT);
}

#[test]
fn escolher_um_fundo_pelo_nome_do_arquivo() {
    let nord = temas("nord");
    assert_eq!(
        chosen(&nord, "2-b.webp"),
        Some(PathBuf::from("/w/nord/2-b.webp"))
    );
    assert_eq!(
        chosen(&nord, " 3-c.png "),
        Some(PathBuf::from("/w/nord/3-c.png"))
    );
    assert_eq!(chosen(&nord, "outro.jpg"), None);
    assert_eq!(chosen(&temas("liso"), "2-b.webp"), None);
}

#[test]
fn fundos_com_o_mesmo_nome_viram_opcoes_distintas() {
    let nord = tema(
        "nord",
        "#010203",
        &[
            "/pessoal/nord/a.jpg",
            "/pessoal/nord/b.jpg",
            "/sistema/nord/a.jpg",
        ],
    );
    let temas = Themes::Ready(Catalog {
        current: nord.palette(),
        themes: vec![nord],
    });
    let Some(linha) = row(&temas, Some("/sistema/nord/a.jpg"), RowId::Wallpaper) else {
        panic!("sem linha")
    };
    let Control::Choice(opcoes) = linha.control else {
        panic!("sem lista")
    };
    assert_eq!(opcoes, ["a.jpg", "b.jpg", "a.jpg (2)"]);
    assert_eq!(linha.value, "3 de 3 · a.jpg (2)");
    assert_eq!(
        chosen(&temas, "a.jpg (2)"),
        Some(PathBuf::from("/sistema/nord/a.jpg"))
    );
    assert_eq!(
        chosen(&temas, "a.jpg"),
        Some(PathBuf::from("/pessoal/nord/a.jpg"))
    );
}
